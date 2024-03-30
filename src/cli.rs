//! # CLI Module for GOSSIP P2P Application
//!
//! This module provides the command-line interface functionality for the GOSSIP P2P application.
//! It includes parsing command-line arguments, generating help messages, and managing the settings
//! required to initiate P2P communication.
//!
//! The module defines the `CliArguments` struct which holds all the necessary parameters
//! extracted from the command line. It also contains functions to parse these arguments
//! and to generate a formatted help message that includes usage instructions and examples.
//!
//! ## Features
//!
//! - Parse command-line arguments for the application.
//! - Provide detailed help messages and usage examples.
//! - Support for essential P2P settings: messaging period, connection port, and optional peer connection.
//!
//! ## Usage
//!
//! To use this module, include it in your main application file and call the `parse_arguments`
//! function with the command-line arguments. This function will return a `CliArguments` struct
//! which can be used to configure the P2P application.
//!
//! ```
//! use gossip_p2p::cli::{parse_arguments, CliArguments, get_help_message};
//!
//! fn main() {
//!     let args: Vec<String> = std::env::args().collect();
//!     match parse_arguments(&args[1..]) {
//!         Ok(cli_args) => {
//!             // Use cli_args to configure the application
//!         },
//!         Err(e) => {
//!             eprintln!("{}", e);
//!             eprintln!("{}", get_help_message(&args[0]));
//!             std::process::exit(1);
//!         }
//!     }
//! }
//! ```
//!
//! Ensure that you handle the `Result` returned by `parse_arguments` properly, displaying
//! the help message and terminating the application in case of an error.

// Constants for the application's name and description.
const APP_NAME: &str = "\t\t\t---{ GOSSIP P2P }---";
const APP_DESCRIPTION: &str = "\t\tSimple p2p gossiping application in Rust.";

/// Structure to hold command-line arguments.
///
/// This structure represents the command-line arguments passed to the
/// application. It includes the messaging period, the port for connections,
/// and optionally, the address of a peer to connect to.
pub struct CliArguments {
    pub period: u64,
    pub port: u16,
    pub connect: Option<String>,
}

/// Generates a help message for the application.
///
/// This function constructs a help message using the application's name,
/// description, usage pattern, and examples of how to use the command-line
/// interface. It formats this information into a readable string that can be
/// displayed to the user.
///
/// # Arguments
///
/// * `program_name` - The name of the program executable. This is typically
///   retrieved from the command-line arguments and used to show how to run the
///   program in the usage examples.
///
/// # Returns
///
/// A string containing the formatted help message.
pub fn get_help_message(program_name: &str) -> String {
    let usage = format!(
        "Usage:\n\t{} --period=<seconds> --port=<port> [--connect=<peer_address_with_port>]",
        program_name
    );
    let arguments = "\
        Arguments:\n\
        \tperiod - messaging period in seconds (required)\n\
        \tport - connection port (required)\n\
        \tconnect - address of the peer";

    let examples = format!(
        "Examples:\n\
        \t# Starting the first peer with messaging period 5 seconds at port 8080:\n\
        \t{} --period=5 --port=8080\n\
        \n\
        \t# Starting the second peer which will connect to the first\n\
        \t# messaging period - 6 seconds\n\
        \t# port - 8081\n\
        \t{} --period=6 --port=8081 --connect=\"127.0.0.1:8080\"\n\
        \n\
        \t# Starting the second peer which will connect to all the peers through the first\n\
        \t# messaging period - 7 seconds\n\
        \t# port - 8082\n\
        \t{} --period=7 --port=8082 --connect=\"127.0.0.1:8080\"",
        program_name, program_name, program_name
    );

    format!(
        "\n{}\n{}\n\n{}\n\n{}\n\n{}",
        APP_NAME, APP_DESCRIPTION, usage, arguments, examples
    )
}

/// Parses a single command-line argument.
///
/// This helper function looks for an argument with a specific prefix, extracts
/// the value after the "=" character, and tries to parse it as a `u64`.
///
/// # Arguments
///
/// * `args` - A slice of strings representing all command-line arguments.
/// * `prefix` - The prefix to look for when filtering the arguments.
///
/// # Returns
///
/// An `Option<u64>` which is `Some` if the argument was found and successfully
/// parsed, or `None` otherwise.
fn parse_each_arg(args: &[String], prefix: &str) -> Option<u64> {
    args.iter()
        .find(|arg| arg.starts_with(prefix))
        .and_then(|arg| arg.split('=').nth(1))
        .and_then(|value| value.parse().ok())
}

/// Parses all command-line arguments.
///
/// This function extracts and validates the command-line arguments required by
/// the application. It ensures that the mandatory arguments `--period` and
/// `--port` are provided and correctly formatted. It also handles the optional
/// `--connect` argument.
///
/// # Arguments
///
/// * `args` - A slice of strings representing all command-line arguments.
///
/// # Returns
///
/// A `Result` which is `Ok` with a `CliArguments` struct if the arguments were
/// successfully parsed, or an `Err` with a static error message otherwise.
pub fn parse_arguments(args: &[String]) -> Result<CliArguments, &'static str> {
    let period_arg = parse_each_arg(args, "--period=")
        .ok_or("Period is required and must be a positive number")?;
    let port_arg =
        parse_each_arg(args, "--port=").ok_or("Port is required and must be a positive number")?;

    let connect_arg = args
        .iter()
        .find(|arg| arg.starts_with("--connect="))
        .map(|s| s.split('=').nth(1).unwrap().to_string());

    Ok(CliArguments {
        period: period_arg,
        port: port_arg as u16,
        connect: connect_arg,
    })
}
