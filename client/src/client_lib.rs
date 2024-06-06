use std::{net::SocketAddr, path::Path};

use actix::{Actor, Addr, WrapFuture};
use common::Transfer;
use log::info;
use s2n_quic::{
    client::Connect,
    stream::{BidirectionalStream, SendStream},
    Client, Connection,
};

use crate::client_listen::ClientListen;

/// representing a Client that connected but not logged in
pub struct InitClient {
    _client: Client,
    _connection: Connection,
    stream: BidirectionalStream,
    email: String,
}

impl InitClient {
    pub async fn new(
        certificate: String,
        server_addr: SocketAddr,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .with_tls(Path::new(&certificate))?
            .with_io("0.0.0.0:0")?
            .start()?;

        let connect = Connect::new(server_addr).with_server_name("localhost");
        let mut connection = client.connect(connect).await?;

        connection.keep_alive(true)?;

        let stream = connection.open_bidirectional_stream().await?;

        // stream.send("data".into()).await.expect("send data failed");
        Ok(Self {
            _client: client,
            _connection: connection,
            stream,
            email: Default::default(),
        })
    }

    pub async fn login(
        mut self,
        email: String,
    ) -> Result<LoggedInClient, Box<dyn std::error::Error>> {
        self.stream.send(email.clone().into()).await?;
        self.stream.flush().await.unwrap();
        info!("sent email change");
        self.email = email;

        let (receiver, sender) = self.stream.split();

        let client_listen = ClientListen::new(receiver, self.email.clone()).start();

        Ok(LoggedInClient {
            _client: self._client,
            _connection: self._connection,
            email: self.email,
            send_stream: sender,
            _session_addr: client_listen,
        })
    }
}

pub struct LoggedInClient {
    _client: Client,
    _connection: Connection,
    send_stream: SendStream,
    email: String,
    _session_addr: Addr<ClientListen>,
}

impl LoggedInClient {
    pub async fn say(
        &mut self,
        to: String,
        content: String,
    ) -> Result<(), s2n_quic::stream::Error> {
        let bytes = Transfer {
            from: self.email.clone(),
            to,
            content: content.into(),
        }
        .to_bytes();

        self.send_stream.send(bytes).await.unwrap();
        self.send_stream.flush().await.unwrap();

        Ok(())
    }
}
