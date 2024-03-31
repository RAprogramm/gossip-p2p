use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Storage for peers - old ones and new ones
pub struct PeersStorage<T: PeerEndpoint + Clone + std::hash::Hash + Eq> {
    map: Arc<Mutex<HashMap<T, PeerInfo>>>,
    self_pub_addr: SocketAddr,
}

/// Trait that generalizes endpoint behavior - for tests and abstraction
pub trait PeerEndpoint {
    fn addr(&self) -> SocketAddr;
}

#[derive(Debug, PartialEq, Clone)]
pub struct PeerAddr<T: PeerEndpoint> {
    pub public: SocketAddr,
    pub endpoint: T,
}

enum PeerInfo {
    OldOne,
    NewOne(SocketAddr),
}

impl<T: PeerEndpoint + std::hash::Hash + Eq + Clone + Send + Sync + 'static> PeersStorage<T> {
    pub fn new(self_pub_addr: SocketAddr) -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
            self_pub_addr,
        }
    }

    pub async fn add_old_one(&self, endpoint: T) {
        let mut map = self.map.lock().await;
        map.insert(endpoint, PeerInfo::OldOne);
    }

    pub async fn add_new_one(&self, endpoint: T, pub_addr: SocketAddr) {
        let mut map = self.map.lock().await;
        map.insert(endpoint, PeerInfo::NewOne(pub_addr));
    }

    pub async fn drop(&self, endpoint: T) {
        let mut map = self.map.lock().await;
        map.remove(&endpoint);
    }

    pub async fn get_peers_list(&self) -> Vec<SocketAddr> {
        let map = self.map.lock().await;
        let mut list: Vec<SocketAddr> = Vec::with_capacity(map.len() + 1);
        list.push(self.self_pub_addr);
        map.iter()
            .map(|(endpoint, info)| match info {
                PeerInfo::OldOne => endpoint.addr(),
                PeerInfo::NewOne(public_addr) => *public_addr,
            })
            .for_each(|addr| list.push(addr));

        list
    }

    pub async fn receivers(&self) -> Vec<PeerAddr<T>> {
        let map = self.map.lock().await;
        map.iter()
            .map(|(endpoint, info)| {
                let public = match info {
                    PeerInfo::OldOne => endpoint.addr(),
                    PeerInfo::NewOne(public_addr) => *public_addr,
                };
                PeerAddr {
                    endpoint: endpoint.clone(),
                    public,
                }
            })
            .collect()
    }

    pub async fn get_pub_addr(&self, endpoint: &T) -> Option<SocketAddr> {
        let map = self.map.lock().await;
        map.get(endpoint).map(|info| match info {
            PeerInfo::OldOne => endpoint.addr(),
            PeerInfo::NewOne(addr) => *addr,
        })
    }
}
