mod cli;
mod logger;
mod peer;
mod peer_storage;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("{}", cli::get_help_message(&args[0]));
        std::process::exit(1);
    }

    match cli::parse_arguments(&args[1..]) {
        Ok(cli_args) => {
            let peer = peer::Peer::new(cli_args.port, cli_args.period, cli_args.connect).await;
            match peer {
                Ok(peer) => {
                    if let Err(e) = peer.run().await {
                        eprintln!("Error running peer: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create peer: {}", e);
                    std::process::exit(1);
                }
            }
            // Здесь логика работы вашего приложения с использованием cli_args
            // println!(
            //     "Period: {}, Port: {}, Connect: {:?}",
            //     cli_args.period, cli_args.port, cli_args.connect
            // );
        }
        Err(_) => {
            eprintln!("{}", cli::get_help_message(&args[0]));
            std::process::exit(1);
        }
    }

    // Ожидание сигнала Ctrl+C
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for event");
    println!("Получен сигнал завершения, выход из программы.");
}
