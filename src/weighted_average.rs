/// Computes the weighted arithmetic mean
/// https://en.wikipedia.org/wiki/Weighted_arithmetic_mean
pub struct WeightedMean {
    value: f64,
    total_weight: f64,
}

impl WeightedMean {
    pub fn new(total_weight: f64) -> Self {
        WeightedMean {
            value: 0.0,
            total_weight,
        }
    }

    pub fn add(&mut self, value: f64, weight: f64) {
        debug_assert!(weight >= 0.0);
        if self.total_weight > 0.0 {
            self.value += value * weight / self.total_weight
        }
    }

    pub fn compute(self) -> f64 {
        self.value
    }
}

pub struct WeightedBool {
    value: f64,
    total_weight: f64,
}

impl WeightedBool {
    pub fn new(total_weight: f64) -> Self {
        WeightedBool {
            value: 0.0,
            total_weight,
        }
    }

    pub fn add(&mut self, value: bool, weight: f64) {
        debug_assert!(weight >= 0.0);
        if self.total_weight > 0.0 {
            self.value += value as i32 as f64 * weight / self.total_weight
        }
    }

    pub fn compute(self) -> bool {
        self.value >= 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_empty_set() {
        let mean = WeightedMean::new(0.0);
        assert_eq!(mean.compute(), 0.0);
    }

    #[test]
    fn mean_single_value() {
        let mut mean = WeightedMean::new(1.0);
        mean.add(42.0, 1.0);
        assert_eq!(mean.compute(), 42.0);
    }

    #[test]
    fn mean_multiple_values() {
        let mut mean = WeightedMean::new(2.0);
        mean.add(0.0, 1.0);
        mean.add(4.0, 0.5);
        mean.add(8.0, 0.5);
        assert_eq!(mean.compute(), 3.0);
    }

    #[test]
    fn bool_empty_set() {
        let bool = WeightedBool::new(0.0);
        assert_eq!(bool.compute(), false);
    }

    #[test]
    fn bool_true() {
        let mut bool = WeightedBool::new(1.0);
        bool.add(true, 1.0);
        assert_eq!(bool.compute(), true);
    }

    #[test]
    fn bool_false() {
        let mut bool = WeightedBool::new(1.0);
        bool.add(false, 1.0);
        assert_eq!(bool.compute(), false);
    }

    #[test]
    fn bool_true_false() {
        let mut bool = WeightedBool::new(1.0);
        bool.add(true, 0.5);
        bool.add(false, 0.5);
        assert_eq!(bool.compute(), true);
    }

    #[test]
    fn bool_false_false() {
        let mut bool = WeightedBool::new(1.0);
        bool.add(false, 0.5);
        bool.add(false, 0.5);
        assert_eq!(bool.compute(), false);
    }

    #[test]
    fn bool_true_true() {
        let mut bool = WeightedBool::new(1.0);
        bool.add(true, 0.5);
        bool.add(true, 0.5);
        assert_eq!(bool.compute(), true);
    }
}
