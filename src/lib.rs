extern crate conv;

pub mod broadcast_channel;
pub mod input_combiner;
pub mod weighted_average;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientInput {
    Mouse { dx: i32, dy: i32, btns: u16 },
    KeyDown { code: String },
    KeyUp { code: String },
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum ClientOutput {
    Output {
        dx: i32,
        dy: i32,
        lb: bool,
        rb: bool,
    },
}
