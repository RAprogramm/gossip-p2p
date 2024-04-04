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

    pub fn run(mut self) {
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

        self.sending_random_message();

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

    fn network_messages(&self, message_sender: Endpoint, message: Message) {
        match message {
            Message::PublicAddress(pub_addr) => {
                let mut participants = self.participants.lock().unwrap();
                participants.add_unknown_participant(message_sender, pub_addr);
            }

            Message::PushParticipantsList => {
                let list = {
                    let participants = self.participants.lock().unwrap();
                    participants.get_participants_list()
                };
                let msg = Message::PullParticipantsList(list);
                send_message(&mut self.node_handler.lock().unwrap(), message_sender, &msg);
            }

            Message::PullParticipantsList(addrs) => {
                self.pull_participants_list(message_sender, addrs)
            }

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

    fn sending_random_message(&self) {
        let tick_duration = Duration::from_secs(self.period as u64);
        let participants_clone = Arc::clone(&self.participants);
        let handler_clone = Arc::clone(&self.node_handler);
        let clone_start_time = self.time_start.clone();

        thread::spawn(move || loop {
            thread::sleep(tick_duration);

            let participants = participants_clone.lock().unwrap();
            let receivers = participants.receivers();

            if receivers.is_empty() {
                continue;
            }

            let mut network = handler_clone.lock().unwrap();

            let msg_text = format!("random message {}", rand::thread_rng().gen_range(0..1000));
            let msg = Message::Text(msg_text.clone());

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

            for ParticipantAddress { endpoint, .. } in &receivers {
                send_message(&mut network, *endpoint, &msg);
            }
        });
    }

    fn connected(&self, endpoint: Endpoint) {
        let mut participants = self.participants.lock().unwrap();
        participants.add_known_participant(endpoint);

        let mut network = self.node_handler.lock().unwrap();

        send_message(
            &mut network,
            endpoint,
            &Message::PublicAddress(self.public_addr),
        );

        send_message(&mut network, endpoint, &Message::PushParticipantsList);
    }

    fn pull_participants_list(&self, message_sender: Endpoint, addrs: Vec<SocketAddr>) {
        let network = self.node_handler.lock().unwrap();
        let mut participants = self.participants.lock().unwrap();

        let mut new_connections = false; // Флаг новых подключений

        for &participant_address in addrs.iter() {
            if participant_address != self.public_addr
                && participant_address != message_sender.addr()
                && !participants.is_known_participant(participant_address)
            {
                match network
                    .network()
                    .connect(Transport::FramedTcp, participant_address)
                {
                    Ok((endpoint, _)) => {
                        participants.add_known_participant(endpoint);
                        new_connections = true;
                    }
                    Err(_) => println!("Failed to connect to {}", participant_address),
                }
            }
        }

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
