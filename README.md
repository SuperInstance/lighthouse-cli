# Forgemaster's Lighthouse

Task routing and safety gate for a multi-agent AI fleet. Classifies tasks, assigns the cheapest appropriate model, configures agent rooms, and screens output before it leaves the fleet.

## Pipeline

```
orient → relay → gate
```

1. **orient** — classify a task, pick a model, create an agent room
2. **relay** — configure the room (seed iterations, status)
3. **gate** — screen the agent's output before it's used

## Model Routing

| Model | Tier cost | Task types |
|---|---|---|
| `claude` | 50.0/day | synthesis, critique, big-idea |
| `glm-5.1` | 5.0/month | architecture, complex-code, orchestration |
| `seed-2.0-mini` | 0.1 | discovery, exploration, drafting, variation |
| `deepseek-flash` | 0.2 | documentation, research, drafting |
| `hermes-70b` | 0.15 | adversarial, second-opinion |

Routing picks the cheapest model with capacity > 10% that fits the task type.

## Installation

**From source:**
```sh
git clone <repo>
cd lighthouse-cli
cargo build --release
cp target/release/lighthouse ~/.local/bin/
```

**Cargo install:**
```sh
cargo install --path .
```

## Usage

### orient
Classify a task and get a model + room assignment.
```sh
lighthouse orient "design the API layer" --type Architecture
lighthouse orient "write a blog draft" --type Drafting
lighthouse orient "punch holes in this plan" --type Adversarial
```
Valid types: `Synthesis`, `Critique`, `BigIdea`, `Architecture`, `ComplexCode`, `Orchestration`, `Discovery`, `Exploration`, `Drafting`, `Variation`, `Documentation`, `Research`, `Adversarial`, `SecondOpinion`

### relay
Configure a room returned by `orient`.
```sh
lighthouse relay agent-3f2a1b00 --seeds 50
lighthouse relay agent-3f2a1b00 --seeds 0
```
`--seeds N` sets iteration count; `0` transitions status to `running`.

### gate
Pipe agent output through the safety gate.
```sh
lighthouse gate agent-3f2a1b00 < agent_output.txt
cat output.txt | lighthouse gate agent-3f2a1b00
```

Exit codes: `0` = approved or needs-approval, `1` = rejected.

### status / models
```sh
lighthouse status    # capacity bars for all model tiers
lighthouse models    # routing table with costs and task types
```

## Gate Safety Checks

| Check | Trigger | Result |
|---|---|---|
| Credential leak | `api_key=`, `password=`, `secret=`, `bearer ` | REJECTED |
| External action | `send_email`, `post_tweet`, `npm publish`, `deploy` | NEEDS APPROVAL |
| Overclaim | `we have proven`, `this proves`, `proven that` | REJECTED |

`NEEDS APPROVAL` warnings print to stdout but do not fail (exit 0). `REJECTED` exits 1.
