use clap::Parser;
use qubit_dependency_policy::cli::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = cli.execute() {
        eprintln!("{error}");
        std::process::exit(if matches!(error.code(), "DP001" | "DP101") {
            1
        } else {
            2
        });
    }
}
