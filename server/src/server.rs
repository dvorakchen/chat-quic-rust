use std::{error::Error, fmt::Display, future::Future, net::SocketAddr, path::Path, task::Poll};

use actix::{Actor, Addr};
use log::info;

use crate::sessions::ServerSession;

pub struct Server {
    certificate: String,
    key: String,
    listen: SocketAddr,
    session: Option<Addr<ServerSession>>,
}

impl Server {
    pub fn new(
        certificate: impl AsRef<str>,
        key: impl AsRef<str>,
        listen: impl AsRef<str>,
    ) -> Result<Self, ConstructServerError> {
        info!("new a Server");

        Ok(Self {
            certificate: certificate.as_ref().to_string(),
            key: key.as_ref().to_string(),
            listen: listen
                .as_ref()
                .parse()
                .map_err(|_| ConstructServerError::InvalidIpAddr)?,
            session: None,
        })
    }

    pub fn start(mut self) -> Result<RunningServer, Box<dyn Error>> {
        let server = s2n_quic::Server::builder()
            .with_tls((Path::new(&self.certificate), Path::new(&self.key)))?
            .with_io(self.listen.to_string().as_str())?
            .start()?;

        info!("start a server session");
        let server_session = ServerSession::new(server);
        let addr = server_session.start();
        self.session = Some(addr);

        Ok(RunningServer(self))
    }
}

pub struct RunningServer(#[allow(dead_code)] Server);

impl Future for RunningServer {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Poll::Pending
    }
}

#[derive(Debug)]
pub enum ConstructServerError {
    InvalidIpAddr,
}

impl Display for ConstructServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_msg = match self {
            ConstructServerError::InvalidIpAddr => "invalid Ip address",
        };

        write!(f, "{}", error_msg)
    }
}

impl Error for ConstructServerError {}
