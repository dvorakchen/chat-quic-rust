use actix::prelude::*;
use async_stream::stream;
use bytes::Bytes;
use common::Transfer;
use log::info;
use s2n_quic::stream::ReceiveStream;

pub(crate) struct ClientListen {
    rece_stream: Option<ReceiveStream>,
    email: String,
}

impl ClientListen {
    pub fn new(rece: ReceiveStream, email: String) -> Self {
        Self {
            rece_stream: Some(rece),
            email,
        }
    }
}

impl Actor for ClientListen {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("client listen started");

        let mut recv = self.rece_stream.take().unwrap();

        let incoming_bytes = stream! {
            while let Ok(bytes) = recv.receive().await {
                info!("received stream");
                yield bytes;
            }
        };

        ctx.add_stream(incoming_bytes);
    }
}

impl StreamHandler<Option<Bytes>> for ClientListen {
    fn handle(&mut self, bytes: Option<Bytes>, ctx: &mut Self::Context) {
        info!("handle incoming bytes");
        if bytes.is_none() {
            ctx.stop();
            return;
        }

        let bytes = bytes.unwrap();

        let transfer = Transfer::try_from(bytes).unwrap();

        if transfer.to != self.email {
            // not message to me, discard
            return;
        }

        let content = String::from_utf8(transfer.content.to_vec()).unwrap();

        println!("\n${}: {}", transfer.from, content);
    }
}
