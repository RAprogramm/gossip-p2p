//! Simple peer implementation using only the Rust standard library.
//!
//! The peer communicates with other peers over TCP using a tiny text protocol
//! defined in the [`message`](crate::message) module. Each instance periodically sends a gossip
//! message to all known peers.
//!
//! This implementation is deliberately minimal and blocking to avoid external
//! dependencies. In a production setting consider replacing it with an
//! asynchronous runtime such as `tokio` or a low level framework like `mio` for
//! improved scalability.

use crate::message::Message;
use crate::printer::{init as printer_init, print_event};

use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Peer participating in the gossip network.
pub struct Peer {
    addr: SocketAddr,
    period: Duration,
    participants: Arc<Mutex<HashSet<SocketAddr>>>,
    start_time: Arc<Instant>,
}

impl Peer {
    /// Create a new peer.
    ///
    /// # Parameters
    /// * `period` - Seconds between gossip messages.
    /// * `port` - TCP port to listen on.
    /// * `connect` - Optional address of an existing peer.
    pub fn new(period: u64, port: u16, connect: Option<String>) -> io::Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let participants = Arc::new(Mutex::new(HashSet::from([addr])));
        let start_time = printer_init(&addr);
        let peer = Self {
            addr,
            period: Duration::from_secs(period),
            participants,
            start_time,
        };
        if let Some(remote) = connect {
            let remote_addr: SocketAddr = remote
                .parse()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            peer.connect_and_sync(remote_addr)?;
        }
        Ok(peer)
    }

    /// Run the peer. This method blocks forever.
    pub fn run(&self) -> io::Result<()> {
        self.spawn_sender();
        let listener = TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let participants = Arc::clone(&self.participants);
                    let start = Arc::clone(&self.start_time);
                    let me = self.addr;
                    thread::spawn(move || {
                        let _ = handle_connection(stream, participants, me, start);
                    });
                }
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }
        Ok(())
    }

    fn connect_and_sync(&self, addr: SocketAddr) -> io::Result<()> {
        if addr == self.addr {
            return Ok(());
        }
        {
            let mut lock = self
                .participants
                .lock()
                .map_err(|_| io::Error::other("lock poisoned"))?;
            if !lock.insert(addr) {
                return Ok(());
            }
        }
        let mut stream = TcpStream::connect(addr)?;
        stream.write_all(Message::Join(self.addr).serialize().as_bytes())?;
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line)?;
        if let Ok(Message::Peers(peers)) = Message::parse(line.trim()) {
            for peer in peers {
                let _ = self.connect_and_sync(peer);
            }
        }
        Ok(())
    }

    fn spawn_sender(&self) {
        let participants = Arc::clone(&self.participants);
        let start = Arc::clone(&self.start_time);
        let me = self.addr;
        let period = self.period;
        thread::spawn(move || loop {
            thread::sleep(period);
            let rnd = random_number();
            let text = format!("gossip {}", rnd);
            let msg = Message::Text(text).serialize();
            let peers: Vec<SocketAddr> = match participants
                .lock()
                .map_err(|_| io::Error::other("lock poisoned"))
            {
                Ok(guard) => guard.iter().copied().filter(|a| *a != me).collect(),
                Err(_) => Vec::new(),
            };
            for addr in peers {
                if let Ok(mut s) = TcpStream::connect(addr) {
                    let _ = s.write_all(msg.as_bytes());
                }
            }
            print_event(start.clone(), "sent gossip");
        });
    }
}

fn handle_connection(
    stream: TcpStream,
    participants: Arc<Mutex<HashSet<SocketAddr>>>,
    me: SocketAddr,
    start: Arc<Instant>,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    match Message::parse(line.trim()) {
        Ok(Message::Join(addr)) => {
            {
                let mut lock = participants
                    .lock()
                    .map_err(|_| io::Error::other("lock poisoned"))?;
                lock.insert(addr);
                let peers: Vec<SocketAddr> = lock.iter().copied().collect();
                let response = Message::Peers(peers).serialize();
                drop(lock);
                reader.get_mut().write_all(response.as_bytes())?;
            }
            broadcast_join(&participants, me, addr);
        }
        Ok(Message::Peers(addrs)) => {
            let mut lock = participants
                .lock()
                .map_err(|_| io::Error::other("lock poisoned"))?;
            for addr in addrs {
                lock.insert(addr);
            }
        }
        Ok(Message::Text(text)) => {
            let formatted = format!("received: {}", text);
            print_event(start, &formatted);
        }
        Err(e) => eprintln!("Failed to parse message: {}", e),
    }
    Ok(())
}

fn broadcast_join(
    participants: &Arc<Mutex<HashSet<SocketAddr>>>,
    me: SocketAddr,
    new_peer: SocketAddr,
) {
    let peers: Vec<SocketAddr> = match participants
        .lock()
        .map_err(|_| io::Error::other("lock poisoned"))
    {
        Ok(guard) => guard
            .iter()
            .copied()
            .filter(|a| *a != me && *a != new_peer)
            .collect(),
        Err(_) => Vec::new(),
    };
    let msg = Message::Join(new_peer).serialize();
    for addr in peers {
        if let Ok(mut s) = TcpStream::connect(addr) {
            let _ = s.write_all(msg.as_bytes());
        }
    }
}

fn random_number() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_number_changes() {
        let a = random_number();
        let b = random_number();
        assert_ne!(a, b);
    }
}
