# ternary-dynamics

**Dynamic markings for agent resource control.**

In music, dynamics (pp, p, mp, mf, f, ff) tell performers how loud or soft to play.
Crescendos swell, diminuendos fade, and sforzandos hit suddenly. This crate brings
those expressive markings to ternary agent systems, mapping abstract dynamic levels to
concrete resource allocations.

## Overview

When an agent needs computational resources — CPU time, memory, API calls, attention
budget — the question isn't just *how much* but *how it changes over time*. Musical
dynamics provide a natural vocabulary for this:

- A **piano** agent uses minimal resources, running quietly in the background
- A **forte** agent uses significant resources, running prominently
- A **crescendo** gradually increases resource usage
- A **sforzando** suddenly spikes, then returns to normal

## Core Components

### DynamicMark

Eight dynamic levels from softest to loudest, each mapped to a resource fraction:

| Mark | Notation | Intensity | Fraction |
|------|----------|-----------|----------|
| Ppp  | ppp      | 0.0625    | 6.25%    |
| Pp   | pp       | 0.125     | 12.5%    |
| P    | p        | 0.25      | 25%      |
| Mp   | mp       | 0.375     | 37.5%    |
| Mf   | mf       | 0.5       | 50%      |
| F    | f        | 0.75      | 75%      |
| Ff   | ff       | 0.875     | 87.5%    |
| Fff  | fff      | 1.0       | 100%     |

Mezzo-forte (mf) is the default — the "normal" operating level at 50% resources.

```rust
let mark = DynamicMark::F;
let resources = mark.resource_amount(100.0); // 75.0
assert_eq!(mark.louder(), DynamicMark::Ff);
assert_eq!(mark.softer(), DynamicMark::Mf);
```

### DynamicCurve

Shapes how dynamics change over time. Six curve types:

- **Flat**: Constant level throughout
- **Crescendo**: Gradually increasing from one level to another
- **Diminuendo**: Gradually decreasing
- **Sforzando**: Sudden accent at a specific position, then return to base
- **Swell**: Crescendo followed by diminuendo (rise and fall)
- **Custom**: User-defined sequence of marks

Curves interpolate smoothly between dynamic levels, producing a vector of intensity
values at each step.

```rust
let curve = DynamicCurve::crescendo(DynamicMark::P, DynamicMark::F, 5);
let intensities = curve.evaluate();
// [0.25, ~0.42, ~0.58, ~0.67, 0.75]
```

### DynamicContext

Interprets dynamics based on surrounding context — just as a musician interprets a
forte differently after a pianissimo than after another forte.

- A mark feels **softer** after a very loud mark (contrast effect)
- A mark feels **louder** after a very quiet mark (contrast effect)
- A mark before a loud section is slightly subdued (anticipation)

```rust
let ctx = DynamicContext::new();
let solo = ctx.interpret(DynamicMark::P, None, None);     // 0.25
let after_loud = ctx.interpret(DynamicMark::P, Some(DynamicMark::Fff), None);
assert!(after_loud < solo); // p after fff feels even softer
```

### DynamicBalance

Distributes a shared resource budget across multiple agents based on their dynamics.
Each agent's share is proportional to their dynamic intensity.

```rust
let mut balance = DynamicBalance::new(100.0);
balance.set_agent("researcher", DynamicMark::F);
balance.set_agent("writer", DynamicMark::P);
let alloc = balance.allocate();
// researcher gets ~75%, writer gets ~25%
assert!(balance.is_balanced()); // allocations sum to budget
```

### DynamicInterpreter

Maps abstract dynamic specifications to concrete resource amounts. Supports context
overrides for special situations (e.g., "emergency" context always gets maximum
resources regardless of the dynamic mark).

```rust
let mut interp = DynamicInterpreter::new(200.0);
interp.set_override("emergency", 1.0);
let normal = interp.interpret_with_context(DynamicMark::P, "normal");
let emergency = interp.interpret_with_context(DynamicMark::P, "emergency");
assert!(emergency > normal);
```

## Usage

```rust
use ternary_dynamics::*;

// Simple resource mapping
let budget = 1000.0;
let mark = DynamicMark::Mf;
let allocation = mark.resource_amount(budget); // 500.0

// Shape dynamics over time
let curve = DynamicCurve::sforzando(DynamicMark::Pp, DynamicMark::Fff, 3, 7);
let profile = curve.evaluate();
// Most values are quiet, with a sudden spike at position 3

// Balance multiple agents
let mut bal = DynamicBalance::new(100.0);
bal.set_agent("agent-a", DynamicMark::F);
bal.set_agent("agent-b", DynamicMark::P);
bal.set_agent("agent-c", DynamicMark::Mf);
for (name, amount) in bal.allocate() {
    println!("{name}: {amount:.1}");
}

// Interpret dynamically
let interp = DynamicInterpreter::new(budget);
let resources = interp.interpret_curve(&curve);
```

## Design Philosophy

Musical dynamics are a surprisingly good model for resource allocation because they're:

1. **Relative, not absolute**: Forte doesn't mean "100 units" — it means "loud relative
   to the current context." This makes dynamics portable across different scales.

2. **Expressive**: The same agent can whisper (pp) or shout (fff) depending on the
   situation, creating natural variation rather than flat resource consumption.

3. **Temporal**: Crescendos and diminuendos model real-world patterns where resource
   needs ramp up and down gradually rather than switching abruptly.

4. **Contextual**: The same dynamic mark can be interpreted differently based on what
   came before and what comes next — just like in real music.

## Testing

```bash
cargo test
```

All 34 tests pass, covering dynamic intensity mapping, crescendo/diminuendo/swell curves,
sforzando accents, multi-agent balancing, context interpretation, and the full
DynamicInterpreter pipeline.

## License

MIT
