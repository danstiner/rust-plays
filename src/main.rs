extern crate conv;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use conv::*;
use enigo::{Enigo, MouseControllable, KeyboardControllable, Key};
use std::net::TcpListener;
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
    let enable_mouse = false;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());

    let server = TcpListener::bind("0.0.0.0:8080").unwrap();
    for stream in server.incoming() {
        let stream = stream.unwrap();
        let log = log.new(o!("peer" => stream.peer_addr().unwrap()));
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

                    match input {
                        Ok(Input::Mouse { x, y, b: _ }) => {

                            let (ws, hs) = Enigo::main_display_size();
                            let w = f64::value_from(ws).unwrap();
                            let h = f64::value_from(hs).unwrap();

                            let x = (x * w).round() as i32;
                            let y = (y * h).round() as i32;
                            if enable_mouse {
                                enigo.mouse_move_to(x, y);
                            }
                        }
                        Ok(Input::KeyDown { code }) => {
                            match code.as_str() {
                                "KeyA" => enigo.key_down(Key::Layout('a')),
                                _ => warn!(log, "Unsupported key"; "code" => code)
                            }
                        }
                        Ok(Input::KeyUp { code }) => {
                            match code.as_str() {
                                "KeyA" => enigo.key_up(Key::Layout('a')),
                                _ => warn!(log, "Unsupported key"; "code" => code)
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
