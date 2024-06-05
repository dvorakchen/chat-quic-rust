mod client;

use std::{error::Error, io::Write};

use clap::Parser;
use s2n_quic::{client::Connect, Client};
use std::{net::SocketAddr, path::Path};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, short)]
    certificate: String,
    #[arg(long, short)]
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let args = Args::parse();

    print!("connected, enter your email: ");
    std::io::stdout().flush().unwrap();

    let client = Client::builder()
        .with_tls(
            // webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
            Path::new(&args.certificate),
        )?
        .with_io("0.0.0.0:0")?
        .start()?;
    // webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
    let addr: SocketAddr = args.server.parse()?;
    let connect = Connect::new(addr).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;

    // ensure the connection doesn't time out with inactivity
    connection.keep_alive(true)?;

    // open a new stream and split the receiving and sending sides
    let stream = connection.open_bidirectional_stream().await?;

    let (mut receive_stream, mut send_stream) = stream.split();

    // spawn a task that copies responses from the server to stdout
    tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
    });

    // copy data from stdin and send it to the server
    let mut stdin = tokio::io::stdin();
    tokio::io::copy(&mut stdin, &mut send_stream).await?;

    Ok(())
}
