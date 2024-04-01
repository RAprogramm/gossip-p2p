mod cli;
mod printer;
mod message;
mod peer;
mod peer_storage;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("{}", cli::get_help_message(&args[0]));
        std::process::exit(1);
    }

    match cli::parse_arguments(&args[1..]) {
        Ok(cli_args) => {
            let peer = peer::Peer::new(
                cli_args.port.into(),
                cli_args.period.try_into().unwrap(),
                cli_args.connect,
            )?;
            peer.run();
            Ok(())
            // match peer {
            //     Ok(peer) => {
            //         if let Err(e) =  {
            //             eprintln!("Error running peer: {}", e);
            //             std::process::exit(1);
            //         }
            //     }
            //     Err(e) => {
            //         eprintln!("Failed to create peer: {}", e);
            //         std::process::exit(1);
            //     }
            // }
        }
        Err(_) => {
            eprintln!("{}", cli::get_help_message(&args[0]));
            std::process::exit(1);
        }
    }
}
