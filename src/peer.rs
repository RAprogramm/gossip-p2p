use crate::peer_storage::{PeerAddr, PeersStorage};

use super::message::Message;

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler, NodeListener};

use crate::printer::{init as logger_init, print_event};
use std::io::{self};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Peer {
    public_addr: SocketAddr,
    handler: NodeHandler<()>,
    period: u32,
    connect: Option<String>,
    node_listener: Option<NodeListener<()>>,
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
            handler,
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
            match self.handler.network().connect(Transport::FramedTcp, addr) {
                Ok((endpoint, _)) => {
                    {
                        let mut peers = self.participants.lock().unwrap();
                        peers.add_old_one(endpoint);
                    }

                    // Передаю свой публичный адрес
                    send_message(
                        &self.handler,
                        endpoint,
                        &Message::MyPubAddr(self.public_addr),
                    );

                    // Request a list of existing peers
                    // Response will be in event queue
                    send_message(&self.handler, endpoint, &Message::GiveMeAListOfPeers);
                }
                Err(_) => {
                    println!("Failed to connect to {}", &addr);
                }
            }
        }

        let handler = self.handler.clone();
        let period = self.period;
        let time_start_clone = Arc::clone(&self.time_start);

        let tick_duration = Duration::from_secs(period as u64);
        let peers_mut = Arc::clone(&self.participants);
        let handler_clone = handler.clone();
        thread::spawn(move || loop {
            thread::sleep(tick_duration);
            print_event(time_start_clone.clone(), "test message");

            let peers = peers_mut.lock().unwrap();
            let receivers = peers.receivers();

            if receivers.is_empty() {
                continue;
            }

            let msg_text = "test message".to_string();
            let msg = Message::Info(msg_text.clone());

            log_sending_message(
                &msg_text,
                &receivers
                    .iter()
                    .map(|PeerAddr { public, .. }| public)
                    .collect::<Vec<&SocketAddr>>(),
            );

            for PeerAddr { endpoint, .. } in receivers {
                send_message(&handler_clone, endpoint, &msg);
            }
        });

        let node_listener = self.node_listener.take().unwrap();
        node_listener.for_each(move |event| match event.network() {
            NetEvent::Accepted(endpoint, _) => {
                {
                    let mut peers = self.participants.lock().unwrap();
                    peers.add_old_one(endpoint);
                }

                // Передаю свой публичный адрес
                send_message(
                    &self.handler,
                    endpoint,
                    &Message::MyPubAddr(self.public_addr),
                );

                // Request a list of existing peers
                // Response will be in event queue
                send_message(&self.handler, endpoint, &Message::GiveMeAListOfPeers);
            }
            NetEvent::Connected(_, _) => {}
            NetEvent::Message(message_sender, input_data) => {
                let message: Message = bincode::deserialize(input_data).unwrap();
                match message {
                    Message::MyPubAddr(pub_addr) => {
                        let mut participants = self.participants.lock().unwrap();
                        participants.add_new_one(message_sender, pub_addr);
                    }
                    Message::GiveMeAListOfPeers => {
                        let list = {
                            let participants = self.participants.lock().unwrap();
                            participants.get_peers_list()
                        };
                        let msg = Message::TakePeersList(list);
                        send_message(&self.handler, message_sender, &msg);
                    }
                    Message::TakePeersList(addrs) => {
                        let filtered: Vec<&SocketAddr> =
                            addrs.iter().filter(|x| x != &&self.public_addr).collect();

                        print_event(self.time_start.clone(), &format_list_of_addrs(&filtered));

                        for peer in &filtered {
                            if peer == &&message_sender.addr() {
                                continue;
                            }

                            log_connected_to_the_peers(&filtered);
                        }
                    }
                    Message::Info(text) => {
                        let pub_addr = self
                            .participants
                            .lock()
                            .unwrap()
                            .get_pub_addr(&message_sender)
                            .unwrap();
                        log_message_received(&pub_addr, &text);
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

trait ToSocketAddr {
    fn get_addr(&self) -> SocketAddr;
}

impl ToSocketAddr for Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

impl ToSocketAddr for &Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

impl ToSocketAddr for SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        *self
    }
}

impl ToSocketAddr for &SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        **self
    }
}

fn format_list_of_addrs<T: ToSocketAddr>(items: &[T]) -> String {
    if items.is_empty() {
        "[no one]".to_owned()
    } else {
        let joined = items
            .iter()
            .map(|x| format!("\"{}\"", ToSocketAddr::get_addr(x)))
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", joined)
    }
}

// fn log_my_address<T: ToSocketAddr>(addr: &T) {
//     println!("My address is \"{}\"", ToSocketAddr::get_addr(addr));
// }

fn log_connected_to_the_peers<T: ToSocketAddr>(peers: &[T]) {
    println!("Connected to the peers at {}", format_list_of_addrs(peers));
}

fn log_message_received<T: ToSocketAddr>(from: &T, text: &str) {
    println!(
        "Received message [{}] from \"{}\"",
        text,
        ToSocketAddr::get_addr(from)
    );
}

fn log_sending_message<T: ToSocketAddr>(message: &str, receivers: &[T]) {
    println!(
        "Sending message [{}] to {}",
        message,
        format_list_of_addrs(receivers)
    );
}

fn send_message(handler: &NodeHandler<()>, to: Endpoint, msg: &Message) {
    let output_data = bincode::serialize(msg).unwrap();
    handler.network().send(to, &output_data);
}
