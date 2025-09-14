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
    use proptest::prelude::*;

    #[test]
    fn parse_invalid_format() {
        let err = Message::parse("UNKNOWN").unwrap_err();
        assert!(matches!(err, MessageError::InvalidFormat));
    }

    #[test]
    fn parse_invalid_address() {
        let err = Message::parse("JOIN not-an-addr").unwrap_err();
        assert!(matches!(err, MessageError::InvalidAddress(_)));
    }

    fn ipv4_addr() -> impl Strategy<Value = SocketAddr> {
        (any::<[u8; 4]>(), any::<u16>()).prop_map(|(ip, port)| SocketAddr::from((ip, port)))
    }

    proptest! {
        #[test]
        fn join_roundtrip(addr in ipv4_addr()) {
            let msg = Message::Join(addr);
            let line = msg.serialize();
            let parsed = Message::parse(&line).unwrap();
            prop_assert_eq!(parsed, msg);
        }

        #[test]
        fn text_roundtrip(text in proptest::string::string_regex("[A-Za-z0-9]{0,64}").unwrap()) {
            let msg = Message::Text(text.clone());
            let line = msg.serialize();
            let parsed = Message::parse(&line).unwrap();
            prop_assert_eq!(parsed, msg);
        }

        #[test]
        fn peers_roundtrip(addrs in proptest::collection::vec(ipv4_addr(), 0..8)) {
            let msg = Message::Peers(addrs.clone());
            let line = msg.serialize();
            let parsed = Message::parse(&line).unwrap();
            prop_assert_eq!(parsed, msg);
        }
    }
}
