extern crate conv;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

mod input_combiner;
mod weighted_average;

use crossbeam_channel::{Receiver, Sender};
use enigo::{Enigo, Key, MouseControllable};
use hotkey;
use serde::{Deserialize, Serialize};
use slog::{Drain, Logger};
use std::net::{SocketAddr, TcpListener};
use std::thread::{self, spawn, JoinHandle};
use std::time::Duration;
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
        client_id: String,
        delta_x: i32,
        delta_y: i32,
        left_button_down: bool,
        right_button_down: bool,
    },
    KeyDown {
        client_id: String,
        key: enigo::Key,
    },
    KeyUp {
        client_id: String,
        key: enigo::Key,
    },
    Tick,
    ToggleInput,
}

const MOUSE_ENABLED: bool = false;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let (tx, rx) = crossbeam_channel::bounded(0);

    action_handler(rx, log.clone());

    register_hotkeys(tx.clone());

    ticker(tx.clone());

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

fn ticker(tx: Sender<Action>) -> JoinHandle<()> {
    spawn(move || loop {
        // TODO Convert to crossbeam_channel::tick and make frequency configurable
        thread::sleep(Duration::from_millis(100));
        tx.send(Action::Tick).unwrap();
    })
}

fn action_handler(rx: Receiver<Action>, log: Logger) -> JoinHandle<()> {
    spawn(move || {
        let mut enigo = Enigo::new();
        let mut input_enabled: bool = true;
        let mut input_combiner = input_combiner::InputCombiner::new();
        let (mut last_mouse_x, mut last_mouse_y) = Enigo::mouse_location();
        let mut last_mouse_left_button_down = false;
        let mut last_mouse_right_button_down = false;


        for action in rx {
            match action {
                Action::Mouse {
                    client_id,
                    delta_x,
                    delta_y,
                    left_button_down,
                    right_button_down,
                } => {
                    let mut channel = input_combiner.channel(client_id);

                    channel.mouse_move_relative(
                        delta_x,
                        delta_y,
                        left_button_down,
                        right_button_down,
                    );
                }
                Action::KeyDown { client_id, key } => {
                    let mut channel = input_combiner.channel(client_id);

                    debug!(log, "key_down"; "key"=>format!("{:?}", key));
                    channel.key_down(key);
                }
                Action::KeyUp { client_id, key } => {
                    let mut channel = input_combiner.channel(client_id);

                    debug!(log, "key_up"; "key"=>format!("{:?}", key));
                    channel.key_up(key);
                }
                Action::ToggleInput => {
                    input_enabled = !input_enabled;
                    info!(log, "Toggle input enabled"; "input_enabled" => input_enabled);
                }
                Action::Tick => {
                    let input_combiner::Output {
                        mouse_delta_x,
                        mouse_delta_y,
                        mouse_left_button_down,
                        mouse_right_button_down,
                    } = input_combiner.step();

                    let (mx, my) = Enigo::mouse_location();
                    if mx != last_mouse_x || my != last_mouse_y {
                        warn!(log, "unexpected mouse move"; "dx"=>mx-last_mouse_x, "dy"=>last_mouse_y-my);
                    }

                    if mouse_delta_x != 0 || mouse_delta_y != 0 || last_mouse_left_button_down != mouse_left_button_down || last_mouse_right_button_down != mouse_right_button_down {
                        debug!(log, "step"; "dx" => mouse_delta_x, "dy"=>mouse_delta_x, "lb"=>mouse_left_button_down, "rb"=>mouse_right_button_down);
                    }

                    if input_enabled && MOUSE_ENABLED {
                        enigo.mouse_move_relative(mouse_delta_x, mouse_delta_y);

                        if mouse_left_button_down && !last_mouse_left_button_down {
                            enigo.mouse_down(enigo::MouseButton::Left)
                        } else if !mouse_left_button_down && last_mouse_left_button_down {
                            enigo.mouse_up(enigo::MouseButton::Left)
                        }

                        if mouse_right_button_down && !last_mouse_right_button_down {
                            enigo.mouse_down(enigo::MouseButton::Right)
                        } else if !mouse_right_button_down && last_mouse_right_button_down {
                            enigo.mouse_up(enigo::MouseButton::Right)
                        }
                    }

                    let (mx, my) = Enigo::mouse_location();
                    last_mouse_x = mx;
                    last_mouse_y = my;
                    last_mouse_left_button_down = mouse_left_button_down;
                    last_mouse_right_button_down = mouse_right_button_down;
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
        let peer_addr = stream.peer_addr().unwrap().to_string();
        let log = log.new(o!("peer_addr" => peer_addr.clone()));

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
                                client_id: peer_addr.clone(),
                                delta_x: dx,
                                delta_y: dy,
                                left_button_down: get_bit_at(btns, 0),
                                right_button_down: get_bit_at(btns, 1),
                            })
                            .unwrap();
                        }
                        Ok(ClientInput::KeyDown { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                tx.send(Action::KeyDown {
                                    client_id: peer_addr.clone(),
                                    key,
                                })
                                .unwrap();
                            } else {
                                warn!(log, "Unsupported key"; "code" => code)
                            }
                        }
                        Ok(ClientInput::KeyUp { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                tx.send(Action::KeyUp {
                                    client_id: peer_addr.clone(),
                                    key,
                                })
                                .unwrap();
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
