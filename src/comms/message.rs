use std::fmt::Display;

use libp2p::identity::Keypair;
use serde::{Deserialize, Serialize};

use crate::block::{Transactable, TransactionError};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Comms(String),
    Transaction(Box<dyn Transactable>),
}

#[typetag::serde]
impl Transactable for Message {
    fn _submit(&self) -> Result<(), TransactionError> {
        todo!()
    }

    fn sign(&self, keypair: &Keypair) -> Result<(), TransactionError> {
        match self {
            Message::Comms(chat) => keypair
                .sign(chat.as_bytes())
                .map_err(TransactionError::SigningError)
                .map(|_| ()),
            Message::Transaction(action) => keypair
                .sign(action.as_ref().to_string().as_bytes())
                .map_err(TransactionError::SigningError)
                .map(|_| ()),
        }
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        let s = s.as_str();
        Message::Comms(s.to_owned())
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Comms(chat) => write!(f, "{chat}"),
            Message::Transaction(action) => write!(f, "{action}"),
        }
    }
}
