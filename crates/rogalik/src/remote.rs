use std::{
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use tungstenite::{accept, Message};
use winit::{
    event::{ElementState, MouseButton},
    event_loop::EventLoopProxy,
};

use crate::app::ExternalEvent;

const IDLE_DELAY_MS: u64 = 10;

pub(crate) enum RemoteResponse {
    ScreenShot,
}

pub(crate) struct RemoteHandle {
    handle: Option<std::thread::JoinHandle<()>>,
    halt: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
    tx: Sender<RemoteResponse>,
}
impl Drop for RemoteHandle {
    fn drop(&mut self) {
        self.halt.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawns a remote controller thread.
///
/// Accepts a single client at a time.
pub(crate) fn spawn_remote_controller(
    event_loop: EventLoopProxy<ExternalEvent>,
) -> Result<RemoteHandle, ()> {
    let port: u16 = std::env::var("ROGALIK_REMOTE_PORT")
        .unwrap_or("5555".to_string())
        .parse()
        .inspect_err(|e| log::error!("Invalid remote port: {e}"))
        .map_err(|_| ())?;
    let host = std::env::var("ROGALIK_REMOTE_HOST").unwrap_or("127.0.0.1".to_string());

    let Ok(server) = TcpListener::bind((host.to_string(), port)) else {
        log::error!("Can't spawn remote server at {host}:{port}");
        return Err(());
    };

    log::info!("Remote server started at: {host}:{port}");

    let (tx, rx) = std::sync::mpsc::channel();

    let halt = Arc::new(AtomicBool::new(false));
    let thread_halt = Arc::clone(&halt);

    let handle = std::thread::spawn(move || {
        for stream in server.incoming() {
            if thread_halt.load(Ordering::Relaxed) {
                return;
            }

            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Invalid remote connection: {e}");
                    continue;
                }
            };
            let mut ws = match accept(stream) {
                Ok(ws) => {
                    log::info!("Remote server accepted new client stream");
                    ws
                }
                Err(e) => {
                    log::error!("Remote handshake failed: {e}");
                    continue;
                }
            };
            if let Err(e) = ws.get_mut().set_nonblocking(true) {
                log::error!("Remote websocket nonblocking mode failed: {e}");
                continue;
            }

            loop {
                let mut idle = true;

                match ws.read() {
                    Ok(Message::Text(msg)) => {
                        idle = false;
                        if let Some(event) = parse_request(msg.as_str()) {
                            log::info!("Received remote event {event:?}");
                            event_loop.send_event(event);
                        } else {
                            log::warn!("Invalid ws text message: {msg}");
                        }
                    }
                    Ok(Message::Close(_)) => (),
                    Ok(msg) => {
                        idle = false;
                        log::warn!("Invalid ws message: {msg}");
                    }
                    Err(tungstenite::Error::Io(e))
                        if e.kind() == std::io::ErrorKind::WouldBlock => {}

                    Err(e) => {
                        log::error!("Remote connection error: {e}");
                        break;
                    }
                }

                match rx.try_recv() {
                    Ok(RemoteResponse::ScreenShot) => {
                        idle = false;
                        // TODO send the screenshot
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => (),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        return;
                    }
                }

                if idle {
                    std::thread::sleep(std::time::Duration::from_millis(IDLE_DELAY_MS));
                }
            }
        }
    });

    Ok(RemoteHandle {
        handle: Some(handle),
        halt,
        tx,
    })
}

fn parse_request(request: &str) -> Option<ExternalEvent> {
    let parts = request.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        ["mousemove", x, y] => Some(ExternalEvent::MouseMove(x.parse().ok()?, y.parse().ok()?)),
        ["mousebutton", "left", "down"] => Some(ExternalEvent::MouseButton(
            MouseButton::Left,
            ElementState::Pressed,
        )),
        ["mousebutton", "left", "up"] => Some(ExternalEvent::MouseButton(
            MouseButton::Left,
            ElementState::Released,
        )),
        _ => None,
    }
}
