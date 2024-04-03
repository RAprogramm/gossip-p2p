use std::net::SocketAddr;

use message_io::network::Endpoint;
use message_io::node::NodeHandler;

use crate::message::Message;

pub trait ToSocketAddr {
    fn get_addr(&self) -> SocketAddr;
}

impl ToSocketAddr for Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

impl ToSocketAddr for &Endpoint {
    fn get_addr(&self) -> SocketAddr {
        self.addr()
    }
}

impl ToSocketAddr for SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        *self
    }
}

impl ToSocketAddr for &SocketAddr {
    fn get_addr(&self) -> SocketAddr {
        **self
    }
}

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

pub fn send_message(handler: &mut NodeHandler<()>, to: Endpoint, msg: &Message) {
    let output_data = bincode::serialize(msg).unwrap();
    handler.network().send(to, &output_data);
}
