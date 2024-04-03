use crate::peer_storage::{PeerAddr, PeersStorage};
use crate::utils::{format_list_of_addrs, send_message};

use super::message::Message;

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler, NodeListener};
use rand::Rng;

use crate::printer::{init as logger_init, print_event};
use std::io::{self};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Peer {
    node_handler: Arc<Mutex<NodeHandler<()>>>,
    node_listener: Option<NodeListener<()>>,
    public_addr: SocketAddr,
    period: u32,
    connect: Option<String>,
    participants: Arc<Mutex<PeersStorage<Endpoint>>>,
    time_start: Arc<Instant>,
}

impl Peer {
    pub fn new(period: u32, port: u32, connect: Option<String>) -> io::Result<Self> {
        let (handler, listener) = node::split::<()>();

        let listen_addr = format!("127.0.0.1:{}", port);
        let (_, public_addr) = handler
            .network()
            .listen(Transport::FramedTcp, listen_addr)?;

        let time_start = logger_init(&public_addr);

        Ok(Self {
            public_addr,
            node_handler: Arc::new(Mutex::new(handler)),
            node_listener: Some(listener),
            connect,
            period,
            participants: Arc::new(Mutex::new(PeersStorage::new(public_addr))),
            time_start,
        })
    }

    pub fn run(mut self) {
        // Connection to the first peer
        if let Some(addr) = &self.connect {
            let network = self.node_handler.lock().unwrap();

            // Connection to the first peer
            match network.network().connect(Transport::FramedTcp, addr) {
                Ok((endpoint, _)) => {
                    let mut peers = self.participants.lock().unwrap();
                    peers.add_known_peer(endpoint);
                }
                Err(_) => {
                    println!("Failed to connect to {}", &addr);
                }
            }
        }

        self.sending_random_message();

        // let node_listener = self.node_listener.take().unwrap();
        if let Some(node_listener) = self.node_listener.take() {
            node_listener.for_each(move |event| match event.network() {
                NetEvent::Accepted(_, _) => {}
                // NetEvent::Connected(_, _) => {}
                NetEvent::Connected(endpoint, established) => {
                    if established {
                        self.connected(endpoint)
                    } else {
                        println!("Can not connect to {}", endpoint.addr());
                        std::process::exit(1);
                    }
                }
                NetEvent::Message(message_sender, input_data) => {
                    let message: Message = bincode::deserialize(input_data).unwrap();

                    match message {
                        Message::PublicAddress(pub_addr) => {
                            let mut participants = self.participants.lock().unwrap();
                            participants.add_unknown_peer(message_sender, pub_addr);
                        }

                        Message::PushPeersList => {
                            let list = {
                                let participants = self.participants.lock().unwrap();
                                participants.get_peers_list()
                            };
                            let msg = Message::PullPeersList(list);
                            send_message(
                                &mut self.node_handler.lock().unwrap(),
                                message_sender,
                                &msg,
                            );
                        }

                        Message::PullPeersList(addrs) => {
                            self.pull_peers_list(message_sender, addrs)
                        }

                        Message::Text(text) => {
                            let pub_addr = self
                                .participants
                                .lock()
                                .unwrap()
                                .get_pub_addr(&message_sender)
                                .unwrap();

                            let formatted_msg =
                                format!("Received message [{}] from \"{}\"", &text, &pub_addr);
                            print_event(self.time_start.clone(), &formatted_msg);
                        }
                    }
                }

                NetEvent::Disconnected(endpoint) => {
                    let mut participants = self.participants.lock().unwrap();
                    // participants.remove_peer(endpoint);

                    PeersStorage::drop(&mut participants, endpoint);
                }
            });
        }
    }

    fn sending_random_message(&self) {
        let tick_duration = Duration::from_secs(self.period as u64);
        let peers_clone = Arc::clone(&self.participants);
        let handler_clone = Arc::clone(&self.node_handler);
        let clone_start_time = self.time_start.clone();

        thread::spawn(move || loop {
            thread::sleep(tick_duration);

            let peers = peers_clone.lock().unwrap();
            let receivers = peers.receivers();

            if receivers.is_empty() {
                continue;
            }

            let mut network = handler_clone.lock().unwrap();

            let msg_text = format!("random message {}", rand::thread_rng().gen_range(0..1000));
            let msg = Message::Text(msg_text.clone());

            let formatted_msg = format!(
                "Sending message [{}] to {}",
                &msg_text,
                format_list_of_addrs(
                    &receivers
                        .iter()
                        .map(|PeerAddr { public, .. }| public)
                        .collect::<Vec<&SocketAddr>>(),
                )
            );
            print_event(clone_start_time.clone(), &formatted_msg);

            for PeerAddr { endpoint, .. } in &receivers {
                send_message(&mut network, *endpoint, &msg);
            }
        });
    }

    fn connected(&self, endpoint: Endpoint) {
        let mut peers = self.participants.lock().unwrap();
        peers.add_known_peer(endpoint);

        let mut network = self.node_handler.lock().unwrap();
        // Передаю свой публичный адрес
        send_message(
            &mut network,
            endpoint,
            &Message::PublicAddress(self.public_addr),
        );

        // Request a list of existing peers
        // Response will be in event queue
        send_message(&mut network, endpoint, &Message::PushPeersList);
    }

    fn pull_peers_list(&self, message_sender: Endpoint, addrs: Vec<SocketAddr>) {
        let filtered: Vec<&SocketAddr> = addrs
            .iter()
            .filter(|addr| addr != &&self.public_addr)
            .collect();

        let foramtted_msg = format!(
            "Connected to the peers at {}",
            format_list_of_addrs(&filtered)
        );
        print_event(self.time_start.clone(), &foramtted_msg);

        let mut network = self.node_handler.lock().unwrap();

        for peer in &filtered {
            if peer == &&message_sender.addr() {
                continue;
            }
            //===============================================================
            let (endpoint, _) = network
                .network()
                .connect(Transport::FramedTcp, **peer)
                .unwrap();

            // sending public address
            let msg = Message::PublicAddress(self.public_addr);
            send_message(&mut network, endpoint, &msg);

            // saving peer
            self.participants.lock().unwrap().add_known_peer(endpoint);
            //================================================================

            // let (endpoint, _) = self
            //     .node_handler
            //     .network()
            //     .connect(Transport::FramedTcp, **peer)
            //     .unwrap();

            // sending public address
            // let msg = Message::MyPubAddr(self.public_addr);
            // send_message(&self.node_handler, endpoint, &msg);

            // saving peer
            // self.participants.lock().unwrap().add_known_peer(endpoint);
        }
    }
}
