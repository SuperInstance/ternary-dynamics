# ternary-dynamics

[![Crates.io](https://img.shields.io/crates/v/ternary-dynamics.svg)](https://crates.io/crates/ternary-dynamics)
[![Documentation](https://docs.rs/ternary-dynamics/badge.svg)](https://docs.rs/ternary-dynamics)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)

Temporal dynamics of ternary agent systems — how strategies evolve over time, phase transitions, and critical points.

## Theory

In ternary agent systems, every agent operates under one of three fundamental strategies:

- **Avoid** — retreat from interaction, minimize risk and exposure
- **Cooperate** — seek mutual benefit through collaboration
- **Exploit** — take advantage of others for personal gain

The interplay of these strategies over time produces rich dynamics analogous to physical systems:

### Phase Transitions

Much like matter transitioning between solid, liquid, and gas, ternary populations undergo sudden shifts in collective behavior. A population of cooperators can collapse into avoidance in a single generation — a **cascade** — when exploitation pressure crosses a critical threshold. These transitions are marked by discontinuities in population-level metrics like diversity and avoidance ratio.

### Critical Points

Critical points mark generations where the system's behavior fundamentally changes:

- **Fitness plateaus** — the population reaches an evolutionary equilibrium
- **Diversity cliffs** — rapid loss of strategy variety, often presaging monoculture
- **Avoidance spikes** — sudden retreat behavior, a hallmark of cascading fear
- **Population crashes** — mass die-offs triggered by over-exploitation

Identifying these points allows researchers to predict and potentially steer system outcomes.

### Dynamic Modes

Over time, ternary systems can be classified into four dynamic regimes:

| Mode | Description |
|------|-------------|
| **Converging** | Metrics trend consistently in one direction |
| **Oscillating** | Regular cycling between states (predator-prey dynamics) |
| **Chaotic** | Unpredictable, sensitive to initial conditions |
| **Stable** | Little change; system has reached equilibrium |

The `ModeClassifier` analyzes time series data to determine which regime a system is in.

### Trajectories

Individual agents don't just follow population trends. A **Trajectory** tracks a single agent's strategy changes over time, enabling analysis of:

- **Loyalty** — agents who never change strategy
- **Oscillation** — agents who switch back to a previous strategy
- **Transition rates** — how often agents change their approach

## Usage

```rust
use ternary_dynamics::*;

// Track population metrics over time
let mut ts = TimeSeries::new();
ts.record(0, PopulationMetrics::from_counts(33, 34, 33));
ts.record(1, PopulationMetrics::from_counts(10, 50, 40));
ts.record(2, PopulationMetrics::from_counts(5, 80, 15));

// Detect phase transitions
let detector = PhaseDetector::default();
let events = detector.detect(&ts.generations, &ts.diversity_series());

// Find critical points
let cd = CriticalDetector::new();
let critical = cd.detect_all(
    &ts.generations,
    &ts.fitness_series(),
    &ts.diversity_series(),
    &ts.avoidance_series(),
    &ts.metrics.iter().map(|m| m.population_size).collect::<Vec<_>>(),
);

// Classify dynamics
let classifier = ModeClassifier::new();
let mode = classifier.classify(&ts.fitness_series());

// Track individual agents
let mut traj = Trajectory::new(42);
traj.adopt(TernaryStrategy::Cooperate, 0);
traj.adopt(TernaryStrategy::Exploit, 50);
assert_eq!(traj.transition_count(), 1);

// Log detailed per-generation state
let mut logger = GenerationLogger::new();
logger.log(GenerationSnapshot::new(0, [33, 34, 33], 0.65, 0.66));
```

## Features

- **TimeSeries** — track fitness, diversity, and avoidance ratio over generations
- **PhaseTransition** / **PhaseDetector** — detect sudden behavioral shifts
- **CriticalPoint** / **CriticalDetector** — identify fitness plateaus, diversity cliffs, avoidance spikes
- **GenerationLogger** — record detailed per-generation snapshots for replay
- **DynamicMode** / **ModeClassifier** — classify system state (converging, oscillating, chaotic, stable)
- **Trajectory** — track individual agent strategy evolution

## Design

- Pure Rust, no unsafe code, no external dependencies
- Edition 2021, MIT licensed
- All metrics computed from raw strategy counts — no simulation engine included

## License

MIT
