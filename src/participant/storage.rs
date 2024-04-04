use message_io::network::Endpoint;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct ParticipantsStorage<T: ParticipantEndpoint> {
    map: HashMap<T, ParticipantInfo>,
    self_pub_addr: SocketAddr,
}

pub trait ParticipantEndpoint {
    fn addr(&self) -> SocketAddr;
}

impl ParticipantEndpoint for Endpoint {
    fn addr(&self) -> SocketAddr {
        self.addr()
    }
}

#[derive(Debug, PartialEq)]
pub struct ParticipantAddress<T: ParticipantEndpoint> {
    pub public: SocketAddr,
    pub endpoint: T,
}

#[derive(Debug)]
enum ParticipantInfo {
    KnownParticipant,
    UnknownParticipant(SocketAddr),
}

impl<T: ParticipantEndpoint + std::hash::Hash + std::cmp::Eq + Clone> ParticipantsStorage<T> {
    pub fn new(self_pub_addr: SocketAddr) -> Self {
        Self {
            map: HashMap::new(),
            self_pub_addr,
        }
    }
    pub fn is_known_participant(&self, addr: SocketAddr) -> bool {
        self.map.iter().any(|(endpoint, info)| match info {
            ParticipantInfo::KnownParticipant => endpoint.addr() == addr,
            ParticipantInfo::UnknownParticipant(public_addr) => *public_addr == addr,
        })
    }

    pub fn add_known_participant(&mut self, endpoint: T) {
        self.map.insert(endpoint, ParticipantInfo::KnownParticipant);
    }

    pub fn drop(&mut self, endpoint: T) {
        self.map.remove(&endpoint);
    }

    pub fn add_unknown_participant(&mut self, endpoint: T, pub_addr: SocketAddr) {
        self.map
            .insert(endpoint, ParticipantInfo::UnknownParticipant(pub_addr));
    }

    pub fn get_participants_list(&self) -> Vec<SocketAddr> {
        let mut list: Vec<SocketAddr> = Vec::with_capacity(self.map.len() + 1);
        list.push(self.self_pub_addr);
        self.map
            .iter()
            .map(|(endpoint, info)| match info {
                ParticipantInfo::KnownParticipant => endpoint.addr(),
                ParticipantInfo::UnknownParticipant(public_addr) => *public_addr,
            })
            .for_each(|addr| {
                list.push(addr);
            });

        list
    }

    pub fn receivers(&self) -> Vec<ParticipantAddress<T>> {
        self.map
            .iter()
            .map(|(endpoint, info)| {
                let public = match info {
                    ParticipantInfo::KnownParticipant => endpoint.addr(),
                    ParticipantInfo::UnknownParticipant(public_addr) => *public_addr,
                };
                ParticipantAddress {
                    endpoint: endpoint.clone(),
                    public,
                }
            })
            .collect()
    }

    pub fn get_pub_addr(&self, endpoint: &T) -> Option<SocketAddr> {
        self.map.get(endpoint).map(|founded| match founded {
            ParticipantInfo::KnownParticipant => endpoint.addr(),
            ParticipantInfo::UnknownParticipant(addr) => *addr,
        })
    }
}
