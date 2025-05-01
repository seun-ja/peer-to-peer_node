use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Message {
    RememeberMe,
    Comms(String),
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        match s.as_str() {
            "RememeberMe" => Message::RememeberMe,
            s => Message::Comms(s.to_owned()),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::RememeberMe => write!(f, "RememeberMe"),
            Message::Comms(s) => write!(f, "{s}"),
        }
    }
}
