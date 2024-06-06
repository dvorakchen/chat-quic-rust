mod client_lib;
mod client_listen;

use core::task;
use std::{
    error::Error,
    io::{stdin, Write},
    net::SocketAddr,
    path::Path,
    thread,
    time::Duration,
};

use actix::fut::future;
use clap::Parser;
use log::info;
use s2n_quic::{client::Connect, Client};
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, short)]
    certificate: String,
    #[arg(long, short)]
    server: String,
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let args = Args::parse();

    // let mut client = Client::builder()
    //     .with_tls(Path::new(&args.certificate))?
    //     .with_io("0.0.0.0:0")?
    //     .start()?;

    // let addr: SocketAddr = args.server.parse()?;
    // let connect = Connect::new(addr).with_server_name("localhost");
    // let mut connection = client.connect(connect).await?;

    // // ensure the connection doesn't time out with inactivity
    // connection.keep_alive(true)?;

    // // open a new stream and split the receiving and sending sides
    // let mut stream = connection.open_bidirectional_stream().await?;

    // stream.write_all(b"hello post-quantum server!").await?;
    // // spawn a task that copies responses from the server to stdout
    // // tokio::spawn(async move {
    // //     let mut stdout = tokio::io::stdout();
    // //     let _ = tokio::io::copy(&mut receive_stream, &mut stdout).await;
    // // });

    // // copy data from stdin and send it to the server
    // // let mut stdin = tokio::io::stdin();
    // // tokio::io::copy(&mut stdin, &mut stream).await?;
    // // tokio::io::copy(&mut stdin, &mut send_stream).await?;

    // client.wait_idle().await.unwrap();

    let client =
        client_lib::InitClient::new(args.certificate, args.server.parse().unwrap()).await?;

    print!("connected, enter your email: ");
    let mut stdout = std::io::stdout();
    stdout.flush().unwrap();

    let stdin = stdin();

    let mut client = {
        let mut txt = String::new();
        stdin.read_line(&mut txt).unwrap();
        info!("get email: {}", txt);
        let logged_in = client.login(txt.trim().to_string()).await?;
        info!("client logged in");

        logged_in
    };

    let talk_to = {
        print!("talk to:");
        stdout.flush().unwrap();
        let mut txt = String::new();
        stdin.read_line(&mut txt).unwrap();
        txt.trim().to_string()
    };

    for line in stdin.lines() {
        if let Ok(txt) = line {
            let txt = txt.trim().to_string();

            client
                .say(talk_to.clone(), txt)
                .await
                .expect("client talk wrong");
        } else {
            break;
        }
    }

    Ok(())
}
