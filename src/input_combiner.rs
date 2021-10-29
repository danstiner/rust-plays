use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Output {
    pub mouse_delta_x: i32,
    pub mouse_delta_y: i32,
    pub mouse_left_button_down: bool,
    pub mouse_right_button_down: bool,
}

pub struct InputChannel {
    data: Rc<RefCell<ChannelData>>,
}

impl InputChannel {
    pub fn key_down(&mut self, key: enigo::Key) {}
    pub fn key_up(&mut self, key: enigo::Key) {}
    pub fn mouse_move_relative(
        &mut self,
        delta_x: i32,
        delta_y: i32,
        left_button_down: bool,
        right_button_down: bool,
    ) {
        let mut data = (*self.data).borrow_mut();
        data.mouse_delta_x += delta_x;
        data.mouse_delta_y += delta_y;
        data.mouse_left_button_down = left_button_down;
        data.mouse_right_button_down = right_button_down;
    }
}

type ChannelId = String;

struct ChannelData {
    mouse_delta_x: i32,
    mouse_delta_y: i32,
    mouse_left_button_down: bool,
    mouse_right_button_down: bool,
}

impl ChannelData {
    fn new() -> Self {
        ChannelData {
            mouse_delta_x: 0,
            mouse_delta_y: 0,
            mouse_left_button_down: false,
            mouse_right_button_down: false,
        }
    }
}

// Collect latest input for each channel and average together
pub struct InputCombiner {
    channel_data: HashMap<ChannelId, Rc<RefCell<ChannelData>>>,
}

impl InputCombiner {
    pub fn new() -> Self {
        InputCombiner {
            channel_data: HashMap::new(),
        }
    }
    pub fn channel<S>(&mut self, id: S) -> InputChannel
    where
        S: Into<String>,
    {
        let id = id.into();
        let data = self
            .channel_data
            .entry(id.clone())
            .or_insert_with(|| Rc::new(RefCell::new(ChannelData::new())));
        InputChannel {
            data: Rc::clone(data),
        }
    }
    pub fn step(&mut self) -> Output {
        let mut sum_mouse_delta_x: i64 = 0;
        let mut sum_mouse_delta_y: i64 = 0;

        for (_id, data) in &mut self.channel_data {
            let mut data: RefMut<ChannelData> = (**data).borrow_mut();

            sum_mouse_delta_x += data.mouse_delta_x as i64;
            sum_mouse_delta_y += data.mouse_delta_y as i64;

            data.mouse_delta_x = 0;
            data.mouse_delta_y = 0;
        }

        // Average mouse delta
        let count = self.channel_data.len() as i64;
        let mouse_avg_delta_x = if count == 0 {
            0
        } else {
            sum_mouse_delta_x / count
        };
        let mouse_avg_delta_y = if count == 0 {
            0
        } else {
            sum_mouse_delta_y / count
        };

        Output {
            mouse_delta_x: mouse_avg_delta_x.try_into().unwrap_or_default(),
            mouse_delta_y: mouse_avg_delta_y.try_into().unwrap_or_default(),
            mouse_left_button_down: false,
            mouse_right_button_down: false,
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
