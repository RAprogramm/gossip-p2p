//! Network Participant Management and Communication Module.
//!
//! This module provides the fundamental functionalities required for managing network
//! participants within a distributed system. It encapsulates the behaviors and actions
//! of a network participant, including the ability to connect to peers, send and receive
//! messages, and maintain an up-to-date list of known participants.
//!
//! Key functionalities include:
//! - Establishing connections with other network participants.
//! - Periodically sending random messages to all known participants.
//! - Handling incoming network events such as new connections, message receipts, and
//!   disconnections.
//! - Dynamically updating the list of known participants based on network interactions.
//!
//! This module leverages `message-io` for network communication, providing an asynchronous,
//! event-driven architecture that facilitates efficient message handling. The use of
//! `Arc<Mutex<...>>` for shared state management ensures thread-safe operations across
//! the different components of the network system.

use crate::printer::{init as logger_init, print_event};

use super::message::Message;
use super::storage::{ParticipantAddress, ParticipantsStorage};
use super::utils::{format_list_of_addrs, send_message};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self, NodeHandler, NodeListener};
use rand::Rng;

use std::io::{self};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Represents a participant in the network.
///
/// This struct encapsulates all the necessary information and functionality
/// for a participant within the network, including its node handler for network
/// operations, its public address, and the storage for other participants.
pub struct Participant {
    node_handler: Arc<Mutex<NodeHandler<()>>>,
    node_listener: Option<NodeListener<()>>,
    public_addr: SocketAddr,
    period: u32,
    connect: Option<String>,
    participants: Arc<Mutex<ParticipantsStorage<Endpoint>>>,
    time_start: Arc<Instant>,
}

impl Participant {
    /// Constructs a new `Participant`.
    ///
    /// Sets up the network node and starts listening on the specified port.
    /// Also initializes the participants storage and records the start time.
    ///
    /// # Parameters
    ///
    /// - `period`: The interval in seconds between each random message broadcast.
    /// - `port`: The port number on which this node will listen for incoming connections.
    /// - `connect`: An optional address of another node to initially connect to.
    ///
    /// # Returns
    ///
    /// An `io::Result<Self>` indicating success or failure.
    pub fn new(period: u32, port: u32, connect: Option<String>) -> io::Result<Self> {
        let (handler, listener) = node::split::<()>();

        let listen_addr = format!("127.0.0.1:{}", port);
        let (_, public_addr) = handler
            .network()
            .listen(Transport::FramedTcp, listen_addr)?;

        let time_start = logger_init(&public_addr);

        Ok(Self {
            public_addr,
            node_handler: Arc::new(Mutex::new(handler)),
            node_listener: Some(listener),
            connect,
            period,
            participants: Arc::new(Mutex::new(ParticipantsStorage::new(public_addr))),
            time_start,
        })
    }

    /// Starts the participant's network operations.
    ///
    /// This method initiates the participant's network activities by optionally connecting to another
    /// network participant, starting the periodic message sending routine, and listening for incoming
    /// network events. It handles new connections, processes incoming messages, and manages
    /// disconnections.
    ///
    /// # Behavior
    ///
    /// 1. **Initial Connection**: If an initial connection address is provided (`self.connect`),
    ///    attempts to connect to it and register the connection.
    ///
    /// 2. **Periodic Messaging**: Launches a separate thread to send random messages at regular intervals
    ///    defined by `self.period`.
    ///
    /// 3. **Event Listening**: Enters a loop to listen for and handle `NetEvent` occurrences, such as
    ///    accepting new connections, receiving messages, and handling disconnections.
    ///
    /// # Event Handling
    ///
    /// - **NetEvent::Accepted**: Triggered when a new incoming connection is accepted.
    ///
    /// - **NetEvent::Connected**: Triggered when a connection attempt is either successful or fails.
    ///    On success, registers the new participant and sends initial synchronization messages.
    ///
    /// - **NetEvent::Message**: Triggered upon receiving a message. It deserializes the message
    ///    and processes it according to its type.
    ///
    /// - **NetEvent::Disconnected**: Triggered when a connection is lost. Removes the disconnected
    ///    participant from the list of known participants.
    ///
    /// # Note
    ///
    /// This method constitutes the participant's main event loop, where all network activities are
    /// centralized. It leverages `message-io` for asynchronous event-driven communication.
    pub fn run(mut self) {
        // Attempt initial connection if an address is provided.
        if let Some(addr) = &self.connect {
            let handler = self.node_handler.lock().unwrap();

            match handler.network().connect(Transport::FramedTcp, addr) {
                Ok((endpoint, _)) => {
                    let mut participants = self.participants.lock().unwrap();
                    participants.add_known_participant(endpoint);
                }
                Err(_) => {
                    println!("Failed to connect to {}", &addr);
                }
            }
        }

        // Start sending random messages at the specified periodic interval.
        self.sending_random_message();

        // Listen for and handle network events.
        if let Some(node_listener) = self.node_listener.take() {
            node_listener.for_each(move |event| match event.network() {
                NetEvent::Accepted(_, _) => {}
                NetEvent::Connected(endpoint, established) => {
                    if established {
                        self.connected(endpoint)
                    } else {
                        println!("Can not connect to {}", endpoint.addr());
                        std::process::exit(1);
                    }
                }
                NetEvent::Message(message_sender, input_data) => {
                    let message: Message = bincode::deserialize(input_data).unwrap();
                    self.network_messages(message_sender, message)
                }

                NetEvent::Disconnected(endpoint) => {
                    let mut participants = self.participants.lock().unwrap();
                    ParticipantsStorage::drop(&mut participants, endpoint);
                }
            });
        }
    }

    /// Handles incoming network messages directed to this participant.
    ///
    /// This method decodes and processes various types of messages that can be received from other
    /// participants in the network. It acts on the message based on its type, performing actions
    /// such as updating the participants list, responding with a list of known participants,
    /// or logging received text messages.
    ///
    /// # Parameters
    ///
    /// - `message_sender`: The `Endpoint` representing the source of the message. This is used
    ///   for identifying the sender and potentially responding.
    /// - `message`: The `Message` enum containing the message received. This enum encapsulates
    ///   different types of messages that can be processed by this function.
    ///
    /// # Supported Message Types
    ///
    /// - `Message::PublicAddress`: Adds the sender's public address to the list of unknown participants
    ///   if it is not already known.
    /// - `Message::PushParticipantsList`: Responds to the sender with a list of known participant
    ///   addresses.
    /// - `Message::PullParticipantsList`: Updates the local list of participants with the addresses
    ///   received in the message.
    /// - `Message::Text`: Logs a received text message along with the sender's address.
    fn network_messages(&self, message_sender: Endpoint, message: Message) {
        match message {
            // A public address message contains the sender's address.
            // This address is added to the list of participants if it is not already known.
            Message::PublicAddress(pub_addr) => {
                let mut participants = self.participants.lock().unwrap();
                participants.add_unknown_participant(message_sender, pub_addr);
            }

            // A request to push the participants list triggers a response with the known participant
            // addresses. This helps newly joined participants to learn about existing ones.
            Message::PushParticipantsList => {
                let list = {
                    let participants = self.participants.lock().unwrap();
                    participants.get_participants_list()
                };
                let msg = Message::PullParticipantsList(list);
                send_message(&mut self.node_handler.lock().unwrap(), message_sender, &msg);
            }

            // When a list of participants is received, update the local storage to include any new
            // addresses. This ensures the participant is aware of other peers in the network.
            Message::PullParticipantsList(addrs) => {
                self.pull_participants_list(message_sender, addrs)
            }

            // For text messages, log the received message along with the sender's address.
            // This is useful for debugging and monitoring the flow of messages.
            Message::Text(text) => {
                let pub_addr = self
                    .participants
                    .lock()
                    .unwrap()
                    .get_pub_addr(&message_sender)
                    .unwrap();

                let formatted_msg = format!("Received message [{}] from \"{}\"", &text, &pub_addr);
                print_event(self.time_start.clone(), &formatted_msg);
            }
        }
    }

    /// Periodically sends a random text message to all known participants.
    ///
    /// This method spawns a new thread that wakes up at regular intervals specified by `self.period`.
    /// Each time it wakes up, it constructs a random message and sends it to all participants
    /// currently known to this instance. The message includes a randomized number to demonstrate
    /// variability and potential for custom message content.
    ///
    /// # Notes
    ///
    /// - The method clones several `Arc`-wrapped resources to move them safely into the thread.
    /// - Uses a `Mutex` to safely access shared resources across threads.
    /// - Demonstrates the use of `thread::sleep` for periodic execution within a spawned thread.
    fn sending_random_message(&self) {
        // Convert the period into a `Duration` for the sleep call.
        let tick_duration = Duration::from_secs(self.period as u64);

        // Clone `Arc`-wrapped resources to move into the thread. This increases the reference count
        // safely without violating Rust's ownership rules.
        let participants_clone = Arc::clone(&self.participants);
        let handler_clone = Arc::clone(&self.node_handler);
        let clone_start_time = self.time_start.clone();

        // Spawn a new thread to handle the periodic sending of messages.
        thread::spawn(move || loop {
            // Sleep for the specified period.
            thread::sleep(tick_duration);

            // Lock the mutex to access participants. This ensures safe access across threads.
            let participants = participants_clone.lock().unwrap();

            // Retrieve the list of receivers (participants) to send the message to.
            let receivers = participants.receivers();

            // If there are no participants to send to, continue the loop.
            if receivers.is_empty() {
                continue;
            }

            // Lock the mutex to access the network handler for sending messages.
            let mut network = handler_clone.lock().unwrap();

            // Generate a random message text.
            let msg_text = format!("random message {}", rand::thread_rng().gen_range(0..1000));
            let msg = Message::Text(msg_text.clone());

            // Log the message being sent for debugging or monitoring purposes.
            let formatted_msg = format!(
                "Sending message [{}] to {}",
                &msg_text,
                format_list_of_addrs(
                    &receivers
                        .iter()
                        .map(|ParticipantAddress { public, .. }| public)
                        .collect::<Vec<&SocketAddr>>(),
                )
            );
            print_event(clone_start_time.clone(), &formatted_msg);

            // Iterate through the list of receivers and send the message to each.
            for ParticipantAddress { endpoint, .. } in &receivers {
                send_message(&mut network, *endpoint, &msg);
            }
        });
    }

    /// Handles the event of a successful connection to another network participant.
    ///
    /// Upon establishing a connection, this method performs two primary actions:
    /// 1. Registers the newly connected participant in the local storage of known participants.
    /// 2. Sends initial messages to the new participant, including the public address of this participant
    ///    and a request to push the list of known participants.
    ///
    /// This setup ensures that every new participant is immediately aware of the network's topology
    /// and can start communicating with other nodes without further manual intervention.
    ///
    /// # Parameters
    ///
    /// - `endpoint`: The `Endpoint` representing the network connection to the new participant.
    ///   This value is used both to register the participant and to target the initial messages.
    fn connected(&self, endpoint: Endpoint) {
        // Lock the mutex to safely access the participants storage. This is necessary
        // because the network operation could be accessed from multiple threads.
        let mut participants = self.participants.lock().unwrap();

        // Add the endpoint of the newly connected participant to the known participants list.
        // This is critical for maintaining an up-to-date view of the network topology.
        participants.add_known_participant(endpoint);

        // Lock the mutex to safely access the node handler. This handler is responsible for
        // network communication and thus needs to be accessed in a thread-safe manner.
        let mut network = self.node_handler.lock().unwrap();

        // Send a message back to the newly connected participant containing this participant's
        // public address. This helps the new participant learn about the existence and address
        // of this node.
        send_message(
            &mut network,
            endpoint,
            &Message::PublicAddress(self.public_addr),
        );

        // Send another message to the newly connected participant requesting it to push
        // its list of known participants. This step is crucial for syncing the view of the
        // network topology with the new participant, enabling it to communicate with other nodes.
        send_message(&mut network, endpoint, &Message::PushParticipantsList);
    }

    /// Attempts to connect to a list of participant addresses received from another participant.
    ///
    /// This method iteratively checks each received address against the current list of known
    /// participants. If the address is not known and is not the address of this participant or
    /// the message sender, it attempts to establish a new connection. Successful new connections
    /// result in the address being added to the list of known participants.
    ///
    /// # Parameters
    ///
    /// - `message_sender`: The `Endpoint` of the participant that sent this list of addresses.
    ///   This is used to avoid trying to reconnect to the sender or to self.
    /// - `addrs`: A `Vec<SocketAddr>` containing the addresses of potential new participants to connect to.
    ///
    /// # Behavior
    ///
    /// For each address in `addrs` that is not already a known participant, this function tries
    /// to establish a new connection. If at least one new connection is successfully established,
    /// a message is logged indicating the successful connection to new participants.
    ///
    /// This approach allows the network to self-organize and expand as new participants join
    /// and share their lists of known connections.
    ///
    /// # Errors
    ///
    /// Connection attempts that fail will not stop the method from attempting to connect to the
    /// next address in the list. Each failure is logged with a message indicating the address
    /// of the failed connection attempt.
    fn pull_participants_list(&self, message_sender: Endpoint, addrs: Vec<SocketAddr>) {
        // Lock the node handler and participants storage to ensure thread-safe access.
        let network = self.node_handler.lock().unwrap();
        let mut participants = self.participants.lock().unwrap();

        // Track whether any new connections have been made to log this event later.
        let mut new_connections = false;

        // Iterate through each received participant address.
        for &participant_address in addrs.iter() {
            // Check if the address is not the current participant's, not the sender's,
            // and not already known.
            if participant_address != self.public_addr
                && participant_address != message_sender.addr()
                && !participants.is_known_participant(participant_address)
            {
                // Attempt to connect to the new participant address.
                match network
                    .network()
                    .connect(Transport::FramedTcp, participant_address)
                {
                    Ok((endpoint, _)) => {
                        // If successful, add the endpoint to the known participants.
                        participants.add_known_participant(endpoint);
                        new_connections = true;
                    }
                    Err(_) => println!("Failed to connect to {}", participant_address),
                }
            }
        }

        // If any new connections were made, log an event with the list of newly connected addresses.
        if new_connections {
            let formatted_msg = format!(
                "Connected to new participants: {}",
                format_list_of_addrs(
                    &addrs
                        .iter()
                        .filter(|&&addr| addr != self.public_addr)
                        .collect::<Vec<_>>()
                )
            );
            print_event(self.time_start.clone(), &formatted_msg);
        }
    }
}
