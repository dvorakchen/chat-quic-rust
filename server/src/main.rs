mod server;
mod sessions;

use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
struct Args {
    /// Certificate
    #[arg(short, long)]
    certificate: String,
    /// private key
    #[arg(short, long)]
    key: String,
    /// server listening address
    #[arg(short, long)]
    listen: String,
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv()?;
    env_logger::init();

    let args = Args::parse();

    let server = server::Server::new(args.certificate, args.key, args.listen)?;
    server.start()?.await;

    Ok(())
}
