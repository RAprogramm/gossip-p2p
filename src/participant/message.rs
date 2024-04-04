//! Network Communication Messages Module.
//!
//! This module defines the core message types used for network communication in a distributed
//! system. It leverages the `serde` crate for serialization and deserialization of message
//! content, facilitating easy and efficient transmission over the network.
//!
//! ## Overview
//!
//! The `Message` enum encapsulates the various kinds of messages that can be exchanged between
//! participants in the network. These messages enable fundamental operations such as sharing
//! public addresses for direct connections, requesting and sharing lists of known participants
//! to maintain an updated network topology, and sending generic text messages for communication
//! or data transfer purposes.
//!
//! The design of these message types aims to support a scalable and dynamic network environment
//! where participants can join, communicate, and leave seamlessly. By standardizing the message
//! format and ensuring compatibility with the `serde` serialization framework, this module
//! contributes to the robustness and extensibility of the network communication system.
//!
//! ## Message Types
//!
//! - `PublicAddress`: Shares the sender's public network address.
//! - `PushParticipantsList`: Requests the receiver to share its list of known participants.
//! - `PullParticipantsList`: Shares a list of known participants with the receiver.
//! - `Text`: Sends a free-form text message, allowing for versatile communication.
//!
//! Each message type is designed to fulfill specific roles within the network's communication
//! protocol, ensuring that participants can effectively discover each other, establish connections,
//! and exchange information.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Defines the types of messages that can be sent between network participants.
///
/// This enum is used for serializing and deserializing message content for network communication.
/// Each variant represents a specific kind of message that can be exchanged in the network,
/// facilitating various aspects of network interaction and management.
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Represents a message containing the public address of a participant.
    ///
    /// This message type is typically used to share a participant's address with others,
    /// allowing them to update their list of known participants and establish direct connections.
    PublicAddress(SocketAddr),

    /// Indicates a request to push the current list of known participant addresses.
    ///
    /// When a participant receives this message, it is expected to respond with a
    /// `PullParticipantsList` message containing its list of known participants. This mechanism
    /// is used to synchronize participants' knowledge of the network topology.
    PushParticipantsList,

    /// Contains a list of participant addresses.
    ///
    /// This message type is sent in response to a `PushParticipantsList` request or proactively
    /// to share the sender's list of known participants. Receiving participants can use the
    /// information to update their own lists and potentially establish connections with new peers.
    PullParticipantsList(Vec<SocketAddr>),

    /// Represents a text message being sent between participants.
    ///
    /// This variant is used for exchanging arbitrary text messages, supporting a wide range of
    /// communication needs, from simple notifications to complex data payloads encoded as strings.
    Text(String),
}
