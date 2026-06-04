
/// Metrics snapshot for a population at a single generation.
#[derive(Debug, Clone)]
pub struct PopulationMetrics {
    /// Average fitness across all agents.
    pub fitness: f64,
    /// Diversity index (0 = monoculture, 1 = uniform distribution).
    pub diversity: f64,
    /// Fraction of agents using the Avoid strategy.
    pub avoidance_ratio: f64,
    /// Total population size.
    pub population_size: usize,
    /// Counts per strategy.
    pub strategy_counts: [usize; 3],
}

impl PopulationMetrics {
    /// Compute metrics from raw strategy counts.
    pub fn from_counts(avoid: usize, cooperate: usize, exploit: usize) -> Self {
        let total = avoid + cooperate + exploit;
        if total == 0 {
            return PopulationMetrics {
                fitness: 0.0,
                diversity: 0.0,
                avoidance_ratio: 0.0,
                population_size: 0,
                strategy_counts: [0, 0, 0],
            };
        }
        let counts = [avoid, cooperate, exploit];
        let avoidance_ratio = avoid as f64 / total as f64;

        // Simpson's diversity index
        let diversity = {
            let mut d = 1.0;
            for &c in &counts {
                let p = c as f64 / total as f64;
                d -= p * p;
            }
            d
        };

        // Fitness heuristic: cooperation bonus, exploit penalty on average
        let fitness = (cooperate as f64 * 1.0 + avoid as f64 * 0.5 + exploit as f64 * 0.3) / total as f64;

        PopulationMetrics {
            fitness,
            diversity,
            avoidance_ratio,
            population_size: total,
            strategy_counts: counts,
        }
    }
}

/// Tracks population metrics over generations.
#[derive(Debug, Clone)]
pub struct TimeSeries {
    /// Generation labels.
    pub generations: Vec<usize>,
    /// Metrics per generation.
    pub metrics: Vec<PopulationMetrics>,
}

impl TimeSeries {
    /// Create an empty time series.
    pub fn new() -> Self {
        TimeSeries {
            generations: Vec::new(),
            metrics: Vec::new(),
        }
    }

    /// Record metrics for a new generation.
    pub fn record(&mut self, generation: usize, m: PopulationMetrics) {
        self.generations.push(generation);
        self.metrics.push(m);
    }

    /// Number of recorded generations.
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// Whether any generations have been recorded.
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    /// Extract the fitness time series as a simple Vec<f64>.
    pub fn fitness_series(&self) -> Vec<f64> {
        self.metrics.iter().map(|m| m.fitness).collect()
    }

    /// Extract the diversity time series.
    pub fn diversity_series(&self) -> Vec<f64> {
        self.metrics.iter().map(|m| m.diversity).collect()
    }

    /// Extract the avoidance ratio time series.
    pub fn avoidance_series(&self) -> Vec<f64> {
        self.metrics.iter().map(|m| m.avoidance_ratio).collect()
    }

    /// Compute the moving average of fitness over a window.
    pub fn fitness_moving_avg(&self, window: usize) -> Vec<f64> {
        moving_average(&self.fitness_series(), window)
    }

    /// Compute the moving average of diversity over a window.
    pub fn diversity_moving_avg(&self, window: usize) -> Vec<f64> {
        moving_average(&self.diversity_series(), window)
    }

    /// Get the metrics for a specific generation, if recorded.
    pub fn get(&self, generation: usize) -> Option<&PopulationMetrics> {
        self.generations.iter().position(|&g| g == generation).and_then(|i| self.metrics.get(i))
    }

    /// Return the last recorded metrics.
    pub fn last(&self) -> Option<&PopulationMetrics> {
        self.metrics.last()
    }

    /// Compute the rate of change (first derivative) of fitness.
    pub fn fitness_derivative(&self) -> Vec<f64> {
        let series = self.fitness_series();
        series.windows(2).map(|w| w[1] - w[0]).collect()
    }
}

/// Simple moving average helper.
fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    if window == 0 || data.len() < window {
        return Vec::new();
    }
    data.windows(window)
        .map(|w| w.iter().sum::<f64>() / window as f64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_from_counts_uniform() {
        let m = PopulationMetrics::from_counts(33, 33, 34);
        assert!((m.diversity - 0.6666).abs() < 0.01);
        assert!((m.avoidance_ratio - 0.33).abs() < 0.01);
        assert_eq!(m.population_size, 100);
    }

    #[test]
    fn test_metrics_from_counts_monoculture() {
        let m = PopulationMetrics::from_counts(100, 0, 0);
        assert!(m.diversity.abs() < 0.001);
        assert!((m.avoidance_ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_metrics_zero_population() {
        let m = PopulationMetrics::from_counts(0, 0, 0);
        assert_eq!(m.population_size, 0);
        assert_eq!(m.fitness, 0.0);
    }

    #[test]
    fn test_timeseries_record_and_get() {
        let mut ts = TimeSeries::new();
        ts.record(0, PopulationMetrics::from_counts(10, 10, 10));
        ts.record(1, PopulationMetrics::from_counts(5, 20, 5));
        assert_eq!(ts.len(), 2);
        assert!(ts.get(0).is_some());
        assert!(ts.get(1).is_some());
        assert!(ts.get(99).is_none());
    }

    #[test]
    fn test_timeseries_series_extraction() {
        let mut ts = TimeSeries::new();
        ts.record(0, PopulationMetrics::from_counts(10, 10, 10));
        ts.record(1, PopulationMetrics::from_counts(5, 20, 5));
        let fitness = ts.fitness_series();
        assert_eq!(fitness.len(), 2);
        // Cooperation increases → fitness increases
        assert!(fitness[1] > fitness[0]);
    }

    #[test]
    fn test_moving_average() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = moving_average(&data, 3);
        assert_eq!(avg, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_fitness_derivative() {
        let mut ts = TimeSeries::new();
        ts.record(0, PopulationMetrics::from_counts(10, 10, 10));
        ts.record(1, PopulationMetrics::from_counts(5, 20, 5));
        ts.record(2, PopulationMetrics::from_counts(5, 20, 5));
        let deriv = ts.fitness_derivative();
        assert!(deriv[0] > 0.0); // fitness increased
        assert!(deriv[1].abs() < 0.001); // fitness plateaued
    }
}
