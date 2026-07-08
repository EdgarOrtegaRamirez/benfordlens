use benfordlens::cli::{self, Cli};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = cli::run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
