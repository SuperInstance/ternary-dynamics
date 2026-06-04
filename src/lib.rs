//! # Ternary Dynamics
//!
//! Temporal dynamics of ternary agent systems — how strategies evolve over time,
//! phase transitions, and critical points in populations governed by ternary
//! (avoid, cooperate, exploit) logic.

mod timeseries;
mod phase;
mod critical;
mod logger;
mod dynamic_mode;
mod trajectory;

pub use timeseries::{TimeSeries, PopulationMetrics};
pub use phase::{PhaseTransition, PhaseEvent, PhaseDetector};
pub use critical::{CriticalPoint, CriticalDetector};
pub use logger::{GenerationLogger, GenerationSnapshot};
pub use dynamic_mode::{DynamicMode, ModeClassifier};
pub use trajectory::{Trajectory, Strategy};

/// The three fundamental strategies in a ternary agent system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryStrategy {
    /// Avoid — retreat from interaction, minimize risk.
    Avoid,
    /// Cooperate — seek mutual benefit.
    Cooperate,
    /// Exploit — take advantage of others.
    Exploit,
}

impl TernaryStrategy {
    /// Returns all three variants in order.
    pub fn all() -> [TernaryStrategy; 3] {
        [TernaryStrategy::Avoid, TernaryStrategy::Cooperate, TernaryStrategy::Exploit]
    }
}
