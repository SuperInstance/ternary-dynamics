use crate::TernaryStrategy;

/// A strategy at a point in time — used to build a trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strategy {
    /// Which ternary strategy.
    pub strategy: TernaryStrategy,
    /// The generation this strategy was adopted.
    pub generation: usize,
}

impl Strategy {
    /// Create a new strategy entry.
    pub fn new(strategy: TernaryStrategy, generation: usize) -> Self {
        Strategy { strategy, generation }
    }
}

/// Tracks a single agent's strategy changes over time.
#[derive(Debug, Clone)]
pub struct Trajectory {
    /// Unique agent identifier.
    pub agent_id: usize,
    /// Strategy history in chronological order.
    pub history: Vec<Strategy>,
}

impl Trajectory {
    /// Create a new trajectory for an agent.
    pub fn new(agent_id: usize) -> Self {
        Trajectory {
            agent_id,
            history: Vec::new(),
        }
    }

    /// Record a strategy adoption at a generation.
    /// If the agent already has a strategy at this generation, it is replaced.
    pub fn adopt(&mut self, strategy: TernaryStrategy, generation: usize) {
        // Check if we already have an entry for this generation
        if let Some(entry) = self.history.iter_mut().find(|s| s.generation == generation) {
            entry.strategy = strategy;
        } else {
            self.history.push(Strategy::new(strategy, generation));
            self.history.sort_by_key(|s| s.generation);
        }
    }

    /// Get the strategy at a specific generation.
    /// Returns the most recently adopted strategy at or before this generation.
    pub fn strategy_at(&self, generation: usize) -> Option<TernaryStrategy> {
        self.history.iter()
            .rev()
            .find(|s| s.generation <= generation)
            .map(|s| s.strategy)
    }

    /// Number of strategy changes (transitions) in the trajectory.
    pub fn transition_count(&self) -> usize {
        if self.history.len() <= 1 {
            return 0;
        }
        self.history.windows(2)
            .filter(|w| w[0].strategy != w[1].strategy)
            .count()
    }

    /// Total number of recorded strategy entries.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Whether the trajectory is empty.
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// The first strategy adopted.
    pub fn first_strategy(&self) -> Option<TernaryStrategy> {
        self.history.first().map(|s| s.strategy)
    }

    /// The most recent strategy adopted.
    pub fn current_strategy(&self) -> Option<TernaryStrategy> {
        self.history.last().map(|s| s.strategy)
    }

    /// The generation at which this agent first appeared.
    pub fn first_generation(&self) -> Option<usize> {
        self.history.first().map(|s| s.generation)
    }

    /// Count how many generations the agent used each strategy.
    pub fn strategy_durations(&self) -> [usize; 3] {
        let mut durations = [0usize; 3];
        for w in self.history.windows(2) {
            let idx = match w[0].strategy {
                TernaryStrategy::Avoid => 0,
                TernaryStrategy::Cooperate => 1,
                TernaryStrategy::Exploit => 2,
            };
            durations[idx] += w[1].generation - w[0].generation;
        }
        durations
    }

    /// Check if the agent is "loyal" — never changed strategy.
    pub fn is_loyal(&self) -> bool {
        self.transition_count() == 0 && self.history.len() > 0
    }

    /// Check if the agent oscillates — switches back to a previous strategy.
    pub fn is_oscillator(&self) -> bool {
        let strategies: Vec<TernaryStrategy> = self.history.iter().map(|s| s.strategy).collect();
        for i in 0..strategies.len() {
            for j in (i + 2)..strategies.len() {
                if strategies[i] == strategies[j] {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trajectory_creation() {
        let t = Trajectory::new(42);
        assert_eq!(t.agent_id, 42);
        assert!(t.is_empty());
    }

    #[test]
    fn test_adopt_and_retrieve() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Exploit, 5);
        t.adopt(TernaryStrategy::Avoid, 10);
        assert_eq!(t.strategy_at(0), Some(TernaryStrategy::Cooperate));
        assert_eq!(t.strategy_at(3), Some(TernaryStrategy::Cooperate));
        assert_eq!(t.strategy_at(5), Some(TernaryStrategy::Exploit));
        assert_eq!(t.strategy_at(9), Some(TernaryStrategy::Exploit));
        assert_eq!(t.strategy_at(10), Some(TernaryStrategy::Avoid));
        assert_eq!(t.strategy_at(100), Some(TernaryStrategy::Avoid));
    }

    #[test]
    fn test_strategy_before_any_adoption() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 5);
        assert_eq!(t.strategy_at(3), None); // before first adoption
    }

    #[test]
    fn test_transition_count() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Exploit, 5);
        t.adopt(TernaryStrategy::Avoid, 10);
        assert_eq!(t.transition_count(), 2);
    }

    #[test]
    fn test_no_transitions_when_same_strategy() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Cooperate, 5);
        assert_eq!(t.transition_count(), 0);
    }

    #[test]
    fn test_is_loyal() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        assert!(t.is_loyal());

        t.adopt(TernaryStrategy::Exploit, 5);
        assert!(!t.is_loyal());
    }

    #[test]
    fn test_is_oscillator() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Exploit, 5);
        t.adopt(TernaryStrategy::Cooperate, 10); // switches back
        assert!(t.is_oscillator());
    }

    #[test]
    fn test_not_oscillator() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Exploit, 5);
        t.adopt(TernaryStrategy::Avoid, 10);
        assert!(!t.is_oscillator());
    }

    #[test]
    fn test_strategy_durations() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 0);
        t.adopt(TernaryStrategy::Avoid, 10);
        t.adopt(TernaryStrategy::Exploit, 15);
        let dur = t.strategy_durations();
        // Cooperate from 0 to 10 = 10, Avoid from 10 to 15 = 5
        assert_eq!(dur[0], 5);  // Avoid: 15-10
        assert_eq!(dur[1], 10); // Cooperate: 10-0
    }

    #[test]
    fn test_replace_at_same_generation() {
        let mut t = Trajectory::new(1);
        t.adopt(TernaryStrategy::Cooperate, 5);
        t.adopt(TernaryStrategy::Avoid, 5); // replaces
        assert_eq!(t.len(), 1);
        assert_eq!(t.strategy_at(5), Some(TernaryStrategy::Avoid));
    }
}
