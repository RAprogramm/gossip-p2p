use crate::peer_storage::PeersStorage;

use super::message::Message;

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler, NodeListener};

use crate::printer::init as logger_init;
use std::io::{self};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// struct ParticipantInfo {
//     addr: SocketAddr,
//     endpoint: Endpoint,
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
                    println!("Successfully connected to {}", connect_str);

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
        node_listener.for_each(move |event| match event.network() {
            NetEvent::Accepted(_, _) => (),
            NetEvent::Connected(endpoint, established) => {
                if established {
                    // println!("Connected to a new peer: {:?}", self.public_addr);

                    let mut participants = self.participants.lock().unwrap();
                    participants.add_new_one(endpoint, endpoint.addr());

                    // println!("Connected {}", endpoint);
                }
            }
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

                        log_connected_to_the_peers(&filtered);

                        for peer in &filtered {
                            if peer == &&message_sender.addr() {
                                continue;
                            }

                            // к каждому подключиться и послать свой публичный адрес и запомнить
                            let (endpoint, _) = self
                                .handler
                                .network()
                                .connect(Transport::FramedTcp, **peer)
                                .unwrap();

                            // sending public address
                            let msg = Message::MyPubAddr(self.public_addr);
                            send_message(&self.handler, endpoint, &msg);

                            // saving peer
                            self.participants.lock().unwrap().add_old_one(endpoint);
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

    //     fn register(
    //         &self,
    //         participants: Arc<Mutex<PeersStorage<Endpoint>>>,
    //         addr: SocketAddr,
    //         endpoint: Endpoint,
    //         handler: &NodeHandler<()>,
    //     ) {
    //         let mut storage = participants.lock().unwrap(); // Блокируем хранилище для безопасного доступа
    //
    //         if storage.get_pub_addr(&endpoint).is_none() {
    //             // Проверяем, что участник не зарегистрирован
    //             // Пример получения списка адресов текущих участников для отправки новому участнику
    //             let list = storage.get_peers_list().to_vec();
    //             // .iter()
    //             // .copied()
    //             // // .map(|addr| addr)
    //             // .collect::<Vec<SocketAddr>>();
    //
    //             let message = Message::ParticipantList(list.clone());
    //             let output_data = bincode::serialize(&message).unwrap();
    //             handler.network().send(endpoint, &output_data); // Отправляем список текущих участников новому участнику
    //
    //             // Оповещаем всех остальных участников о добавлении нового участника
    //             let message = Message::ParticipantNotificationAdded(addr.to_string(), addr);
    //             let output_data = bincode::serialize(&message).unwrap();
    //
    //             for peer in storage.receivers() {
    //                 // Итерируемся по всем известным участникам
    //                 handler.network().send(peer.endpoint, &output_data); // Отправляем уведомление об новом участнике
    //             }
    //
    //             storage.add_new_one(endpoint, addr); // Добавляем нового участника в хранилище
    //             println!("Added participant with ip {}", addr);
    //             log_connected_to_the_peers(&list)
    //         } else {
    //             println!("Participant with addr '{}' already exists", addr);
    //         }
    //     }
    //
    //     fn unregister(
    //         &self,
    //         peers: &Arc<Mutex<PeersStorage<Endpoint>>>,
    //         endpoint: Endpoint,
    //         handler: &NodeHandler<()>,
    //     ) {
    //         let mut peers = peers.lock().unwrap(); // Блокировка для безопасного доступа к хранилищу
    //
    //         // Сначала удаляем участника, используя его endpoint
    //         peers.remove_peer(endpoint);
    //
    //         // Теперь уведомляем остальных участников об удалении этого участника
    //         let addr = endpoint.addr(); // Получаем адрес участника для уведомлений
    //         let message = Message::ParticipantNotificationRemoved(addr.to_string());
    //         let output_data = bincode::serialize(&message).unwrap();
    //
    //         for peer in peers.receivers() {
    //             // Используем метод receivers для получения списка всех текущих участников
    //             if peer.endpoint != endpoint {
    //                 // Убедимся, что не отправляем сообщение самому удаляемому участнику
    //                 handler.network().send(peer.endpoint, &output_data);
    //             }
    //         }
    //
    //         println!("Removed participant with endpoint {:?}", endpoint);
    //     }
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

fn format_list_of_addrs<T: ToSocketAddr>(items: &Vec<T>) -> String {
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

fn log_connected_to_the_peers<T: ToSocketAddr>(peers: &Vec<T>) {
    println!("Connected to the peers at {}", format_list_of_addrs(peers));
}

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
