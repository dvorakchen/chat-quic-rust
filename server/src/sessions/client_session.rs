use std::{fmt::Display};

use actix::prelude::*;
use async_stream::stream;
use bytes::Bytes;
use log::{info, warn};
use s2n_quic::{
    connection::Connection,
    stream::{BidirectionalStream, SendStream},
};

use super::Stop;

pub struct ClientSession {
    conn: Option<Connection>,
    email: String,
    send_stream: Option<SendStream>,
    status: ClientStatus,
}

impl ClientSession {
    pub fn new(conn: Connection, email: String) -> Self {
        info!("client new, email: {}", email);

        Self {
            conn: Some(conn),
            email,
            send_stream: None,
            status: ClientStatus::Init,
        }
    }

    fn handle_data(&mut self, bytes: Bytes) -> Result<(), ClientSessionError> {
        let old_email = self.email.clone();
        if let Ok(email) = String::from_utf8(bytes.to_vec()) {
            match self.status {
                ClientStatus::Init => {
                    self.email = email;
                    self.status = ClientStatus::LoggedIn;
                    info!("client: {} change email: {}", old_email, self.email);
                }
                ClientStatus::LoggedIn => {
                    info!("client: {} send back data", self.email);
                    self.send_stream.as_mut().unwrap().send_data(bytes).unwrap();
                }
            }
        } else {
            return Err(ClientSessionError::EmailInvalid);
        }

        Ok(())
    }
}

enum ClientStatus {
    Init,
    LoggedIn,
}

#[derive(Debug)]
pub enum ClientSessionError {
    EmailInvalid,
}

impl Display for ClientSessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ClientSessionError::EmailInvalid => "invalid email",
        };

        write!(f, "{}", msg)
    }
}

impl std::error::Error for ClientSessionError {}

impl Actor for ClientSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("client: {} startd", self.email);
        let email = self.email.clone();

        let recv_stream = {
            let conn = self.conn.take();
            if conn.is_none() {
                return;
            }
            let mut conn = conn.unwrap();

            stream! {
                while let Ok(stream) = conn.accept_bidirectional_stream().await {
                    if stream.is_some() {
                        info!("client: {} received stream", email);
                        yield stream.unwrap();
                    } else {
                        info!("client: {} connection closed without error", email);
                        return;
                    }
                }
                info!("client: {} connection closed with error", email);
            }
        };

        ctx.add_stream(recv_stream);
    }
}

impl StreamHandler<BidirectionalStream> for ClientSession {
    fn handle(&mut self, stream: BidirectionalStream, ctx: &mut Self::Context) {
        println!("connected, enter your email");
        let email = self.email.clone();
        let (mut recv, send) = stream.split();

        self.send_stream = Some(send);

        let recv_bytes = stream! {
            while let Ok(bytes) = recv.receive().await {
                info!("client: {} received data", email);
                yield bytes;
            }

            warn!("client: {} stream closed", email);
        };

        ctx.add_stream(recv_bytes);
    }
}

impl StreamHandler<Option<Bytes>> for ClientSession {
    fn handle(&mut self, bytes: Option<Bytes>, ctx: &mut Self::Context) {
        info!("client: {} handling data", self.email);

        if bytes.is_none() {
            info!("client: {} handle data none, stop session", self.email);
            ctx.stop();
        }

        if let Err(e) = self.handle_data(bytes.unwrap()) {
            self.send_stream
                .as_mut()
                .unwrap()
                .send_data(e.to_string().into())
                .expect("client send data failed");
        }
    }
}

impl Handler<Stop> for ClientSession {
    type Result = ();

    fn handle(&mut self, _msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
