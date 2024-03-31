use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::logger::{init as logger_init, log};
use crate::message::Message;
use crate::peer_storage::PeersStorage;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Endpoint(SocketAddr);

impl Endpoint {
    fn addr(&self) -> SocketAddr {
        self.0
    }
}


pub struct Peer {
    peers: Arc<Mutex<PeersStorage>>,
    public_addr: SocketAddr,
    period: Duration,
    connect: Option<String>,
    time_start: Arc<Instant>,          
    receiver: mpsc::Receiver<Message>, 
    sender: mpsc::Sender<Message>,     
}

impl Peer {
    pub async fn new(port: u16, period: u64, connect: Option<String>) -> Result<Self, io::Error> {
        let public_addr = format!("127.0.0.1:{}", port).parse().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to parse socket address",
            )
        })?;
        let peers_storage = PeersStorage {
            peers: HashMap::new(),
        };

        let time_start = logger_init(&public_addr).await;

        let (sender, receiver) = mpsc::channel(100); // create channel

        Ok(Self {
            peers: Arc::new(Mutex::new(peers_storage)),
            public_addr,
            period: Duration::from_secs(period),
            connect,
            time_start, 
            sender,
            receiver,
        })
    }
    // init and run peer
    pub async fn run(&self) -> Result<(), io::Error> {
        // handle peer connection
        if let Some(addr) = &self.connect {
            self.initialize_connection(addr).await?;
        }

        // listen incom
        self.listen_for_incoming_connections().await;

        // mess
        self.periodic_message_sending().await;

        Ok(())
    }
    async fn initialize_connection(&self, addr: &str) -> Result<(), io::Error> {
        let peer_addr = addr
            .parse::<SocketAddr>()
            .expect("Failed to parse connect address");
        send_message_to_peer(&peer_addr, &Message::GiveMeAListOfPeers, &self.sender).await?;
        Ok(())
    }

    // listen and handle
    async fn listen_for_incoming_connections(&self) {
        let listener = TcpListener::bind(self.public_addr)
            .await
            .expect("Failed to bind to port");
        let peers = self.peers.clone();
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                let peers_clone = peers.clone();
                tokio::spawn(handle_incoming_messages(socket, peers_clone, self.sender));
            }
        });
    }

    // mess
    async fn periodic_message_sending(&self) {
        let period = self.period;
        let peers = self.peers.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            while interval.tick().await {
                let peers_guard = peers.lock().await;
                for peer_addr in peers_guard.peers.keys() {
                    let msg = Message::Info("Periodic message".to_string());
                    send_message_to_peer(&peer_addr.addr(), &msg, &sender)
                        .await
                        .unwrap_or_else(|e| eprintln!("Error: {}", e));
                }
            }
        });
    }
}

// impl Peer {
//     pub async fn new(port: u16, period: u64, connect: Option<String>) -> Result<Self, io::Error> {
//         let public_addr = format!("127.0.0.1:{}", port).parse().map_err(|_| {
//             io::Error::new(
//                 io::ErrorKind::InvalidInput,
//                 "Failed to parse socket address",
//             )
//         })?;
//         let peers_storage = PeersStorage {
//             peers: HashMap::new(),
//         };
//
//         let time_start = logger_init(&public_addr).await;
//
//         let (sender, receiver) = mpsc::channel(100); 
//
//         Ok(Self {
//             peers: Arc::new(Mutex::new(peers_storage)),
//             public_addr,
//             period: Duration::from_secs(period),
//             connect,
//             time_start, 
//             sender,
//             receiver,
//         })
//     }
//
//     pub async fn run(&self) -> Result<(), io::Error> {
//         if let Some(addr) = &self.connect {
//             let peer_addr = addr.parse().expect("Failed to parse address");
//             send_message_to_peer(
//                 &peer_addr,
//                 &Message::Info("Hello, peer!".to_string()),
//                 &self.sender,
//             )
//             .await?;
//             let stream = match TcpStream::connect(addr).await {
//                 Ok(mut stream) => {
//                     let connect_message = format!("connected to {}", addr);
//
//                     log(self.time_start.clone(), &connect_message).await; 
//                                                                           
//                     let local_addr = stream.local_addr()?;
//                     let message = format!("My public port is: {}", self.public_addr.port());
//                     stream.write_all(message.as_bytes()).await?;
//                     stream
//                 }
//                 Err(e) => {
//                     let error_message = format!("can not connect to {}: {}", addr, e);
//                     log(self.time_start.clone(), &error_message).await; 
//                     return Err(e);
//                 }
//             };
//         }
//
//         // Слушаем входящие соединения
//         let listener = TcpListener::bind(self.public_addr).await?;
//         tokio::spawn(async move {
//             while let Ok((socket, _)) = listener.accept().await {
//                 tokio::spawn(handle_incoming_messages(socket));
//             }
//         });
//
//         // let listener = TcpListener::bind(self.public_addr).await?;
//         // let peers = self.peers.clone();
//         // let time_start = self.time_start.clone(); 
//         // tokio::spawn(async move {
//         //     loop {
//         //         match listener.accept().await {
//         //             Ok((mut socket, addr)) => {
//         //                 let mut buffer = [0; 1024]; // buf for read
//         //                 match socket.read(&mut buffer).await {
//         //                     Ok(_) => {
//         //                         let message = String::from_utf8_lossy(&buffer);
//         //                         let received_message = format!("Received message: {}", message);
//         //                         log(time_start.clone(), &received_message).await;
//         //                     }
//         //                     Err(e) => {
//         //                         let error_message = format!("err mes reading: {}", e);
//         //                         log(time_start.clone(), &error_message).await;
//         //                     }
//         //                 }
//         //             }
//         //             Err(e) => {
//         //                 let error_message = format!("Failed to accept connection: {}", e);
//         //                 log(time_start.clone(), &error_message).await;
//         //             }
//         //         }
//         //     }
//         // });
//
//         // let receiver = self.receiver.clone();
//         let peers = self.peers.clone();
//         let period = self.period;
//
//         let sender_clone = self.sender.clone();
//
//         let time_start = self.time_start.clone(); // clone to use in closure
//         tokio::spawn(async move {
//             loop {
//                 sleep(period).await;
//
//                 log(time_start.clone(), "tik").await;
//                 // clone peer for async
//                 let peers_guard = peers.lock().await;
//                 for peer_addr in peers_guard.peers.keys() {
//                     let msg = Message::Info("Periodic message".to_string());
//                     let peer_socket_addr = peer_addr.addr();
//                     // clone sender to use in async
//                     let sender_clone = sender_clone.clone();
//                     tokio::spawn(async move {
//                         send_message_to_peer(&peer_socket_addr, &msg, &sender_clone)
//                             .await
//                             .unwrap_or_else(|e| eprintln!("Error sending message: {}", e));
//                     });
//                 }
//             }
//         });
//
//         Ok(())
//     }
// }

trait ToSocketAddr {
    fn get_addr(&self) -> SocketAddr;
}

impl ToSocketAddr for SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        *self
    }
}

fn format_list_of_addrs<T: ToSocketAddr>(items: &Vec<T>) -> String {
    if items.len() == 0 {
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

fn log_connected_to_the_peers<T: ToSocketAddr>(peers: &Vec<T>) {
    println!("Connected to the peers at {}", format_list_of_addrs(peers));
}

async fn send_message_to_peer(
    peer_addr: &SocketAddr,
    message: &Message,
    sender: &mpsc::Sender<Message>,
) -> io::Result<()> {
    let mut stream = TcpStream::connect(peer_addr).await?;
    let serialized_message = message.serialize();
    let send_result = stream.write_all(serialized_message.as_bytes()).await;

    match send_result {
        Ok(_) => {
            let confirm_message = Message::Info(format!("Message sent to {}", peer_addr));
            if sender.send(confirm_message).await.is_err() {
                eprintln!("Failed to send confirm message to channel");
            }
        }
        Err(e) => {
            let error_message =
                Message::Info(format!("Failed to send message to {}: {}", peer_addr, e));
            if sender.send(error_message).await.is_err() {
                eprintln!("Failed to send error message to channel");
            }
            return Err(e);
        }
    }

    Ok(())
}

// async fn handle_incoming_messages(mut socket: TcpStream) {
//     let mut buf = [0; 1024]; // buf for mes
//     while let Ok(size) = socket.read(&mut buf).await {
//         if size == 0 {
//             break;
//         } // conn slose
//
//         let received_data = String::from_utf8_lossy(&buf[..size]);
//         if let Some(message) = Message::deserialize(&received_data) {
//             match message {
//                 // handle mess
//                 Message::Info(msg) => println!("Info message: {}", msg),
//                 Message::TakePeersList(msg) => println!("Peer list: {:?}", msg),
//                 _ => {}
//             }
//         }
//     }
// }

async fn handle_incoming_messages(
    mut socket: TcpStream,
    peers: Arc<Mutex<PeersStorage>>,
    sender: mpsc::Sender<Message>,
) {
    let mut buf = [0; 1024];
    while let Ok(size) = socket.read(&mut buf).await {
        if size == 0 {
            break;
        }

        if let Some(message) = Message::deserialize(&String::from_utf8_lossy(&buf[..size])) {
            match message {
                Message::GiveMeAListOfPeers => {
                    // get peers list
                    let peers_list = peers.lock().await.get_peers_list().await;
                    // redirect peers list
                    send_message_to_peer(
                        &socket.peer_addr().unwrap(),
                        &Message::TakePeersList(peers_list),
                        &sender,
                    )
                    .await
                    .unwrap();
                }
                _ => {}
            }
        }
    }
}

async fn handle_give_me_a_list_of_peers(
    sender: &mpsc::Sender<Message>,
    peers: Arc<Mutex<PeersStorage>>,
) {
    let peers_guard = peers.lock().await;
    let peer_addresses: Vec<SocketAddr> = peers_guard
        .peers
        .iter()
        .map(|(endpoint, _info)| endpoint.addr())
        .collect();
    let message = Message::TakePeersList(peer_addresses);
    if let Err(e) = sender.send(message).await {
        eprintln!("Error sending list of peers: {}", e);
    }
}
