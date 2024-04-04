use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PublicAddress(SocketAddr),
    PushPeersList,
    PullPeersList(Vec<SocketAddr>),
    Text(String),
}
