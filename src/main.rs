mod client;
mod merkle;
mod server;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <client|server>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "client" => client::run().await?,
        "server" => server::run().await?,
        _ => {
            eprintln!("Invalid argument. Use 'client' or 'server'.");
            std::process::exit(1);
        }
    }

    Ok(())
}