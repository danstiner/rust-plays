extern crate conv;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use crossbeam_channel::{Receiver, Sender};
use enigo::{Enigo, Key, KeyboardControllable, MouseControllable};
use hotkey;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger};
use std::net::{SocketAddr, TcpListener};
use std::thread::{spawn, JoinHandle};
use tungstenite::server::accept;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientInput {
    Mouse { dx: i32, dy: i32, btns: u16 },
    KeyDown { code: String },
    KeyUp { code: String },
}

enum Action {
    Mouse {
        delta_x: i32,
        delta_y: i32,
        left_button_down: bool,
        right_button_down: bool,
    },
    KeyDown(enigo::Key),
    KeyUp(enigo::Key),
    ToggleInput,
}

const MOUSE_ENABLED: bool = false;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let (tx, rx) = crossbeam_channel::bounded(0);

    register_hotkeys(tx.clone());

    action_handler(rx, log.clone());

    websocket_listener("0.0.0.0:8090".parse().unwrap(), tx, log);
}

fn register_hotkeys(tx: Sender<Action>) -> JoinHandle<()> {
    spawn(move || {
        let mut hk = hotkey::Listener::new();
        hk.register_hotkey(hotkey::modifiers::SHIFT, hotkey::keys::ESCAPE, move || {
            tx.send(Action::ToggleInput).unwrap();
        })
        .unwrap();
        hk.listen();
    })
}

fn action_handler(rx: Receiver<Action>, log: Logger) -> JoinHandle<()> {
    spawn(move || {
        let mut enigo = Enigo::new();
        let mut input_enabled: bool = true;

        for action in rx {
            match action {
                Action::Mouse {
                    delta_x,
                    delta_y,
                    left_button_down,
                    right_button_down,
                } => {
                    let (w, h) = Enigo::main_display_size();
                    let (mx, my) = Enigo::mouse_location();
                    debug!(log, "mouse"; "dx" => delta_x, "dy"=>delta_y, "lb"=>left_button_down, "rb"=>right_button_down, "x"=>mx, "y"=>my);

                    if input_enabled && MOUSE_ENABLED {
                        enigo.mouse_move_relative(delta_x, delta_y);

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
                    debug!(log, "key_down"; "key"=>format!("{:?}", key));
                    enigo.key_down(key);
                }
                Action::KeyUp(key) => {
                    debug!(log, "key_up"; "key"=>format!("{:?}", key));
                    enigo.key_up(key);
                }
                Action::ToggleInput => {
                    input_enabled = !input_enabled;
                    info!(log, "Toggle input enabled"; "input_enabled" => input_enabled);
                }
            }
        }
    })
}

fn websocket_listener(address: SocketAddr, tx: Sender<Action>, log: Logger) {
    let server = TcpListener::bind(address).unwrap();
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
                        Ok(ClientInput::Mouse { dx, dy, btns }) => {
                            tx.send(Action::Mouse {
                                delta_x: dx,
                                delta_y: dy,
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
