use std::fmt::Display;

use actix::prelude::*;
use async_stream::stream;
use bytes::Bytes;
use log::{error, info, warn};
use s2n_quic::{
    connection::Connection,
    stream::{BidirectionalStream, SendStream},
};

use super::{server_session::ClientChange, ServerSession, Stop};

pub struct ClientSession {
    server_addr: Addr<ServerSession>,
    conn: Option<Connection>,
    email: String,
    send_stream: Option<SendStream>,
    status: ClientStatus,
}

impl ClientSession {
    pub fn new(conn: Connection, email: String, server_addr: Addr<ServerSession>) -> Self {
        info!("client new, email: {}", email);

        Self {
            server_addr,
            conn: Some(conn),
            email,
            send_stream: None,
            status: ClientStatus::Init,
        }
    }

    fn handle_data(
        &mut self,
        bytes: Bytes,
        ctx: &mut actix::Context<ClientSession>,
    ) -> Result<(), ClientSessionError> {
        if let Ok(email) = String::from_utf8(bytes.to_vec()) {
            info!("received data: {}", email);
            info!("status: {:?}", self.status);
            match self.status {
                ClientStatus::Init => {
                    self.change_email(email, ctx);
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

    fn change_email(&mut self, email: String, ctx: &mut actix::Context<ClientSession>) {
        self.server_addr
            .send(ClientChange::UpdateEmail(self.email.clone(), email.clone()))
            .into_actor(self)
            .map(|res, act, _ctx| {
                info!("in map");
                if let Err(e) = res {
                    error!("{}", e);
                } else {
                    act.email = email;
                    act.status = ClientStatus::LoggedIn;
                    info!("change email successful");
                }
            })
            .wait(ctx);
        info!("after email changed");
    }
}

#[derive(Debug)]
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

        if let Err(e) = self.handle_data(bytes.unwrap(), ctx) {
            error!("{}", e);
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
