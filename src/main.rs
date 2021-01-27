extern crate conv;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use conv::*;
use enigo::{Enigo, MouseControllable, KeyboardControllable, Key};
use hotkey;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;
use tungstenite::server::accept;
use tungstenite::Message;
use serde::{Deserialize, Serialize};
use slog::Drain;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Input {
    Mouse {x: f64, y: f64, b: u16},
    KeyDown { code: String },
    KeyUp { code: String },
}

fn main() {
    let input_enabled  = Arc::new(AtomicBool::new(true));
    let mouse_enabled = false;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let input_enabled_clone = Arc::clone(&input_enabled);
    let log_clone = log.new(o!());
    spawn (move || {
        let mut hk = hotkey::Listener::new();
        hk.register_hotkey(
            hotkey::modifiers::SHIFT,
            hotkey::keys::ESCAPE,
            move || {
                println!("Escape pressed!");
                let input_enabled_old = input_enabled_clone.fetch_xor(true, Ordering::Relaxed);
                info!(log_clone, "Toggle input enabled"; "input_enabled" => !input_enabled_old);
            },
        )
            .unwrap();
        hk.listen();
    });

    let server = TcpListener::bind("0.0.0.0:8080").unwrap();
    for stream in server.incoming() {
        let stream = stream.unwrap();
        let log = log.new(o!("peer" => stream.peer_addr().unwrap()));
        let input_enabled = Arc::clone(&input_enabled);

        spawn (move || {
            let mut enigo = Enigo::new();
            let mut websocket = accept(stream).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();

                if msg.is_ping() {
                    websocket.write_message(Message::Pong(msg.into_data())).unwrap();
                } else if msg.is_binary() {
                    info!(log, "Binary message");
                } else if msg.is_text() {
                    let input: serde_json::error::Result<Input> = serde_json::from_str(msg.to_text().unwrap());

                    let input_enabled= input_enabled.load(Ordering::Relaxed);
                    if !input_enabled {
                        continue
                    }

                    match input {
                        Ok(Input::Mouse { x, y, b: _ }) => {

                            let (ws, hs) = Enigo::main_display_size();
                            let w = f64::value_from(ws).unwrap();
                            let h = f64::value_from(hs).unwrap();

                            let x = (x * w).round() as i32;
                            let y = (y * h).round() as i32;
                            if input_enabled && mouse_enabled {
                                enigo.mouse_move_to(x, y);
                            }
                        }
                        Ok(Input::KeyDown { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                enigo.key_down(key);
                            } else {
                                warn!(log, "Unsupported key"; "code" => code)
                            }
                        }
                        Ok(Input::KeyUp { code }) => {
                            if let Some(key) = translate_key_code(&code) {
                                enigo.key_up(key);
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
        _ => None
    }
}
