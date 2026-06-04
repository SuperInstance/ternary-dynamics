/// A detected phase transition event.
#[derive(Debug, Clone)]
pub struct PhaseEvent {
    /// Generation where the transition was detected.
    pub generation: usize,
    /// Magnitude of the change (absolute difference in metric).
    pub magnitude: f64,
    /// Description of what changed.
    pub label: String,
}

impl PhaseEvent {
    /// Create a new phase event.
    pub fn new(generation: usize, magnitude: f64, label: impl Into<String>) -> Self {
        PhaseEvent {
            generation,
            magnitude,
            label: label.into(),
        }
    }
}

/// Detects phase transitions in time series data.
#[derive(Debug, Clone)]
pub struct PhaseDetector {
    /// Threshold for detecting a significant change.
    pub threshold: f64,
    /// Minimum number of generations to look back for trend comparison.
    pub window: usize,
}

impl PhaseDetector {
    /// Create a detector with given threshold and window size.
    pub fn new(threshold: f64, window: usize) -> Self {
        PhaseDetector { threshold, window }
    }

    /// Create a detector with sensible defaults.
    pub fn default() -> Self {
        PhaseDetector {
            threshold: 0.1,
            window: 3,
        }
    }

    /// Detect phase transitions in a metric series.
    /// Returns events where the absolute change exceeds the threshold.
    pub fn detect(&self, generations: &[usize], values: &[f64]) -> Vec<PhaseEvent> {
        if values.len() < 2 {
            return Vec::new();
        }
        let mut events = Vec::new();
        for i in 1..values.len() {
            let change = (values[i] - values[i - 1]).abs();
            if change > self.threshold {
                events.push(PhaseEvent::new(
                    generations[i],
                    change,
                    format!("phase shift at generation {}", generations[i]),
                ));
            }
        }
        events
    }

    /// Detect species collapse: sudden drop in population diversity.
    pub fn detect_collapse(&self, generations: &[usize], diversity: &[f64]) -> Vec<PhaseEvent> {
        self.detect_with_label(generations, diversity, "diversity collapse")
    }

    /// Detect cascade onset: rapid shift in avoidance ratio.
    pub fn detect_cascade(&self, generations: &[usize], avoidance: &[f64]) -> Vec<PhaseEvent> {
        self.detect_with_label(generations, avoidance, "cascade onset")
    }

    /// Detect using a rolling window average comparison.
    pub fn detect_rolling(&self, generations: &[usize], values: &[f64]) -> Vec<PhaseEvent> {
        if values.len() < self.window * 2 {
            return Vec::new();
        }
        let mut events = Vec::new();
        for i in (self.window..values.len()).rev() {
            let start = i.saturating_sub(self.window);
            if start < self.window {
                continue;
            }
            let prev_avg: f64 = values[start - self.window..start].iter().sum::<f64>() / self.window as f64;
            let curr_avg: f64 = values[start..i].iter().sum::<f64>() / (i - start) as f64;
            let change = (curr_avg - prev_avg).abs();
            if change > self.threshold {
                events.push(PhaseEvent::new(
                    generations[i],
                    change,
                    format!("rolling phase shift at generation {}", generations[i]),
                ));
            }
        }
        events.sort_by_key(|e| e.generation);
        events.dedup_by_key(|e| e.generation);
        events
    }

    fn detect_with_label(&self, generations: &[usize], values: &[f64], label: &str) -> Vec<PhaseEvent> {
        if values.len() < 2 {
            return Vec::new();
        }
        let mut events = Vec::new();
        for i in 1..values.len() {
            let change = values[i] - values[i - 1];
            if change.abs() > self.threshold {
                events.push(PhaseEvent::new(
                    generations[i],
                    change.abs(),
                    label.to_string(),
                ));
            }
        }
        events
    }
}

/// Convenience struct for a full phase transition analysis.
#[derive(Debug, Clone)]
pub struct PhaseTransition {
    /// All detected events.
    pub events: Vec<PhaseEvent>,
    /// The detector used.
    pub detector: PhaseDetector,
}

impl PhaseTransition {
    /// Run a full analysis on the given generations and metric series.
    pub fn analyze(generations: &[usize], values: &[f64], threshold: f64) -> Self {
        let detector = PhaseDetector::new(threshold, 3);
        let events = detector.detect(generations, values);
        PhaseTransition { events, detector }
    }

    /// Number of transitions detected.
    pub fn count(&self) -> usize {
        self.events.len()
    }

    /// The largest transition by magnitude.
    pub fn largest(&self) -> Option<&PhaseEvent> {
        self.events.iter().max_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// The first transition detected.
    pub fn first(&self) -> Option<&PhaseEvent> {
        self.events.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_basic_transition() {
        let detector = PhaseDetector::new(0.1, 3);
        let gens = vec![0, 1, 2, 3, 4];
        let vals = vec![0.5, 0.5, 0.5, 0.9, 0.9];
        let events = detector.detect(&gens, &vals);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].generation, 3);
    }

    #[test]
    fn test_no_transition_below_threshold() {
        let detector = PhaseDetector::new(0.5, 3);
        let gens = vec![0, 1, 2];
        let vals = vec![0.5, 0.55, 0.6];
        let events = detector.detect(&gens, &vals);
        assert!(events.is_empty());
    }

    #[test]
    fn test_collapse_detection() {
        let detector = PhaseDetector::new(0.1, 3);
        let gens = vec![0, 1, 2, 3, 4];
        let div = vec![0.6, 0.6, 0.6, 0.1, 0.1];
        let events = detector.detect_collapse(&gens, &div);
        assert_eq!(events.len(), 1);
        assert!(events[0].label.contains("collapse"));
    }

    #[test]
    fn test_cascade_detection() {
        let detector = PhaseDetector::new(0.1, 3);
        let gens = vec![0, 1, 2, 3];
        let avoid = vec![0.1, 0.1, 0.8, 0.9];
        let events = detector.detect_cascade(&gens, &avoid);
        assert_eq!(events.len(), 1);
        assert!(events[0].label.contains("cascade"));
    }

    #[test]
    fn test_phase_transition_analyze() {
        let gens = vec![0, 1, 2, 3, 4, 5];
        let vals = vec![0.5, 0.5, 0.5, 0.9, 0.9, 0.9];
        let pt = PhaseTransition::analyze(&gens, &vals, 0.1);
        assert_eq!(pt.count(), 1);
        assert!(pt.largest().is_some());
        assert!(pt.first().is_some());
    }
}
