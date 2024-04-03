use message_io::network::Endpoint;
use std::collections::HashMap;
use std::net::SocketAddr;

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

enum PeerInfo {
    KnownPeer,
    NewOne(SocketAddr),
}

impl<T: PeerEndpoint + std::hash::Hash + std::cmp::Eq + Clone> PeersStorage<T> {
    pub fn new(self_pub_addr: SocketAddr) -> Self {
        Self {
            map: HashMap::new(),
            self_pub_addr,
        }
    }

    pub fn add_old_one(&mut self, endpoint: T) {
        self.map.insert(endpoint, PeerInfo::KnownPeer);
    }

    // pub fn add_new_one(&mut self, endpoint: T, pub_addr: SocketAddr) {
    //     self.map.insert(endpoint, PeerInfo::NewOne(pub_addr));
    // }

    pub fn add_new_one(&mut self, endpoint: T, pub_addr: SocketAddr) {
        // Не добавляем, если такой адрес уже присутствует
        if !self
            .map
            .values()
            .any(|info| matches!(info, PeerInfo::NewOne(addr) if addr == &pub_addr))
        {
            self.map.insert(endpoint, PeerInfo::NewOne(pub_addr));
        }
    }

    pub fn remove_peer(&mut self, endpoint: T) {
        self.map.remove(&endpoint);
    }

    pub fn get_peers_list(&self) -> Vec<SocketAddr> {
        let mut list: Vec<SocketAddr> = Vec::with_capacity(self.map.len() + 1);
        list.push(self.self_pub_addr);
        self.map
            .iter()
            .map(|(endpoint, info)| match info {
                PeerInfo::KnownPeer => endpoint.addr(),
                PeerInfo::NewOne(public_addr) => *public_addr,
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
                    PeerInfo::NewOne(public_addr) => *public_addr,
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
            PeerInfo::NewOne(addr) => *addr,
        })
    }
}
