use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DISCOVERY_PORT: u16 = 45678;
const MESSAGE_PORT: u16 = 45679;
const BROADCAST_ADDR: &str = "255.255.255.255";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    Discovery,
    Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub msg_type: MessageType,
    pub sender_name: String,
    pub sender_ip: String,
    pub content: String,
}

pub struct Peer {
    pub name: String,
    pub ip: String,
}

pub fn start_discovery_service(peers: Arc<Mutex<HashMap<String, Peer>>>) {
    let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to bind discovery socket: {}", e);
            return;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        log::error!("Failed to enable broadcast: {}", e);
        return;
    }

    log::info!("Discovery service started on port {}", DISCOVERY_PORT);

    let mut buf = [0u8; 1024];
    socket.set_nonblocking(true).ok();

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&buf[..len]) {
                    if msg.msg_type == MessageType::Discovery {
                        let peer_ip = addr.ip().to_string();
                        let mut peers_guard = peers.lock().unwrap();
                        peers_guard.insert(
                            peer_ip.clone(),
                            Peer {
                                name: msg.sender_name.clone(),
                                ip: peer_ip,
                            },
                        );
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                log::error!("Discovery recv error: {}", e);
            }
        }
    }
}

pub fn broadcast_presence(username: String) {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create broadcast socket: {}", e);
            return;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        log::error!("Failed to enable broadcast: {}", e);
        return;
    }

    let addr: SocketAddr = format!("{}:{}", BROADCAST_ADDR, DISCOVERY_PORT)
        .parse()
        .unwrap();

    let msg = NetworkMessage {
        msg_type: MessageType::Discovery,
        sender_name: username,
        sender_ip: local_ip_address().unwrap_or_default(),
        content: String::new(),
    };

    if let Ok(data) = serde_json::to_vec(&msg) {
        let _ = socket.send_to(&data, addr);
    }
}

pub fn start_message_service<F>(peers: Arc<Mutex<HashMap<String, Peer>>>, callback: F)
where
    F: FnMut(String) + Send + Clone + 'static,
{
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", MESSAGE_PORT)) {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind message server: {}", e);
            return;
        }
    };

    log::info!("Message service started on port {}", MESSAGE_PORT);

    listener.set_nonblocking(true).ok();

    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    let mut callback_clone = callback.clone();
    thread::spawn(move || {
        for msg in rx {
            callback_clone(msg);
        }
    });

    loop {
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                let tx_clone = tx.clone();
                thread::spawn(move || {
                    let mut buf = [0u8; 4096];
                    if let Ok(n) = stream.read(&mut buf) {
                        if n > 0 {
                            if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&buf[..n]) {
                                if msg.msg_type == MessageType::Message {
                                    let display = format!(
                                        "[{}] {}: {}",
                                        chrono::Local::now().format("%H:%M:%S"),
                                        msg.sender_name,
                                        msg.content
                                    );
                                    let _ = tx_clone.send(display);
                                }
                            }
                        }
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => log::error!("Accept error: {}", e),
        }
    }
}

pub fn send_message_tcp(
    peer_ip: &str,
    sender_name: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect_timeout(
        &format!("{}:{}", peer_ip, MESSAGE_PORT).parse()?,
        Duration::from_secs(5),
    )?;

    let msg = NetworkMessage {
        msg_type: MessageType::Message,
        sender_name: sender_name.to_string(),
        sender_ip: local_ip_address().unwrap_or_default(),
        content: content.to_string(),
    };

    let data = serde_json::to_vec(&msg)?;
    stream.write_all(&data)?;
    Ok(())
}

fn local_ip_address() -> Option<String> {
    if let Ok(local_ip) = local_ip_address::local_ip() {
        Some(local_ip.to_string())
    } else {
        None
    }
}
