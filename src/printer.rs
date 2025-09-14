//! A utility module for printing time-stamped events.
//!
//! This module offers functionality for logging events with a timestamp indicating
//! the elapsed time since a specified starting point. It's designed to aid in logging
//! and debugging, helping to track the sequence and timing of events within an application.
//!
//! The `SimplePrinter` struct acts as a namespace for the printing function, which can
//! be utilized directly to log messages with their elapsed time from a given `Instant`.
//! The `init` and `print_event` functions facilitate easy tracking of events relative
//! to an application-defined starting point.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A simple printer for logging events with time elapsed since an `Instant`.
pub struct SimplePrinter;

impl SimplePrinter {
    /// Prints a message with the elapsed time since a given start `Instant`.
    ///
    /// # Parameters
    ///
    /// * `start_time`: An `Arc<Instant>` representing the start time from which
    ///   elapsed time is calculated.
    /// * `msg`: The message to print along with the elapsed time.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::sync::Arc;
    /// use std::time::Instant;
    ///
    /// let start_time = Arc::new(Instant::now());
    /// gossip_p2p::printer::SimplePrinter::time(start_time, "Hello, world!");
    /// ```
    fn time(start_time: Arc<Instant>, msg: &str) {
        let elapsed = Instant::now().duration_since(*start_time);
        println!("{}", format_elapsed(elapsed, msg));
    }
}

/// Format a message with the elapsed time.
///
/// This helper returns the formatted output used by [`SimplePrinter`].
///
/// # Examples
///
/// ```
/// use gossip_p2p::printer::format_elapsed;
/// use std::time::Duration;
///
/// assert_eq!(
///     format_elapsed(Duration::new(3661, 0), "hi"),
///     "# 01:01:01 - hi"
/// );
/// ```
pub fn format_elapsed(elapsed: Duration, msg: &str) -> String {
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let seconds = elapsed.as_secs() % 60;
    format!("# {:02}:{:02}:{:02} - {}", hours, minutes, seconds, msg)
}

/// Initializes the printing utility and logs the starting event.
///
/// This function marks the beginning of event logging by printing the start event
/// with the address provided and returns an `Arc<Instant>` representing the start time.
///
/// # Parameters
///
/// * `addr`: A reference to a `SocketAddr` indicating the address related to the start event.
///
/// # Returns
///
/// Returns an `Arc<Instant>` that represents the start time for calculating elapsed time
/// in future events.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let addr = "127.0.0.1:8080".parse().unwrap();
/// let start_time = gossip_p2p::printer::init(&addr);
/// ```
pub fn init(addr: &SocketAddr) -> Arc<Instant> {
    let start_time = Arc::new(Instant::now());
    let start_time_clone = start_time.clone();
    let msg = format!("My address is \"{}\"", addr);

    SimplePrinter::time(start_time_clone, &msg);

    start_time
}

/// Prints an event with the elapsed time since the initial `Instant`.
///
/// This function is used to log an event with its elapsed time since the start time
/// specified by the `Arc<Instant>` argument.
///
/// # Parameters
///
/// * `start_time`: An `Arc<Instant>` representing the start time from which
///   elapsed time is calculated.
/// * `msg`: The message to print along with the elapsed time.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// // Assuming `start_time` has been initialized using `init` function
/// gossip_p2p::printer::print_event(start_time, "Event occurred");
/// ```
pub fn print_event(start_time: Arc<Instant>, msg: &str) {
    SimplePrinter::time(start_time, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_elapsed_formats() {
        let formatted = format_elapsed(Duration::new(3661, 0), "test");
        assert_eq!(formatted, "# 01:01:01 - test");
    }

    #[test]
    fn init_sets_start_time() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let start = init(&addr);
        assert!(Instant::now().duration_since(*start) < Duration::from_secs(1));
    }
}
