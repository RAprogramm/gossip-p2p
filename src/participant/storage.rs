//! Network Participant Storage Management.
//!
//! This module provides functionalities for managing network participants, including storing
//! and querying participant addresses and endpoints. It supports distinguishing between known
//! and unknown participants to facilitate network communication and discovery processes.

use message_io::network::Endpoint;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Represents a storage mechanism for network participants.
///
/// This struct manages a collection of network participants, tracking their known state
/// and associated network endpoints. It allows for efficient querying and updating of
/// participant information.
#[derive(Debug)]
pub struct ParticipantsStorage<T: ParticipantEndpoint> {
    map: HashMap<T, ParticipantInfo>,
    self_pub_addr: SocketAddr,
}

/// Defines behavior for types that can be used as network endpoints.
pub trait ParticipantEndpoint {
    /// Returns the network address associated with this endpoint.
    fn addr(&self) -> SocketAddr;
}

impl ParticipantEndpoint for Endpoint {
    fn addr(&self) -> SocketAddr {
        self.addr()
    }
}

/// Represents an address associated with a network participant.
///
/// This struct encapsulates both the public socket address and the specific endpoint
/// of a network participant.
#[derive(Debug, PartialEq)]
pub struct ParticipantAddress<T: ParticipantEndpoint> {
    pub public: SocketAddr,
    pub endpoint: T,
}

/// Enumerates the possible information states of a network participant.
#[derive(Debug)]
enum ParticipantInfo {
    KnownParticipant,
    UnknownParticipant(SocketAddr),
}

impl<T: ParticipantEndpoint + std::hash::Hash + std::cmp::Eq + Clone> ParticipantsStorage<T> {
    /// Constructs a new `ParticipantsStorage`.
    ///
    /// Initializes an empty storage for managing network participants.
    ///
    /// # Parameters
    ///
    /// * `self_pub_addr` - The public address of the node owning this storage.
    pub fn new(self_pub_addr: SocketAddr) -> Self {
        Self {
            map: HashMap::new(),
            self_pub_addr,
        }
    }

    /// Determines whether a participant with the given address is known.
    ///
    /// # Parameters
    ///
    /// * `addr` - The socket address to query.
    pub fn is_known_participant(&self, addr: SocketAddr) -> bool {
        self.map.iter().any(|(endpoint, info)| match info {
            ParticipantInfo::KnownParticipant => endpoint.addr() == addr,
            ParticipantInfo::UnknownParticipant(public_addr) => *public_addr == addr,
        })
    }

    /// Adds a participant as known in the storage.
    ///
    /// # Parameters
    ///
    /// * `endpoint` - The endpoint associated with the participant to add.
    pub fn add_known_participant(&mut self, endpoint: T) {
        self.map.insert(endpoint, ParticipantInfo::KnownParticipant);
    }

    /// Removes a participant from the storage.
    ///
    /// # Parameters
    ///
    /// * `endpoint` - The endpoint associated with the participant to remove.
    pub fn drop(&mut self, endpoint: T) {
        self.map.remove(&endpoint);
    }

    /// Adds a participant as unknown in the storage.
    ///
    /// # Parameters
    ///
    /// * `endpoint` - The endpoint associated with the participant to add.
    /// * `pub_addr` - The public address of the participant.
    pub fn add_unknown_participant(&mut self, endpoint: T, pub_addr: SocketAddr) {
        self.map
            .insert(endpoint, ParticipantInfo::UnknownParticipant(pub_addr));
    }

    /// Retrieves a list of all participant addresses, including the self address.
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

    /// Retrieves a list of `ParticipantAddress` instances for communication purposes.
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

    /// Gets the public address for a given participant endpoint.
    ///
    /// # Parameters
    ///
    /// * `endpoint` - The endpoint of the participant whose address is being queried.
    pub fn get_pub_addr(&self, endpoint: &T) -> Option<SocketAddr> {
        self.map.get(endpoint).map(|founded| match founded {
            ParticipantInfo::KnownParticipant => endpoint.addr(),
            ParticipantInfo::UnknownParticipant(addr) => *addr,
        })
    }
}
