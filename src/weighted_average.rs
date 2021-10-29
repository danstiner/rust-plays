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

    pub fn average(self) -> f64 {
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

    pub fn average(self) -> bool {
        self.value >= 0.5
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
