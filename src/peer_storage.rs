use message_io::network::Endpoint;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct PeersStorage<T: PeerEndpoint> {
    map: HashMap<T, PeerInfo>,
    self_pub_addr: SocketAddr,
}

pub trait PeerEndpoint {
    fn addr(&self) -> SocketAddr;
}

impl PeerEndpoint for Endpoint {
    fn addr(&self) -> SocketAddr {
        self.addr()
    }
}

#[derive(Debug, PartialEq)]
pub struct PeerAddr<T: PeerEndpoint> {
    pub public: SocketAddr,
    pub endpoint: T,
}

#[derive(Debug)]
enum PeerInfo {
    KnownPeer,
    UnknownPeer(SocketAddr),
}

impl<T: PeerEndpoint + std::hash::Hash + std::cmp::Eq + Clone> PeersStorage<T> {
    pub fn new(self_pub_addr: SocketAddr) -> Self {
        Self {
            map: HashMap::new(),
            self_pub_addr,
        }
    }
    pub fn is_known_peer(&self, addr: SocketAddr) -> bool {
        self.map.iter().any(|(endpoint, info)| match info {
            PeerInfo::KnownPeer => endpoint.addr() == addr,
            PeerInfo::UnknownPeer(public_addr) => *public_addr == addr,
        })
    }

    pub fn add_known_peer(&mut self, endpoint: T) {
        self.map.insert(endpoint, PeerInfo::KnownPeer);
    }

    pub fn drop(&mut self, endpoint: T) {
        self.map.remove(&endpoint);
    }

    pub fn add_unknown_peer(&mut self, endpoint: T, pub_addr: SocketAddr) {
        self.map.insert(endpoint, PeerInfo::UnknownPeer(pub_addr));
    }

    pub fn get_peers_list(&self) -> Vec<SocketAddr> {
        let mut list: Vec<SocketAddr> = Vec::with_capacity(self.map.len() + 1);
        list.push(self.self_pub_addr);
        self.map
            .iter()
            .map(|(endpoint, info)| match info {
                PeerInfo::KnownPeer => endpoint.addr(),
                PeerInfo::UnknownPeer(public_addr) => *public_addr,
            })
            .for_each(|addr| {
                list.push(addr);
            });

        list
    }

    pub fn receivers(&self) -> Vec<PeerAddr<T>> {
        self.map
            .iter()
            .map(|(endpoint, info)| {
                let public = match info {
                    PeerInfo::KnownPeer => endpoint.addr(),
                    PeerInfo::UnknownPeer(public_addr) => *public_addr,
                };
                PeerAddr {
                    endpoint: endpoint.clone(),
                    public,
                }
            })
            .collect()
    }

    pub fn get_pub_addr(&self, endpoint: &T) -> Option<SocketAddr> {
        self.map.get(endpoint).map(|founded| match founded {
            PeerInfo::KnownPeer => endpoint.addr(),
            PeerInfo::UnknownPeer(addr) => *addr,
        })
    }
}
