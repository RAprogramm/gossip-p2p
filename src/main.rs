mod cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("{}", cli::get_help_message(&args[0]));
        std::process::exit(1);
    }

    match cli::parse_arguments(&args[1..]) {
        Ok(cli_args) => {
            // Здесь логика работы вашего приложения с использованием cli_args
            println!(
                "Period: {}, Port: {}, Connect: {:?}",
                cli_args.period, cli_args.port, cli_args.connect
            );
        }
        Err(_) => {
            eprintln!("{}", cli::get_help_message(&args[0]));
            std::process::exit(1);
        }
    }
}
