mod cli;
mod message;
mod peer;
mod peer_storage;
mod printer;
mod utils;

use std::convert::TryInto;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("{}", cli::get_help_message(&args[0]));
        std::process::exit(1);
    }

    match cli::parse_arguments(&args[1..]) {
        Ok(cli_args) => {
            let is_participant = cli_args.connect.is_some();

            if is_participant {
                let participant_or_server = peer::Peer::new(
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
                let participant_or_server = peer::Peer::new(
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
