use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct ConfFileNotFoundError {
    pub msg: String,
}

impl ConfFileNotFoundError {
    pub fn new(msg: &str) -> Self {
        ConfFileNotFoundError {
            msg: msg.to_string()
        }
    }
}

impl fmt::Display for ConfFileNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

pub enum ForwardingError {
    BindError(String),
    SshError(String),
    RemoteChannelError(String),
    ListeningError(String),
}

impl fmt::Display for ForwardingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ForwardingError::BindError(msg) => write!(f, "{}", msg),
            ForwardingError::SshError(msg) => write!(f, "{}", msg),
            ForwardingError::RemoteChannelError(msg) => write!(f, "{}", msg),
            ForwardingError::ListeningError(msg) => write!(f, "{}", msg)
        }
    }
}
