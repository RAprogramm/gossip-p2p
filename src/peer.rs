use crate::peer_storage::{PeerAddr, PeersStorage};

use super::message::Message;

use message_io::events::TimerId;
use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler, NodeListener};

use crate::printer::{init as logger_init, print_event};
use std::io::{self};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// struct ParticipantInfo {
//     addr: SocketAddr,
//     endpoint: Endpoint,
// }

// enum MyEvent {
//     TimerTick,
//     // Другие события...
// }

pub struct Peer {
    public_addr: SocketAddr,
    handler: NodeHandler<()>,
    period: u32,
    node_listener: Option<NodeListener<()>>,
    participants: Arc<Mutex<PeersStorage<Endpoint>>>,
    time_start: Arc<Instant>,
}

impl Peer {
    pub fn new(period: u32, port: u32, connect: Option<String>) -> io::Result<Peer> {
        let (handler, node_listener) = node::split::<()>();

        let listen_addr = format!("127.0.0.1:{}", port);
        let (_, public_addr) = handler
            .network()
            .listen(Transport::FramedTcp, listen_addr)?;

        let time_start = logger_init(&public_addr);

        let peer = Peer {
            public_addr,
            handler,
            period,
            node_listener: Some(node_listener),
            participants: Arc::new(Mutex::new(PeersStorage::new(public_addr))),
            time_start,
        };

        if let Some(connect_str) = connect {
            // Попытка подключения к другому узлу
            match peer
                .handler
                .network()
                .connect(Transport::FramedTcp, &connect_str)
            {
                Ok((endpoint, _)) => {
                    // Здесь вы можете добавить логику регистрации или отправки специфических сообщений
                    // на подключенный узел сразу после установления соединения
                    let msg = format!("Successfully connected to {}", connect_str);
                    print_event(peer.time_start.clone(), &msg);

                    // Например, регистрируем этот endpoint в participants
                    peer.participants
                        .lock()
                        .unwrap()
                        .add_new_one(endpoint, public_addr);
                }
                Err(e) => {
                    eprintln!("Failed to connect to {}: {}", connect_str, e);
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to connect"));
                }
            }
        }

        Ok(peer)
    }

    pub fn run(mut self) {
        let node_listener = self.node_listener.take().unwrap();

        // let (sender, receiver) = message_io::events::split::<MyEvent>();
        // let timer_id: TimerId =
        //     sender.schedule_with_delay(Duration::from_secs(self.period.into()), MyEvent::TimerTick);
        // for event in receiver {
        //     match event {
        //         MyEvent::TimerTick => {}
        //     }
        // }
        // self.spawn_emit_loop(&self.handler, self.participants.clone(), self.period);

        node_listener.for_each(move |event| match event.network() {
            // NetEvent::Accepted(_, _) => (),
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
            // NetEvent::Connected(endpoint, established) => {
            //     if established {
            //
            //         let mut participants = self.participants.lock().unwrap();
            //         participants.add_new_one(endpoint, endpoint.addr());
            //
            //         println!("Connected {}", endpoint.addr());
            //     }
            // }
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
                                println!("{}", peer);
                                continue;
                            }

                            // к каждому подключиться и послать свой публичный адрес и запомнить
                            // let (endpoint, _) = self
                            //     .handler
                            //     .network()
                            //     .connect(Transport::FramedTcp, **peer)
                            //     .unwrap();
                            println!("{:?}", &filtered);

                            // sending public address
                            let msg = Message::MyPubAddr(self.public_addr);
                                println!("{:?}", msg);
                            // send_message(&self.handler, endpoint, &msg);

                            // saving peer
                            // self.participants.lock().unwrap().add_old_one(endpoint);
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
                    } // Message::RegisterParticipant(addr) => {
                      //     self.register(
                      //         self.participants.clone(),
                      //         addr,
                      //         message_sender,
                      //         &self.handler,
                      //     );
                      // }
                      // Message::UnregisterParticipant(_) => {
                      //     self.unregister(&self.participants, message_sender, &self.handler);
                      // }
                      // _ => unreachable!(),
                }
            }
            NetEvent::Disconnected(endpoint) => {
                let mut participants = self.participants.lock().unwrap();
                participants.remove_peer(endpoint);
            } // NetEvent::Disconnected(endpoint) => {
              //     self.peers_storage.remove_peer(endpoint);
              //     if endpoint == self.discovery_endpoint {
              //         println!("Discovery server disconnected, closing");
              //         self.handler.stop();
              //     }
              // }
        });
    }

    fn spawn_emit_loop(
        &self,
        handler: &NodeHandler<()>,
        participants: Arc<Mutex<PeersStorage<Endpoint>>>,
        period: u32,
    ) {
        let sleep_duration = Duration::from_secs(period as u64);
        let peers_mut = Arc::clone(&participants);
        let handler_clone = handler.clone(); // Клонируем NodeHandler для использования в потоке

        thread::spawn(move || loop {
            thread::sleep(sleep_duration);

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

// fn log_connected_to_the_peers<T: ToSocketAddr>(peers: &Vec<T>) {
//     print_event("Connected to the peers at {}", format_list_of_addrs(peers));
// }

fn log_message_received<T: ToSocketAddr>(from: &T, text: &str) {
    println!(
        "Received message [{}] from \"{}\"",
        text,
        ToSocketAddr::get_addr(from)
    );
}

fn log_sending_message<T: ToSocketAddr>(message: &str, receivers: &Vec<T>) {
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
