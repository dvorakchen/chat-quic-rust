use actix::prelude::*;
use bytes::Bytes;
use std::fmt::Display;

#[derive(Message)]
#[rtype(result = "Result<(), ClientChangeError>")]
pub enum ClientChange {
    UpdateEmail(String, String),
}

#[derive(Debug)]
pub enum ClientChangeError {
    NewEmailAlreadyExisted,
}

impl Display for ClientChangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ClientChangeError::NewEmailAlreadyExisted => "new email already exsited",
        };

        write!(f, "{}", msg)
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), TransferError>")]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub content: Bytes,
}

impl Transfer {
    /// convert Transfer to bytes
    /// representing the belowing form:
    ///
    /// <from email>\n
    /// <to email>\n
    /// <content>
    ///
    pub fn to_bytes(self) -> Bytes {
        let mut tb = String::from(self.from);
        tb.push('\n');
        tb.push_str(&self.to);
        tb.push('\n');
        tb.push_str(&String::from_utf8(self.content.to_vec()).unwrap());

        tb.into()
    }
}

impl std::convert::TryFrom<Bytes> for Transfer {
    type Error = TransferError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let infos = value.split(|v| *v == b'\n').collect::<Vec<_>>();
        if infos.len() != 3 {
            Err(TransferError::ConvertFromBytesFail)
        } else {
            Ok(Self {
                from: String::from_utf8(infos[0].to_vec()).unwrap(),
                to: String::from_utf8(infos[1].to_vec()).unwrap(),
                content: Bytes::copy_from_slice(infos[2]),
            })
        }
    }
}

#[derive(Debug)]
pub enum TransferError {
    DestinationClientOffline,
    ContentNotUTF8,
    ConvertFromBytesFail,
}

impl Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TransferError::*;

        let msg = match self {
            DestinationClientOffline => "destination client offline",
            ContentNotUTF8 => "transfer content not UTF-8 encoding",
            ConvertFromBytesFail => "convert from bytes failed",
        };

        write!(f, "{}", msg)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stop;
