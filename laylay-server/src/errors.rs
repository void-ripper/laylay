use std::{backtrace::Backtrace, fmt::Display};

#[derive(Debug)]
pub enum ServerErrorKind {
    Io,
    Internal,
    Db,
}

#[derive(Debug)]
pub struct ServerErrors {
    msg: String,
    kind: ServerErrorKind,
    backtrace: Backtrace,
}

impl ServerErrors {
    pub fn internal(msg: &str) ->Self {
        Self {
            msg: msg.to_string(),
            kind: ServerErrorKind::Internal,
            backtrace: Backtrace::capture(),
        }
    }
    
    pub fn db(e: rusqlite::Error, msg: &str) -> Self {
        Self {
            msg: format!("{msg} -> {e}"),
            kind: ServerErrorKind::Db,
            backtrace: Backtrace::capture(),
        }
    }
}

impl Display for ServerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "{:?} {}\n{}", self.kind, self.msg, self.backtrace)
        write!(f, "{:?} {}", self.kind, self.msg)
    }
}

impl From<std::io::Error> for ServerErrors {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: ServerErrorKind::Io,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<Box<dyn std::error::Error>> for ServerErrors {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self {
            kind: ServerErrorKind::Internal,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<rusqlite::Error> for ServerErrors {
    fn from(value: rusqlite::Error) -> Self {
        Self {
            kind: ServerErrorKind::Db,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}
