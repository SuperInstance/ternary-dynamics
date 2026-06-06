# PLUG_AND_PLAY — Dynamics

> Temporal dynamics of ternary agent systems

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-dynamics = { git = "https://github.com/SuperInstance/ternary-dynamics" }
```

Use in your code:

```rust
use ternary_dynamics::{System, TernaryStrategy};

let mut sys = System::new(100);
sys.run(1000);
let phase = sys.detect_phase();
```

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
