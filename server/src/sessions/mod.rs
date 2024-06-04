mod client_session;
mod server_session;

pub use client_session::ClientSession;
pub use server_session::ServerSession;

use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stop;
