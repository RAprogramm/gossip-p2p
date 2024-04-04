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
//!   - `participant`: Manages network participant logic, including message handling and participant storage.
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
mod participant;
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
            // Determine if the participant is the first in the network or joining an existing one.
            let not_first_participant = cli_args.connect.is_some();

            if not_first_participant {
                let participant_or_server = participant::model::Participant::new(
                    cli_args.period.try_into().unwrap(),
                    cli_args.port.into(),
                    cli_args.connect,
                );
                match participant_or_server {
                    Ok(instance) => instance.run(),
                    Err(e) => {
                        eprintln!("Failed to create instance: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let participant_or_server = participant::model::Participant::new(
                    cli_args.period.try_into().unwrap(),
                    cli_args.port.into(),
                    None,
                );
                match participant_or_server {
                    Ok(instance) => instance.run(),
                    Err(err) => {
                        eprintln!("Can not run the instance: {}", err);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("{}", cli::get_help_message(&args[0]));
            std::process::exit(1);
        }
    }
}
