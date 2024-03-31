//! # Simple Logger Module
//!
//! This module provides a straightforward logging system designed for asynchronously tracking and reporting the execution time of tasks along with custom messages. Its main purpose is to offer developers a convenient tool for monitoring the duration of operations or tasks within asynchronous applications.
//!
//! ## Features
//! - Supports asynchronous logging of elapsed time since initialization.
//! - Outputs elapsed time in the HH:MM:SS format alongside a custom message.
//! - Ease of use with minimal configuration and setup required.
//!
//! ## Usage
//! To use this module, an instance of `Arc<Instant>` must be created to serve as the starting point for time measurement. Subsequently, the `log` function can be used to asynchronously log the duration of tasks along with custom messages. This system is particularly useful in applications where tracking the execution time of various asynchronous operations is necessary for debugging, performance monitoring, or merely for informational purposes.
//!
//! The module's design allows for easy integration into existing Tokio-based asynchronous applications, providing a lightweight and efficient means of time tracking without significant overhead.
//!
//! ### Example
//! Here's a simple example demonstrating how to initialize the logger and log a message with the elapsed time:
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() {
//!     let addr = "127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap();
//!     let start_time = init(&addr).await; // Initialize the logger and start time.
//!
//!     // Perform some operations...
//!
//!     log(start_time, "Operation completed").await; // Log a custom message with the elapsed time.
//! }
//! ```
//!
//! This documentation should provide a clear understanding of the module's functionality and how it can be applied to enhance logging and time tracking within asynchronous Rust applications.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;

/// A simple logger for tracking and outputting elapsed time with messages.
pub struct SimpleLogger;

impl SimpleLogger {
    /// Logs the elapsed time along with a custom message.
    ///
    /// # Parameters
    /// - `start_time`: An `Arc<Instant>` marking the start time. This is shared across tasks to ensure a consistent reference point.
    /// - `msg`: A message to be logged alongside the elapsed time.
    ///
    /// # Examples
    /// ```no_run
    /// let start_time = Arc::new(Instant::now());
    /// SimpleLogger::log_time(start_time, "Task completed").await;
    /// ```
    async fn log_time(start_time: Arc<Instant>, msg: &str) {
        let elapsed = Instant::now().duration_since(*start_time);

        // Simple time formatting to HH:MM:SS.
        let hours = elapsed.as_secs() / 3600;
        let minutes = (elapsed.as_secs() % 3600) / 60;
        let seconds = elapsed.as_secs() % 60;

        // Output the formatted time and message.
        println!("# {:02}:{:02}:{:02} - {}", hours, minutes, seconds, msg);
    }
}

/// Initializes logging and returns a reference to the start time.
///
/// # Parameters
/// - `addr`: A reference to a `SocketAddr` indicating the address to be logged.
///
/// # Returns
/// An `Arc<Instant>` marking the start time of the logger.
///
/// # Examples
/// ```no_run
/// let addr = "127.0.0.1:8080".parse().unwrap();
/// let start_time = init(&addr).await;
/// ```
pub async fn init(addr: &SocketAddr) -> Arc<Instant> {
    // Capture the current time as the start time.
    let start_time = Arc::new(Instant::now());

    // Clone the start time to share among tasks.
    let start_time_clone = start_time.clone();

    // Format the message to include the address.
    let msg = format!("My address is \"{}\"", addr);

    // Spawn a new asynchronous task to log the address and start time.
    task::spawn(async move {
        SimpleLogger::log_time(start_time_clone, &msg).await;
    });

    // Return the start time for further use.
    start_time
}

/// Logs a message with the elapsed time since initialization.
///
/// # Parameters
/// - `start_time`: An `Arc<Instant>` marking the start time.
/// - `msg`: The message to log.
///
/// # Examples
/// ```no_run
/// let start_time = Arc::new(Instant::now());
/// log(start_time, "Another task completed").await;
/// ```
pub async fn log(start_time: Arc<Instant>, msg: &str) {
    SimpleLogger::log_time(start_time, msg).await;
}
