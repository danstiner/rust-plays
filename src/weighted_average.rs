/// Computes the weighted arithmetic mean
/// https://en.wikipedia.org/wiki/Weighted_arithmetic_mean
pub struct WeightedAverage {
    value: f64,
    total_weight: f64,
}

impl WeightedAverage {
    pub fn new(total_weight: f64) -> Self {
        WeightedAverage {
            value: 0.0,
            total_weight,
        }
    }

    pub fn add(&mut self, value: f64, weight: f64) {
        debug_assert!(weight >= 0.0);
        self.value += value * weight
    }

    pub fn average(self) -> f64 {
        if self.total_weight == 0.0 {
            0.0
        } else {
            self.value / self.total_weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_() {
        // TODO
    }
}
