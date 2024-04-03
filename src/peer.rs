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
    node_handler: NodeHandler<()>,
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
            node_handler: handler,
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
            match self
                .node_handler
                .network()
                .connect(Transport::FramedTcp, addr)
            {
                Ok((endpoint, _)) => {
                    {
                        let mut peers = self.participants.lock().unwrap();
                        peers.add_known_peer(endpoint);
                    }

                    // Передаю свой публичный адрес
                    send_message(
                        &self.node_handler,
                        endpoint,
                        &Message::MyPubAddr(self.public_addr),
                    );

                    // Request a list of existing peers
                    // Response will be in event queue
                    send_message(&self.node_handler, endpoint, &Message::GiveMeAListOfPeers);
                }
                Err(_) => {
                    println!("Failed to connect to {}", &addr);
                }
            }
        }

        let handler = self.node_handler.clone();
        let period = self.period;
        let tick_duration = Duration::from_secs(period as u64);
        let peers_mut = Arc::clone(&self.participants);
        let handler_clone = handler.clone();
        let clone_start_time = self.time_start.clone();

        thread::spawn(move || loop {
            thread::sleep(tick_duration);

            let peers = peers_mut.lock().unwrap();
            let receivers = peers.receivers();

            if receivers.is_empty() {
                continue;
            }

            let msg_text = format!("random message {}", rand::thread_rng().gen_range(0..1000));
            let msg = Message::Info(msg_text.clone());

            let formated_msg = format!(
                "Sending message [{}] to {}",
                &msg_text,
                format_list_of_addrs(
                    &receivers
                        .iter()
                        .map(|PeerAddr { public, .. }| public)
                        .collect::<Vec<&SocketAddr>>(),
                )
            );
            print_event(clone_start_time.clone(), &formated_msg);

            for PeerAddr { endpoint, .. } in receivers {
                send_message(&handler_clone, endpoint, &msg);
            }
        });

        let node_listener = self.node_listener.take().unwrap();
        node_listener.for_each(move |event| match event.network() {
            NetEvent::Accepted(_, _) => {}
            NetEvent::Connected(endpoint, _) => {
                {
                    let mut peers = self.participants.lock().unwrap();
                    peers.add_known_peer(endpoint);
                }

                // Передаю свой публичный адрес
                send_message(
                    &self.node_handler,
                    endpoint,
                    &Message::MyPubAddr(self.public_addr),
                );

                // Request a list of existing peers
                // Response will be in event queue
                send_message(&self.node_handler, endpoint, &Message::GiveMeAListOfPeers);
            }

            NetEvent::Message(message_sender, input_data) => {
                let message: Message = bincode::deserialize(input_data).unwrap();

                match message {
                    Message::MyPubAddr(pub_addr) => {
                        let mut participants = self.participants.lock().unwrap();
                        participants.add_unknown_peer(message_sender, pub_addr);
                    }

                    Message::GiveMeAListOfPeers => {
                        let list = {
                            let participants = self.participants.lock().unwrap();
                            participants.get_peers_list()
                        };
                        let msg = Message::TakePeersList(list);
                        send_message(&self.node_handler, message_sender, &msg);
                    }

                    Message::TakePeersList(addrs) => {
                        let filtered: Vec<&SocketAddr> = addrs
                            .iter()
                            .filter(|addr| addr != &&self.public_addr)
                            .collect();

                        let foramtted_msg = format!(
                            "Connected to the peers at {}",
                            format_list_of_addrs(&filtered)
                        );
                        print_event(self.time_start.clone(), &foramtted_msg);

                        for peer in &filtered {
                            if peer == &&message_sender.addr() {
                                continue;
                            }
                        }
                    }

                    Message::Info(text) => {
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
                participants.remove_peer(endpoint);
            }
        });
    }
}

