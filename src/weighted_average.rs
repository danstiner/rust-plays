use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

/// Computes the weighted arithmetic mean
/// https://en.wikipedia.org/wiki/Weighted_arithmetic_mean
pub struct WeightedAverage {
    value: f64,
    total_weight: f64,
}

impl WeightedAverage {
    pub fn new(total_weight: f64) -> Self {
        WeightedAverager {
            value: 0,
            total_weight,
        }
    }

    pub fn add(value: f64, weight: f64) {
        debug_assert!(value >= 0);
        self.value += value * weight
    }

    pub fn average(self) -> f64 {
        if total_weight == 0 {
            0
        } else {
            value / total_weight
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_empty() {
        let mut combiner = InputCombiner::new();
        assert_eq!(combiner.step(), Default::default());
    }

    #[test]
    fn test_step_single_mouse_move() {
        let mut combiner = InputCombiner::new();

        combiner
            .channel("chan")
            .mouse_move_relative(1, -1, true, false);

        assert_eq!(
            combiner.step(),
            Output {
                mouse_delta_x: 1,
                mouse_delta_y: -1,
                ..Default::default()
            }
        );

        assert_eq!(combiner.step(), Default::default());
    }
}
