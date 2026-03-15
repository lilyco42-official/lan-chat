slint::include_modules!();

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod network;

fn main() {
    env_logger::init();
    println!("Starting LAN Chat application...");

    let peers: Arc<Mutex<HashMap<String, network::Peer>>> = Arc::new(Mutex::new(HashMap::new()));

    let ui = MainWindow::new().unwrap();
    let ui_weak = ui.as_weak();

    let peers_clone = peers.clone();

    ui.on_refresh_peers_click(move || {
        let peers = peers_clone.clone();
        let ui_weak = ui_weak.clone();

        thread::spawn(move || {
            network::broadcast_presence("User".to_string());
            thread::sleep(Duration::from_millis(500));

            let peers_guard = peers.lock().unwrap();
            let mut peer_list: Vec<slint::SharedString> = Vec::new();
            for p in peers_guard.values() {
                peer_list.push(slint::SharedString::from(format!("{} ({})", p.name, p.ip)));
            }
            drop(peers_guard);

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_peer_list(slint::ModelRc::from(peer_list.as_slice()));
                }
            });
        });
    });

    let peers_clone = peers.clone();

    ui.on_send_message(move |msg| {
        let msg_str = msg.to_string();
        let peers = peers_clone.clone();

        thread::spawn(move || {
            let peers_guard = peers.lock().unwrap();
            for (ip, _peer) in peers_guard.iter() {
                let _ = network::send_message_tcp(ip, "User", &msg_str);
            }
        });
    });

    ui.on_send_file(move |_path| {
        log::info!("File transfer not yet implemented in simple mode");
    });

    let peers_clone = peers.clone();

    thread::spawn(move || {
        network::start_discovery_service(peers_clone);
    });

    let peers_clone = peers.clone();
    let ui_weak = ui.as_weak();

    thread::spawn(move || {
        let (tx, rx): (Sender<String>, Receiver<String>) = channel();

        network::start_message_service(peers_clone, move |msg| {
            let _ = tx.send(msg);
        });

        loop {
            match rx.recv() {
                Ok(msg) => {
                    let ui_weak = ui_weak.clone();
                    let msg_clone = msg.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_message_history(slint::ModelRc::from(
                                &[slint::SharedString::from(msg_clone)][..],
                            ));
                        }
                    });
                }
                Err(_) => break,
            }
        }
    });

    ui.run().unwrap();
}
