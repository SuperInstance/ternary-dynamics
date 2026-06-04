/// A detected critical point in the time series.
#[derive(Debug, Clone)]
pub struct CriticalPoint {
    /// Generation at which the critical point occurs.
    pub generation: usize,
    /// The metric value at this point.
    pub value: f64,
    /// Kind of critical point.
    pub kind: CriticalKind,
}

/// Classification of critical point types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CriticalKind {
    /// Fitness plateau: fitness stops increasing.
    FitnessPlateau,
    /// Diversity cliff: sharp drop in diversity.
    DiversityCliff,
    /// Diversity floor: diversity hits near-zero.
    DiversityFloor,
    /// Avoidance spike: sudden increase in avoidance.
    AvoidanceSpike,
    /// Population crash: significant population loss.
    PopulationCrash,
}

/// Detects critical points in population time series.
#[derive(Debug, Clone)]
pub struct CriticalDetector {
    /// How many consecutive flat generations constitute a plateau.
    pub plateau_window: usize,
    /// Threshold for "flat" (max change per step).
    pub plateau_tolerance: f64,
    /// Drop threshold for a diversity cliff.
    pub cliff_threshold: f64,
    /// Value below which diversity is considered a floor.
    pub diversity_floor: f64,
    /// Threshold for an avoidance spike (rate of change).
    pub avoidance_spike_threshold: f64,
    /// Threshold for population crash (fraction lost in one step).
    pub crash_threshold: f64,
}

impl CriticalDetector {
    /// Default detector with reasonable thresholds.
    pub fn new() -> Self {
        CriticalDetector {
            plateau_window: 5,
            plateau_tolerance: 0.01,
            cliff_threshold: 0.15,
            diversity_floor: 0.05,
            avoidance_spike_threshold: 0.2,
            crash_threshold: 0.3,
        }
    }

    /// Detect fitness plateaus. Returns the first generation of each plateau.
    pub fn detect_fitness_plateaus(&self, generations: &[usize], fitness: &[f64]) -> Vec<CriticalPoint> {
        if fitness.len() < self.plateau_window {
            return Vec::new();
        }
        let mut points = Vec::new();
        let mut i = 0;
        while i + self.plateau_window <= fitness.len() {
            let window = &fitness[i..i + self.plateau_window];
            let max_diff = window.iter().zip(window.iter().skip(1))
                .map(|(a, b)| (b - a).abs())
                .fold(0.0f64, f64::max);
            if max_diff < self.plateau_tolerance {
                points.push(CriticalPoint {
                    generation: generations[i],
                    value: fitness[i],
                    kind: CriticalKind::FitnessPlateau,
                });
                i += self.plateau_window; // skip past plateau
            } else {
                i += 1;
            }
        }
        points
    }

    /// Detect diversity cliffs: single-generation drops exceeding the threshold.
    pub fn detect_diversity_cliffs(&self, generations: &[usize], diversity: &[f64]) -> Vec<CriticalPoint> {
        let mut points = Vec::new();
        for i in 1..diversity.len() {
            let drop = diversity[i - 1] - diversity[i];
            if drop > self.cliff_threshold {
                points.push(CriticalPoint {
                    generation: generations[i],
                    value: diversity[i],
                    kind: CriticalKind::DiversityCliff,
                });
            }
        }
        points
    }

    /// Detect diversity floors: diversity drops below a minimum threshold.
    pub fn detect_diversity_floors(&self, generations: &[usize], diversity: &[f64]) -> Vec<CriticalPoint> {
        let mut points = Vec::new();
        for i in 0..diversity.len() {
            if diversity[i] < self.diversity_floor {
                points.push(CriticalPoint {
                    generation: generations[i],
                    value: diversity[i],
                    kind: CriticalKind::DiversityFloor,
                });
            }
        }
        points
    }

    /// Detect avoidance spikes: sudden jumps in avoidance ratio.
    pub fn detect_avoidance_spikes(&self, generations: &[usize], avoidance: &[f64]) -> Vec<CriticalPoint> {
        let mut points = Vec::new();
        for i in 1..avoidance.len() {
            let spike = avoidance[i] - avoidance[i - 1];
            if spike > self.avoidance_spike_threshold {
                points.push(CriticalPoint {
                    generation: generations[i],
                    value: avoidance[i],
                    kind: CriticalKind::AvoidanceSpike,
                });
            }
        }
        points
    }

    /// Detect population crashes: significant population loss in one step.
    pub fn detect_population_crashes(&self, generations: &[usize], population: &[usize]) -> Vec<CriticalPoint> {
        let mut points = Vec::new();
        for i in 1..population.len() {
            if population[i - 1] == 0 {
                continue;
            }
            let loss = (population[i - 1] - population[i]) as f64 / population[i - 1] as f64;
            if loss > self.crash_threshold {
                points.push(CriticalPoint {
                    generation: generations[i],
                    value: loss,
                    kind: CriticalKind::PopulationCrash,
                });
            }
        }
        points
    }

    /// Run all detectors and return combined critical points, sorted by generation.
    pub fn detect_all(
        &self,
        generations: &[usize],
        fitness: &[f64],
        diversity: &[f64],
        avoidance: &[f64],
        population: &[usize],
    ) -> Vec<CriticalPoint> {
        let mut points = Vec::new();
        points.extend(self.detect_fitness_plateaus(generations, fitness));
        points.extend(self.detect_diversity_cliffs(generations, diversity));
        points.extend(self.detect_diversity_floors(generations, diversity));
        points.extend(self.detect_avoidance_spikes(generations, avoidance));
        points.extend(self.detect_population_crashes(generations, population));
        points.sort_by_key(|p| p.generation);
        points
    }
}

impl Default for CriticalDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fitness_plateau_detection() {
        let d = CriticalDetector::new();
        let gens: Vec<usize> = (0..10).collect();
        let fitness = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5];
        let plateaus = d.detect_fitness_plateaus(&gens, &fitness);
        assert!(!plateaus.is_empty());
        assert_eq!(plateaus[0].generation, 4);
        assert_eq!(plateaus[0].kind, CriticalKind::FitnessPlateau);
    }

    #[test]
    fn test_no_plateau_if_still_growing() {
        let d = CriticalDetector::new();
        let gens: Vec<usize> = (0..8).collect();
        let fitness = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let plateaus = d.detect_fitness_plateaus(&gens, &fitness);
        assert!(plateaus.is_empty());
    }

    #[test]
    fn test_diversity_cliff() {
        let d = CriticalDetector::new();
        let gens = vec![0, 1, 2, 3, 4];
        let div = vec![0.6, 0.6, 0.6, 0.2, 0.2];
        let cliffs = d.detect_diversity_cliffs(&gens, &div);
        assert_eq!(cliffs.len(), 1);
        assert_eq!(cliffs[0].generation, 3);
    }

    #[test]
    fn test_diversity_floor() {
        let d = CriticalDetector::new();
        let gens = vec![0, 1, 2];
        let div = vec![0.5, 0.3, 0.01];
        let floors = d.detect_diversity_floors(&gens, &div);
        assert_eq!(floors.len(), 1);
        assert_eq!(floors[0].generation, 2);
    }

    #[test]
    fn test_avoidance_spike() {
        let d = CriticalDetector::new();
        let gens = vec![0, 1, 2, 3];
        let avoid = vec![0.1, 0.1, 0.5, 0.6];
        let spikes = d.detect_avoidance_spikes(&gens, &avoid);
        assert_eq!(spikes.len(), 1);
        assert_eq!(spikes[0].generation, 2);
    }

    #[test]
    fn test_population_crash() {
        let d = CriticalDetector::new();
        let gens = vec![0, 1, 2];
        let pop = vec![100, 50, 45];
        let crashes = d.detect_population_crashes(&gens, &pop);
        assert_eq!(crashes.len(), 1);
        assert_eq!(crashes[0].generation, 1);
    }
}
