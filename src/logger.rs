use crate::TernaryStrategy;

/// A snapshot of the full system state at one generation.
#[derive(Debug, Clone)]
pub struct GenerationSnapshot {
    /// Generation index.
    pub generation: usize,
    /// Strategy counts: [avoid, cooperate, exploit].
    pub strategy_counts: [usize; 3],
    /// Average fitness.
    pub avg_fitness: f64,
    /// Diversity index.
    pub diversity: f64,
    /// Optional tag/label for this generation (e.g., "mutation event").
    pub tag: Option<String>,
}

impl GenerationSnapshot {
    /// Create a new snapshot from raw data.
    pub fn new(
        generation: usize,
        strategy_counts: [usize; 3],
        avg_fitness: f64,
        diversity: f64,
    ) -> Self {
        GenerationSnapshot {
            generation,
            strategy_counts,
            avg_fitness,
            diversity,
            tag: None,
        }
    }

    /// Attach a tag to this snapshot.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Total population.
    pub fn total_population(&self) -> usize {
        self.strategy_counts.iter().sum()
    }

    /// Fraction of each strategy.
    pub fn strategy_fractions(&self) -> [f64; 3] {
        let total = self.total_population() as f64;
        if total == 0.0 {
            return [0.0; 3];
        }
        [
            self.strategy_counts[0] as f64 / total,
            self.strategy_counts[1] as f64 / total,
            self.strategy_counts[2] as f64 / total,
        ]
    }
}

/// Records detailed per-generation state for analysis and replay.
#[derive(Debug, Clone)]
pub struct GenerationLogger {
    /// All recorded snapshots, in order.
    pub snapshots: Vec<GenerationSnapshot>,
}

impl GenerationLogger {
    /// Create a new empty logger.
    pub fn new() -> Self {
        GenerationLogger {
            snapshots: Vec::new(),
        }
    }

    /// Log a generation snapshot.
    pub fn log(&mut self, snapshot: GenerationSnapshot) {
        self.snapshots.push(snapshot);
    }

    /// Number of logged generations.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Whether any generations have been logged.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Get a snapshot by generation number.
    pub fn get(&self, generation: usize) -> Option<&GenerationSnapshot> {
        self.snapshots.iter().find(|s| s.generation == generation)
    }

    /// Get the last snapshot.
    pub fn last(&self) -> Option<&GenerationSnapshot> {
        self.snapshots.last()
    }

    /// Find all snapshots with a given tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&GenerationSnapshot> {
        self.snapshots.iter().filter(|s| s.tag.as_deref() == Some(tag)).collect()
    }

    /// Extract generations where a specific strategy dominates (>50%).
    pub fn dominated_by(&self, strategy: TernaryStrategy) -> Vec<&GenerationSnapshot> {
        let idx = match strategy {
            TernaryStrategy::Avoid => 0,
            TernaryStrategy::Cooperate => 1,
            TernaryStrategy::Exploit => 2,
        };
        self.snapshots.iter().filter(|s| {
            let fracs = s.strategy_fractions();
            fracs[idx] > 0.5
        }).collect()
    }

    /// Return a summary: (start_gen, end_gen, total_generations).
    pub fn summary(&self) -> (usize, usize, usize) {
        if self.snapshots.is_empty() {
            return (0, 0, 0);
        }
        let start = self.snapshots.first().unwrap().generation;
        let end = self.snapshots.last().unwrap().generation;
        (start, end, self.snapshots.len())
    }

    /// Export strategy counts as parallel arrays: (avoid[], cooperate[], exploit[]).
    pub fn export_counts(&self) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        let mut a = Vec::with_capacity(self.snapshots.len());
        let mut c = Vec::with_capacity(self.snapshots.len());
        let mut e = Vec::with_capacity(self.snapshots.len());
        for s in &self.snapshots {
            a.push(s.strategy_counts[0]);
            c.push(s.strategy_counts[1]);
            e.push(s.strategy_counts[2]);
        }
        (a, c, e)
    }
}

impl Default for GenerationLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snap = GenerationSnapshot::new(0, [10, 20, 30], 0.7, 0.6);
        assert_eq!(snap.total_population(), 60);
        assert!(snap.tag.is_none());
    }

    #[test]
    fn test_snapshot_with_tag() {
        let snap = GenerationSnapshot::new(5, [10, 10, 10], 0.5, 0.5)
            .with_tag("mutation");
        assert_eq!(snap.tag.as_deref(), Some("mutation"));
    }

    #[test]
    fn test_snapshot_fractions() {
        let snap = GenerationSnapshot::new(0, [0, 100, 0], 1.0, 0.0);
        let fracs = snap.strategy_fractions();
        assert!((fracs[1] - 1.0).abs() < 0.001);
        assert!((fracs[0]).abs() < 0.001);
    }

    #[test]
    fn test_logger_record_and_retrieve() {
        let mut logger = GenerationLogger::new();
        logger.log(GenerationSnapshot::new(0, [10, 10, 10], 0.5, 0.6));
        logger.log(GenerationSnapshot::new(1, [5, 20, 5], 0.7, 0.4)
            .with_tag("shift"));
        assert_eq!(logger.len(), 2);
        assert!(logger.get(0).is_some());
        assert!(logger.get(1).is_some());
        assert!(logger.get(99).is_none());
    }

    #[test]
    fn test_logger_find_by_tag() {
        let mut logger = GenerationLogger::new();
        logger.log(GenerationSnapshot::new(0, [10, 10, 10], 0.5, 0.6));
        logger.log(GenerationSnapshot::new(1, [5, 20, 5], 0.7, 0.4).with_tag("event"));
        let tagged = logger.find_by_tag("event");
        assert_eq!(tagged.len(), 1);
        assert_eq!(tagged[0].generation, 1);
    }

    #[test]
    fn test_logger_dominated_by() {
        let mut logger = GenerationLogger::new();
        logger.log(GenerationSnapshot::new(0, [10, 60, 30], 0.7, 0.5)); // cooperate dominates
        logger.log(GenerationSnapshot::new(1, [70, 20, 10], 0.5, 0.4)); // avoid dominates
        logger.log(GenerationSnapshot::new(2, [30, 30, 40], 0.5, 0.6)); // no one dominates
        let coop = logger.dominated_by(TernaryStrategy::Cooperate);
        assert_eq!(coop.len(), 1);
        assert_eq!(coop[0].generation, 0);
    }

    #[test]
    fn test_logger_export_counts() {
        let mut logger = GenerationLogger::new();
        logger.log(GenerationSnapshot::new(0, [10, 20, 30], 0.5, 0.6));
        let (a, c, e) = logger.export_counts();
        assert_eq!(a, vec![10]);
        assert_eq!(c, vec![20]);
        assert_eq!(e, vec![30]);
    }
}
