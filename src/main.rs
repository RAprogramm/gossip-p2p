//! Entry Point for the Distributed Network Application.
//!
//! This application manages participants within a distributed network, handling their creation,
//! communication, and overall network interaction. It leverages command-line arguments to configure
//! participant instances and supports operations such as joining an existing network or starting
//! a new one.
//!
//! ## Features
//!
//! - Parses command-line arguments to configure the network participant's behavior.
//! - Supports starting a participant as part of an existing network or as the first node in a new network.
//! - Utilizes submodules for specific functionalities:
//!   - `cli`: Parses and interprets command-line arguments.
//!   - `peer`: Manages network peer logic, including message handling and participant storage.
//!   - `printer`: Provides utilities for logging and output formatting.
//!
//! ## Usage
//!
//! The application requires specific command-line arguments to run, including the period for
//! sending messages and the port to listen on. Optionally, it can connect to an existing network
//! participant to join the network.
//!
//! ```plaintext
//! Usage: my_network_app --period=<period> --port=<port> [--connect=<address>]
//! ```
//!
//! ## Example
//!
//! Starting a new network participant on port 8080 with a message sending period of 5 seconds:
//!
//! ```shell
//! cargo run -- --period=5 --port=8080
//! ```
//!
//! Joining an existing network by connecting to a known participant at `127.0.0.1:8081`:
//!
//! ```shell
//! cargo run -- --period=5 --port=8080 --connect=127.0.0.1:8081
//! ```

mod cli;
mod message;
mod peer;
mod printer;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Ensure that the necessary arguments are provided, otherwise display the help message.
    if args.len() < 3 {
        eprintln!("{}", cli::get_help_message(&args[0]));
        std::process::exit(1);
    }

    // Parse the command-line arguments and configure the application accordingly.
    match cli::parse_arguments(&args[1..]) {
        Ok(cli_args) => {
            let peer = peer::Peer::new(cli_args.period, cli_args.port, cli_args.connect);
            match peer {
                Ok(instance) => {
                    if let Err(e) = instance.run() {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("{}", cli::get_help_message(&args[0]));
            std::process::exit(1);
        }
    }
}
