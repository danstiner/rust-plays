extern crate conv;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use conv::*;
use crossbeam_channel::Sender;
use enigo::{Enigo, Key, KeyboardControllable, MouseControllable};
use hotkey;
use serde::{Deserialize, Serialize};
use slog::Drain;
use std::net::TcpListener;
use std::thread::{JoinHandle, spawn};
use tungstenite::server::accept;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientInput {
    AbsMouse { xp: f64, yp: f64, btns: u16 },
    KeyDown { code: String },
    KeyUp { code: String },
}

enum Action {
    AbsMouse {
        x_percentage: f64,
        y_percentage: f64,
        left_button_down: bool,
        right_button_down: bool,
    },
    KeyDown(enigo::Key),
    KeyUp(enigo::Key),
    ToggleInput,
}

const mouse_enabled: bool = false;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let (tx, rx) = crossbeam_channel::bounded(0);

    register_hotkeys(tx.clone());

    let log_clone = log.new(o!());
    spawn(move || {
        let mut enigo = Enigo::new();
        let mut input_enabled: bool = true;

        for action in rx {
            match action {
                Action::AbsMouse {
                    x_percentage,
                    y_percentage,
                    left_button_down,
                    right_button_down,
                } => {
                    let (ws, hs) = Enigo::main_display_size();
                    let w = f64::value_from(ws).unwrap();
                    let h = f64::value_from(hs).unwrap();

                    let x = (x_percentage * w).round() as i32;
                    let y = (y_percentage * h).round() as i32;
                    if input_enabled && mouse_enabled {
                        enigo.mouse_move_to(x, y);

                        if left_button_down {
                            enigo.mouse_down(enigo::MouseButton::Left)
                        } else {
                            enigo.mouse_up(enigo::MouseButton::Left)
                        }

                        if right_button_down {
                            enigo.mouse_down(enigo::MouseButton::Right)
                        } else {
                            enigo.mouse_up(enigo::MouseButton::Right)
                        }
                    }
                }
                Action::KeyDown(key) => {
                    enigo.key_down(key);
                }
                Action::KeyUp(key) => {
                    enigo.key_up(key);
                }
                Action::ToggleInput => {
                    input_enabled = !input_enabled;
                    info!(log_clone, "Toggle input enabled"; "input_enabled" => input_enabled);
                }
            }
        }
    });

    let server = TcpListener::bind("0.0.0.0:8090").unwrap();
    for stream in server.incoming() {
        let tx = tx.clone();
        let stream = stream.unwrap();
        let log = log.new(o!("peer" => stream.peer_addr().unwrap()));

        info!(log, "Incoming connection");

        spawn(move || {
            let mut websocket = accept(stream).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                if msg.is_ping() {
                    // Do nothing, pings are handled automatically
                } else if msg.is_binary() {
                    info!(log, "Unexpected binary message");
                } else if msg.is_text() {
                    let input: serde_json::error::Result<ClientInput> =
                        serde_json::from_str(msg.to_text().unwrap());

                    match input {
                        Ok(ClientInput::AbsMouse { xp, yp, btns }) => {
                            tx.send(Action::AbsMouse {
                                x_percentage: xp,
                                y_percentage: yp,
                                left_button_down: get_bit_at(btns, 0),
                                right_button_down: get_bit_at(btns, 1),
                            })
                            .unwrap();
                        }
                        Ok(ClientInput::KeyDown { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                tx.send(Action::KeyDown(key)).unwrap();
                            } else {
                                warn!(log, "Unsupported key"; "code" => code)
                            }
                        }
                        Ok(ClientInput::KeyUp { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                tx.send(Action::KeyUp(key)).unwrap();
                            } else {
                                warn!(log, "Unsupported key"; "code" => code)
                            }
                        }
                        Err(err) => {
                            warn!(log, "Bad input"; "text" => msg.to_text().unwrap(), "err" => err.to_string());
                        }
                    }
                }
            }
        });
    }
}

fn register_hotkeys(tx: Sender<Action>) -> JoinHandle<()> {
    spawn(move || {
        let mut hk = hotkey::Listener::new();
        hk.register_hotkey(hotkey::modifiers::SHIFT, hotkey::keys::ESCAPE, move || {
            tx.send(Action::ToggleInput).unwrap();
        }).unwrap();
        hk.listen();
    })
}

fn translate_key_code(code: &str) -> Option<Key> {
    match code {
        "KeyW" => Some(Key::Layout('w')),
        "KeyA" => Some(Key::Layout('a')),
        "KeyS" => Some(Key::Layout('s')),
        "KeyD" => Some(Key::Layout('d')),
        "KeyQ" => Some(Key::Layout('q')),
        "KeyE" => Some(Key::Layout('e')),
        "KeyR" => Some(Key::Layout('r')),
        "Enter" => Some(Key::Return),
        "Space" => Some(Key::Space),
        "ArrowUp" => Some(Key::UpArrow),
        "ArrowLeft" => Some(Key::LeftArrow),
        "ArrowRight" => Some(Key::RightArrow),
        "ArrowDown" => Some(Key::DownArrow),
        "Digit1" => Some(Key::Layout('1')),
        "Digit2" => Some(Key::Layout('2')),
        "Digit3" => Some(Key::Layout('3')),
        "Digit4" => Some(Key::Layout('4')),
        "Digit5" => Some(Key::Layout('5')),
        "Digit6" => Some(Key::Layout('6')),
        "Digit7" => Some(Key::Layout('7')),
        "Digit8" => Some(Key::Layout('8')),
        "Digit9" => Some(Key::Layout('9')),
        _ => None,
    }
}

fn get_bit_at(input: u16, n: u8) -> bool {
    if n < 16 {
        input & (1 << n) != 0
    } else {
        false
    }
}
