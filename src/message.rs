use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub enum Message {
    MyPubAddr(SocketAddr),
    GiveMeAListOfPeers,
    TakePeersList(Vec<SocketAddr>),
    Info(String),
}

// impl Message {
//     pub fn serialize(&self) -> String {
//         match self {
//             Message::MyPubAddr(addr) => format!("MyPubAddr:{}", addr),
//             Message::GiveMeAListOfPeers => "GiveMeAListOfPeers".to_string(),
//             Message::TakePeersList(peers) => {
//                 let peers_str = peers
//                     .iter()
//                     .map(|addr| addr.to_string())
//                     .collect::<Vec<_>>()
//                     .join(",");
//                 format!("TakePeersList:{}", peers_str)
//             }
//             Message::Info(msg) => format!("Info:{}", msg),
//         }
//     }
//
//     pub fn deserialize(input: &str) -> Option<Message> {
//         let parts: Vec<&str> = input.splitn(2, ':').collect();
//         match parts[0] {
//             "MyPubAddr" => parts.get(1)?.parse().ok().map(Message::MyPubAddr),
//             "GiveMeAListOfPeers" => Some(Message::GiveMeAListOfPeers),
//             "TakePeersList" => {
//                 let peers = parts
//                     .get(1)?
//                     .split(',')
//                     .filter_map(|s| s.parse().ok())
//                     .collect();
//                 Some(Message::TakePeersList(peers))
//             }
//             "Info" => Some(Message::Info(parts.get(1)?.to_string())),
//             _ => None,
//         }
//     }
// }
