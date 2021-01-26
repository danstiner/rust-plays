extern crate conv;
use conv::*;
use enigo::{Enigo, MouseControllable};

fn main() {
    let (xp, yp) = (0.5, 0.5);

    let (ws, hs) = Enigo::main_display_size();
    let w = f64::value_from(ws).unwrap();
    let h = f64::value_from(hs).unwrap();

    let mut enigo = Enigo::new();
    enigo.mouse_move_to((xp*w).round() as i32, (yp*h).round() as i32);
}
