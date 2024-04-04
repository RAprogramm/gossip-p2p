//! Network Communication Utilities.
//!
//! This module provides essential utilities for network communication within a distributed
//! system. It focuses on simplifying the handling of socket addresses and the serialization
//! and sending of messages across the network. The module facilitates the conversion between
//! different types that represent network addresses and abstracts the process of message
//! serialization and dispatching to network participants.
//!
//! ## Features
//!
//! - **Address Conversion**: A trait `ToSocketAddr` and its implementations allow for flexible
//!   conversion from various types to `SocketAddr`, streamlining operations that require
//!   network addresses.
//! - **Address Formatting**: `format_list_of_addrs` function for generating human-readable
//!   strings from lists of addresses, aiding in logging and diagnostics.
//! - **Message Sending**: `send_message` function encapsulates the serialization of message
//!   content and network transmission, leveraging `message-io` for efficient asynchronous
//!   communication.
//!
//! These utilities are designed to work with the `message-io` library, providing a high-level
//! abstraction for network message handling that can be easily integrated into applications
//! requiring network communication capabilities.

use std::net::SocketAddr;

use message_io::network::Endpoint;
use message_io::node::NodeHandler;

use crate::participant::message::Message;

/// Trait for obtaining a `SocketAddr` from various types.
///
/// This trait abstracts over different types that can be converted into a `SocketAddr`,
/// simplifying address handling in network operations.
pub trait ToSocketAddr {
    /// Returns the `SocketAddr` associated with the implementing type.
    fn get_addr(&self) -> SocketAddr;
}

/// Implementation of `ToSocketAddr` for `Endpoint`.
impl ToSocketAddr for Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

/// Implementation of `ToSocketAddr` for a reference to `Endpoint`.
impl ToSocketAddr for &Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

/// Implementation of `ToSocketAddr` for `SocketAddr`.
impl ToSocketAddr for SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        *self
    }
}

/// Implementation of `ToSocketAddr` for a reference to `SocketAddr`.
impl ToSocketAddr for &SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        **self
    }
}

/// Formats a list of items that implement `ToSocketAddr` into a string.
///
/// This function takes a slice of items that can be converted to `SocketAddr` and formats
/// them into a human-readable string representation. It is particularly useful for logging
/// and displaying participant addresses in the network.
///
/// # Parameters
///
/// - `items`: A slice of items implementing `ToSocketAddr`.
///
/// # Returns
///
/// A string representation of the list of addresses.
pub fn format_list_of_addrs<T: ToSocketAddr>(items: &[T]) -> String {
    if items.is_empty() {
        "[no one]".to_owned()
    } else {
        let joined = items
            .iter()
            .map(|x| format!("\"{}\"", ToSocketAddr::get_addr(x)))
            .collect::<Vec<String>>()
            .join(", ");

        format!("[{}]", joined)
    }
}

/// Sends a serialized message to a specified endpoint using a `NodeHandler`.
///
/// This function serializes a given message and sends it to the specified endpoint
/// via the network managed by the `NodeHandler`. It encapsulates the serialization
/// and network sending steps, streamlining message dispatch.
///
/// # Parameters
///
/// - `handler`: A mutable reference to a `NodeHandler` for managing network operations.
/// - `to`: The target `Endpoint` to send the message to.
/// - `msg`: A reference to the message to be sent.
pub fn send_message(handler: &mut NodeHandler<()>, to: Endpoint, msg: &Message) {
    let output_data = bincode::serialize(msg).unwrap();
    handler.network().send(to, &output_data);
}
