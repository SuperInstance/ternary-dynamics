/// Classification of system dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DynamicMode {
    /// Population is converging to a fixed point.
    Converging,
    /// Population is oscillating between states.
    Oscillating,
    /// Population behavior is chaotic / unpredictable.
    Chaotic,
    /// Population is stable (little change).
    Stable,
}

impl std::fmt::Display for DynamicMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicMode::Converging => write!(f, "converging"),
            DynamicMode::Oscillating => write!(f, "oscillating"),
            DynamicMode::Chaotic => write!(f, "chaotic"),
            DynamicMode::Stable => write!(f, "stable"),
        }
    }
}

/// Classifies the dynamic mode of a time series.
#[derive(Debug, Clone)]
pub struct ModeClassifier {
    /// Threshold for "stable" (max variance).
    pub stable_variance: f64,
    /// Minimum number of sign changes to be "oscillating".
    pub oscillation_threshold: usize,
    /// Threshold for "converging" (consistent trend).
    pub convergence_slope: f64,
}

impl ModeClassifier {
    /// Create with default parameters.
    pub fn new() -> Self {
        ModeClassifier {
            stable_variance: 0.001,
            oscillation_threshold: 3,
            convergence_slope: 0.01,
        }
    }

    /// Classify the dynamic mode of a metric time series.
    pub fn classify(&self, values: &[f64]) -> DynamicMode {
        if values.len() < 3 {
            return DynamicMode::Stable;
        }

        // Check stability: low variance
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        if variance < self.stable_variance {
            return DynamicMode::Stable;
        }

        // Compute derivatives
        let derivs: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();

        // Count sign changes in derivatives → oscillation
        let sign_changes = derivs.windows(2)
            .filter(|w| w[0].signum() != w[1].signum() && w[0] != 0.0 && w[1] != 0.0)
            .count();

        if sign_changes >= self.oscillation_threshold {
            return DynamicMode::Oscillating;
        }

        // Check for consistent trend (converging)
        let avg_slope = derivs.iter().sum::<f64>() / derivs.len() as f64;
        if avg_slope.abs() > self.convergence_slope {
            return DynamicMode::Converging;
        }

        // Check for chaos: high variance + no clear pattern
        if variance > self.stable_variance * 10.0 {
            return DynamicMode::Chaotic;
        }

        DynamicMode::Stable
    }

    /// Classify with a richer feature set: uses both fitness and diversity.
    pub fn classify_combined(&self, fitness: &[f64], diversity: &[f64]) -> (DynamicMode, DynamicMode) {
        (self.classify(fitness), self.classify(diversity))
    }

    /// Compute the overall system mode from fitness, diversity, and avoidance.
    /// Uses majority vote among the three signals.
    pub fn classify_system(
        &self,
        fitness: &[f64],
        diversity: &[f64],
        avoidance: &[f64],
    ) -> DynamicMode {
        let modes = [
            self.classify(fitness),
            self.classify(diversity),
            self.classify(avoidance),
        ];

        // Majority vote
        let mut best_mode = DynamicMode::Stable;
        let mut best_count = 0;
        for &mode in &modes {
            let count = modes.iter().filter(|&&m| m == mode).count();
            if count > best_count {
                best_count = count;
                best_mode = mode;
            }
        }
        best_mode
    }
}

impl Default for ModeClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_series() {
        let classifier = ModeClassifier::new();
        let vals = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        assert_eq!(classifier.classify(&vals), DynamicMode::Stable);
    }

    #[test]
    fn test_converging_series() {
        let classifier = ModeClassifier::new();
        let vals = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        assert_eq!(classifier.classify(&vals), DynamicMode::Converging);
    }

    #[test]
    fn test_oscillating_series() {
        let classifier = ModeClassifier::new();
        let vals = vec![0.1, 0.9, 0.1, 0.9, 0.1, 0.9, 0.1, 0.9];
        assert_eq!(classifier.classify(&vals), DynamicMode::Oscillating);
    }

    #[test]
    fn test_short_series_is_stable() {
        let classifier = ModeClassifier::new();
        let vals = vec![0.5];
        assert_eq!(classifier.classify(&vals), DynamicMode::Stable);
    }

    #[test]
    fn test_combined_classification() {
        let classifier = ModeClassifier::new();
        let fitness = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let diversity = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let (fmode, dmode) = classifier.classify_combined(&fitness, &diversity);
        assert_eq!(fmode, DynamicMode::Converging);
        assert_eq!(dmode, DynamicMode::Stable);
    }

    #[test]
    fn test_system_classification() {
        let classifier = ModeClassifier::new();
        let fitness = vec![0.5, 0.5, 0.5, 0.5, 0.5];
        let diversity = vec![0.6, 0.6, 0.6, 0.6, 0.6];
        let avoidance = vec![0.1, 0.1, 0.1, 0.1, 0.1];
        assert_eq!(
            classifier.classify_system(&fitness, &diversity, &avoidance),
            DynamicMode::Stable
        );
    }
}
