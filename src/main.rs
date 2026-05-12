//! Forgemaster's Lighthouse CLI — orient, relay, gate.
//!
//! Usage:
//!   lighthouse orient "design the API" --type Architecture
//!   lighthouse relay <room-id> --seeds 50
//!   lighthouse gate <room-id> < output.txt
//!   lighthouse status
//!   lighthouse models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Lighthouse Types ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ModelTier {
    Claude,
    GLM,
    Seed,
    DeepSeek,
    Hermes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TaskType {
    Synthesis,
    Critique,
    BigIdea,
    Architecture,
    ComplexCode,
    Orchestration,
    Discovery,
    Exploration,
    Drafting,
    Variation,
    Documentation,
    Research,
    Adversarial,
    SecondOpinion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GateResult {
    Approved,
    Rejected(String),
    NeedsApproval(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentRoom {
    room_id: String,
    role: String,
    model: String,
    task_type: String,
    seed_iterations: usize,
    status: String,
    created_at: u64,
}

struct Lighthouse {
    agents: HashMap<String, AgentRoom>,
    capacity: HashMap<ModelTier, f64>,
}

impl Lighthouse {
    fn new() -> Self {
        let mut cap = HashMap::new();
        cap.insert(ModelTier::Claude, 1.0);
        cap.insert(ModelTier::GLM, 1.0);
        cap.insert(ModelTier::Seed, 1.0);
        cap.insert(ModelTier::DeepSeek, 1.0);
        cap.insert(ModelTier::Hermes, 1.0);
        Lighthouse {
            agents: HashMap::new(),
            capacity: cap,
        }
    }

    fn orient(&mut self, task: &str, task_type: TaskType) -> AgentRoom {
        let model = self.cheapest_appropriate(task_type);
        let room_id = format!("agent-{:08x}", simple_hash(task));
        let agent = AgentRoom {
            room_id: room_id.clone(),
            role: task.to_string(),
            model: format_tier(model).to_string(),
            task_type: format!("{:?}", task_type),
            seed_iterations: 0,
            status: "orienting".to_string(),
            created_at: now_secs(),
        };
        self.agents.insert(room_id, agent.clone());
        agent
    }

    fn relay(&mut self, room_id: &str, seeds: usize) -> Option<AgentRoom> {
        let agent = self.agents.get_mut(room_id)?;
        agent.seed_iterations = seeds;
        agent.status = if seeds > 0 {
            "seeding".to_string()
        } else {
            "running".to_string()
        };
        Some(self.agents.get(room_id)?.clone())
    }

    fn gate(&mut self, room_id: &str, output: &str) -> GateResult {
        // Check 1: credential leaks
        let lower = output.to_lowercase();
        if lower.contains("api_key=")
            || lower.contains("password=")
            || lower.contains("secret=")
            || lower.contains("bearer ")
        {
            return GateResult::Rejected("Credential leak detected".into());
        }
        // Check 2: external actions
        let ext = ["send_email", "post_tweet", "npm publish", "deploy"];
        for m in &ext {
            if output.contains(m) {
                return GateResult::NeedsApproval(format!("External action: {}", m));
            }
        }
        // Check 3: overclaims
        let claims = ["we have proven", "this proves", "proven that"];
        for m in &claims {
            if lower.contains(m) {
                return GateResult::Rejected("Overclaim detected — falsify first".into());
            }
        }
        GateResult::Approved
    }

    fn cheapest_appropriate(&self, tt: TaskType) -> ModelTier {
        let tiers = [
            ModelTier::Seed,
            ModelTier::Hermes,
            ModelTier::DeepSeek,
            ModelTier::GLM,
            ModelTier::Claude,
        ];
        for &t in &tiers {
            if appropriate(t, tt) {
                if let Some(c) = self.capacity.get(&t) {
                    if *c > 0.1 {
                        return t;
                    }
                }
            }
        }
        ModelTier::Seed
    }

    fn resource_summary(&self) -> String {
        let mut lines = vec!["LIGHTHOUSE RESOURCE STATUS".to_string()];
        for (tier, rem) in &self.capacity {
            let n = (*rem * 20.0) as usize;
            let bar = "█".repeat(n) + &"░".repeat(20 - n);
            lines.push(format!("  {:?}: [{}] {:.0}%", tier, bar, rem * 100.0));
        }
        lines.push(format!("  Active agents: {}", self.agents.len()));
        lines.join("\n")
    }
}

fn appropriate(tier: ModelTier, tt: TaskType) -> bool {
    match (tier, tt) {
        (ModelTier::Claude, TaskType::Synthesis | TaskType::Critique | TaskType::BigIdea) => true,
        (
            ModelTier::GLM,
            TaskType::Architecture | TaskType::ComplexCode | TaskType::Orchestration,
        ) => true,
        (
            ModelTier::Seed,
            TaskType::Discovery | TaskType::Exploration | TaskType::Drafting | TaskType::Variation,
        ) => true,
        (
            ModelTier::DeepSeek,
            TaskType::Documentation | TaskType::Research | TaskType::Drafting,
        ) => true,
        (ModelTier::Hermes, TaskType::Adversarial | TaskType::SecondOpinion) => true,
        _ => false,
    }
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_tier(t: ModelTier) -> &'static str {
    match t {
        ModelTier::Claude => "claude (synthesis)",
        ModelTier::GLM => "glm-5.1 (architecture)",
        ModelTier::Seed => "seed-2.0-mini (discovery)",
        ModelTier::DeepSeek => "deepseek-flash (docs)",
        ModelTier::Hermes => "hermes-70b (adversarial)",
    }
}

fn parse_task_type(s: &str) -> TaskType {
    match s.to_lowercase().as_str() {
        "synthesis" => TaskType::Synthesis,
        "critique" => TaskType::Critique,
        "bigidea" => TaskType::BigIdea,
        "architecture" => TaskType::Architecture,
        "complexcode" => TaskType::ComplexCode,
        "orchestration" => TaskType::Orchestration,
        "discovery" => TaskType::Discovery,
        "exploration" => TaskType::Exploration,
        "drafting" => TaskType::Drafting,
        "variation" => TaskType::Variation,
        "documentation" | "docs" => TaskType::Documentation,
        "research" => TaskType::Research,
        "adversarial" => TaskType::Adversarial,
        "secondopinion" => TaskType::SecondOpinion,
        _ => TaskType::Drafting,
    }
}

// ─── CLI ──────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Forgemaster's Lighthouse ⚒️\n");
        eprintln!("Usage:");
        eprintln!("  lighthouse orient <task> --type <type>   Classify task, pick model");
        eprintln!("  lighthouse relay <room> --seeds <n>       Configure agent");
        eprintln!("  lighthouse gate <room>                     Gate stdin output");
        eprintln!("  lighthouse status                          Resource summary");
        eprintln!("  lighthouse models                          List model tiers");
        std::process::exit(1);
    }

    let mut lh = Lighthouse::new();

    match args[1].as_str() {
        "orient" => {
            let default_task = "unnamed task".to_string();
            let task = args.get(2).unwrap_or(&default_task);
            let tt = args
                .iter()
                .position(|a| a == "--type")
                .and_then(|i| args.get(i + 1))
                .map(|s| parse_task_type(s))
                .unwrap_or(TaskType::Drafting);
            let agent = lh.orient(task, tt);
            println!("🧭 ORIENTED");
            println!("  Room:   {}", agent.room_id);
            println!("  Task:   {}", agent.role);
            println!("  Type:   {}", agent.task_type);
            println!("  Model:  {}", agent.model);
            println!("\nNext: lighthouse relay {} --seeds <n>", agent.room_id);
        }
        "relay" => {
            let default_room = String::new();
            let room = args.get(2).unwrap_or(&default_room);
            let seeds: usize = args
                .iter()
                .position(|a| a == "--seeds")
                .and_then(|i| args.get(i + 1))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            // Create a dummy room for demo
            lh.orient("relay task", TaskType::Drafting);
            // Override room_id
            match lh.relay(room, seeds) {
                Some(agent) => {
                    println!("📡 RELAYED");
                    println!("  Room:   {}", agent.room_id);
                    println!("  Status: {}", agent.status);
                    println!("  Seeds:  {}", agent.seed_iterations);
                }
                None => {
                    eprintln!("❌ Room not found: {}", room);
                    std::process::exit(1);
                }
            }
        }
        "gate" => {
            let default_room2 = "unknown".to_string();
            let room = args.get(2).unwrap_or(&default_room2);
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap_or_default();
            lh.orient("gate task", TaskType::Drafting);
            match lh.gate(room, &input) {
                GateResult::Approved => println!("✅ GATE PASSED"),
                GateResult::Rejected(r) => {
                    println!("❌ REJECTED — {}", r);
                    std::process::exit(1);
                }
                GateResult::NeedsApproval(r) => println!("⚠️  NEEDS APPROVAL — {}", r),
            }
        }
        "status" => println!("{}", lh.resource_summary()),
        "models" => {
            println!("LIGHTHOUSE MODEL TIERS\n");
            for (tier, cost, tasks) in [
                (
                    ModelTier::Claude,
                    "50.0 (daily)",
                    "synthesis, critique, big-idea",
                ),
                (
                    ModelTier::GLM,
                    "5.0 (monthly)",
                    "architecture, complex-code, orchestration",
                ),
                (
                    ModelTier::Seed,
                    "0.1 (cheap)",
                    "discovery, exploration, drafting, variation",
                ),
                (
                    ModelTier::DeepSeek,
                    "0.2 (cheap)",
                    "documentation, research, drafting",
                ),
                (
                    ModelTier::Hermes,
                    "0.15 (cheap)",
                    "adversarial, second-opinion",
                ),
            ] {
                println!("  {:?} — cost: {}", tier, cost);
                println!("    → {}\n", tasks);
            }
        }
        _ => {
            eprintln!(
                "Unknown: {}. Try: orient, relay, gate, status, models",
                args[1]
            );
            std::process::exit(1);
        }
    }
}
