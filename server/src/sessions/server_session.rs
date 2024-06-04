use actix::prelude::*;
use async_stream::stream;
use log::{info};
use s2n_quic::Connection;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use super::Stop;
use crate::sessions::ClientSession;

pub struct ServerSession {
    quic_server: Arc<Mutex<s2n_quic::Server>>,
    clients: HashMap<String, Addr<ClientSession>>,
}

impl ServerSession {
    pub fn new(quic_server: s2n_quic::Server) -> Self {
        info!("new server session");
        Self {
            quic_server: Arc::new(Mutex::new(quic_server)),
            clients: HashMap::new(),
        }
    }
}

impl Actor for ServerSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("server session started");
        let quic_server = Arc::clone(&self.quic_server);

        let incoming = stream! {
            let mut quic_server = quic_server.lock().await;
            loop {
                let incoming = quic_server.accept().await ;
                yield incoming;
            }
        };
        info!("listening incoming connections");
        ctx.add_stream(incoming);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl StreamHandler<Option<Connection>> for ServerSession {
    fn handle(&mut self, item: Option<Connection>, ctx: &mut Self::Context) {
        info!("handle received connection");

        if item.is_none() {
            info!("none connection, server stop");
            ctx.stop();
        }

        info!("generate client session");
        let connection = item.unwrap();
        let tempoparily_id = ulid::Ulid::new().to_string();

        let client_addr = ClientSession::new(connection, tempoparily_id.clone()).start();

        info!("temporarily client id is: {}", tempoparily_id);
        self.clients.insert(tempoparily_id, client_addr);
    }
}

impl Handler<Stop> for ServerSession {
    type Result = ();

    fn handle(&mut self, _msg: Stop, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
