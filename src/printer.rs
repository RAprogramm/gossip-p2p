use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

pub struct SimplePrinter;

impl SimplePrinter {
    fn time(start_time: Arc<Instant>, msg: &str) {
        let elapsed = Instant::now().duration_since(*start_time);

        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        println!("# {:02}:{:02}:{:02} - {}", hours, minutes, seconds, msg);
    }
}

pub fn init(addr: &SocketAddr) -> Arc<Instant> {
    let start_time = Arc::new(Instant::now());
    let start_time_clone = start_time.clone();
    let msg = format!("My address is \"{}\"", addr);

    SimplePrinter::time(start_time_clone, &msg);

    start_time
}

pub fn print_event(start_time: Arc<Instant>, msg: &str) {
    SimplePrinter::time(start_time, msg);
}
