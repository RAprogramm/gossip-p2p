//! Message types for peer-to-peer communication.
//!
//! The protocol is intentionally simple and text based. Each message is a single
//! line terminated by a newline character. In production systems this module can
//! be replaced by a binary protocol using crates such as `serde` and `bincode`
//! for efficient serialization.
//!
//! # Examples
//!
//! ```
//! use gossip_p2p::message::{Message, MessageError};
//! use std::net::SocketAddr;
//!
//! let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
//! let msg = Message::Join(addr);
//! let line = msg.serialize();
//! assert_eq!(Message::parse(&line).unwrap(), msg);
//! ```

use std::fmt;
use std::net::SocketAddr;

/// Messages exchanged between peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// A peer announces its address to another peer.
    Join(SocketAddr),
    /// A list of known peers.
    Peers(Vec<SocketAddr>),
    /// Arbitrary text data.
    Text(String),
}

/// Errors that can occur during message parsing.
#[derive(Debug)]
pub enum MessageError {
    /// The message did not follow the expected format.
    InvalidFormat,
    /// Failed to parse a socket address.
    InvalidAddress(std::net::AddrParseError),
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat => write!(f, "invalid message format"),
            Self::InvalidAddress(e) => write!(f, "invalid address: {}", e),
        }
    }
}

impl std::error::Error for MessageError {}

impl Message {
    /// Serialize the message into a protocol line.
    pub fn serialize(&self) -> String {
        match self {
            Self::Join(addr) => format!("JOIN {}\n", addr),
            Self::Peers(list) => {
                let mut line = String::from("PEERS ");
                for (i, addr) in list.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                    }
                    line.push_str(&addr.to_string());
                }
                line.push('\n');
                line
            }
            Self::Text(text) => format!("TEXT {}\n", text),
        }
    }

    /// Parse a protocol line into a message.
    pub fn parse(line: &str) -> Result<Self, MessageError> {
        let trimmed = line.trim();
        let mut parts = trimmed.splitn(2, ' ');
        let command = parts.next().ok_or(MessageError::InvalidFormat)?;
        let rest = parts.next().unwrap_or("").trim();
        match command {
            "JOIN" => {
                let addr = rest.parse().map_err(MessageError::InvalidAddress)?;
                Ok(Self::Join(addr))
            }
            "PEERS" => {
                if rest.is_empty() {
                    return Ok(Self::Peers(Vec::new()));
                }
                let mut addrs = Vec::new();
                for item in rest.split(',') {
                    let addr = item.trim().parse().map_err(MessageError::InvalidAddress)?;
                    addrs.push(addr);
                }
                Ok(Self::Peers(addrs))
            }
            "TEXT" => Ok(Self::Text(rest.to_string())),
            _ => Err(MessageError::InvalidFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_roundtrip() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let messages = vec![
            Message::Join(addr),
            Message::Peers(vec![addr]),
            Message::Text("hi".to_string()),
        ];
        for msg in messages {
            let line = msg.serialize();
            let parsed = Message::parse(&line).unwrap();
            assert_eq!(msg, parsed);
        }
    }

    #[test]
    fn parse_invalid() {
        let err = Message::parse("UNKNOWN").unwrap_err();
        matches!(err, MessageError::InvalidFormat);
    }
}
