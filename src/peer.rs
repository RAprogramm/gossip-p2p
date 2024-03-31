use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::logger::{init as logger_init, log};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Endpoint(SocketAddr);

pub struct PeersStorage {
    peers: HashMap<Endpoint, String>, // Пример хранилища для пиров
}

pub struct Peer {
    peers: Arc<Mutex<PeersStorage>>,
    public_addr: SocketAddr,
    period: Duration,
    connect: Option<String>,
    time_start: Arc<Instant>, // Добавляем поле для времени запуска
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

        Ok(Self {
            peers: Arc::new(Mutex::new(peers_storage)),
            public_addr,
            period: Duration::from_secs(period),
            connect,
            time_start, // Инициализируем поле времени запуска
        })
    }

    pub async fn run(&self) -> Result<(), io::Error> {
        if let Some(addr) = &self.connect {
            let stream = match TcpStream::connect(addr).await {
                Ok(mut stream) => {
                    let connect_message = format!("Успешное подключение к {}", addr);
                    log(self.time_start.clone(), &connect_message).await; // Используем логгер и передаем время запуска
                                                                          // Отправляем информацию о нашем порте серверу
                    let local_addr = stream.local_addr()?;
                    let message = format!("My public port is: {}", self.public_addr.port());
                    stream.write_all(message.as_bytes()).await?;
                    stream
                }
                Err(e) => {
                    let error_message = format!("Невозможно подключиться к {}: {}", addr, e);
                    log(self.time_start.clone(), &error_message).await; // Используем логгер и передаем время запуска
                    return Err(e);
                }
            };
        }

        let listener = TcpListener::bind(self.public_addr).await?;
        let peers = self.peers.clone();
        let time_start = self.time_start.clone(); // Копируем время запуска для использования внутри closure
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut socket, addr)) => {
                        let mut buffer = [0; 1024]; // Буфер для чтения сообщения
                        match socket.read(&mut buffer).await {
                            Ok(_) => {
                                let message = String::from_utf8_lossy(&buffer);
                                let received_message = format!("Received message: {}", message);
                                log(time_start.clone(), &received_message).await;
                                // Используем логгер и передаем время запуска
                                // Здесь можно добавить логику для ассоциации адреса и порта клиента
                            }
                            Err(e) => {
                                let error_message = format!("Ошибка при чтении сообщения: {}", e);
                                log(time_start.clone(), &error_message).await;
                                // Используем логгер и передаем время запуска
                            }
                        }
                    }
                    Err(e) => {
                        let error_message = format!("Failed to accept connection: {}", e);
                        log(time_start.clone(), &error_message).await;
                        // Используем логгер и передаем время запуска
                    }
                }
            }
        });

        let period = self.period;
        tokio::spawn(async move {
            loop {
                sleep(period).await;
                // Здесь код для отправки сообщений
            }
        });

        Ok(())
    }
}

trait ToSocketAddr {
    fn get_addr(&self) -> SocketAddr;
}

impl ToSocketAddr for SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        *self
    }
}

// fn log_my_address(addr: &SocketAddr) {
//     println!("My address is \"{}\"", addr);
// }

// fn log_connected_to_the_peers<T: ToSocketAddr>(peers: &Vec<T>) {
//     println!("Connected to the peers at {}", format_list_of_addrs(peers));
// }
