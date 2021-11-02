extern crate conv;
extern crate enigo;
extern crate rust_plays;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate tokio;

use enigo::{Enigo, MouseControllable};
use rust_plays::ClientOutput;
use slog::Drain;
use tungstenite::connect;
use url::Url;

const MOUSE_ENABLED: bool = true;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let url = Url::parse("ws://localhost:8090").unwrap();

    let (mut socket, _) = connect(url).expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let mut last_mouse_left_button_down = false;
    let mut last_mouse_right_button_down = false;
    let input_enabled = true;

    let mut enigo = Enigo::new();

    loop {
        let msg = socket.read_message().expect("Error reading message");

        if msg.is_ping() {
            // Do nothing, pings are handled automatically
        } else if msg.is_binary() {
            info!(log, "Unexpected binary message");
        } else if msg.is_text() {
            let msg = msg.into_text().unwrap();
            let output: serde_json::error::Result<ClientOutput> = serde_json::from_str(&msg);

            match output {
                Ok(ClientOutput::Output { dx, dy, lb, rb }) => {
                    let changed = dx != 0
                        || dy != 0
                        || last_mouse_left_button_down != lb
                        || last_mouse_right_button_down != rb;

                    if changed {
                        debug!(log, "step"; "dx" => dx, "dy"=>dy, "lb"=>lb, "rb"=>rb);
                    }

                    if input_enabled && MOUSE_ENABLED {
                        enigo.mouse_move_relative(dx, dy);

                        if lb && !last_mouse_left_button_down {
                            enigo.mouse_down(enigo::MouseButton::Left)
                        } else if !lb && last_mouse_left_button_down {
                            enigo.mouse_up(enigo::MouseButton::Left)
                        }

                        if rb && !last_mouse_right_button_down {
                            enigo.mouse_down(enigo::MouseButton::Right)
                        } else if !rb && last_mouse_right_button_down {
                            enigo.mouse_up(enigo::MouseButton::Right)
                        }
                    }

                    last_mouse_left_button_down = lb;
                    last_mouse_right_button_down = rb;
                }
                Err(err) => {
                    warn!(log, "Bad output"; "text" => &msg, "err" => err.to_string());
                }
            }
        }
    }
}
