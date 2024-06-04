use std::{backtrace::Backtrace, error::Error, fmt::Display};

use openxr::LoadError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug)]
pub enum ClientErrorKind {
    Io,
    Xr,
    Internal,
    Tracing,
}

#[derive(Debug)]
pub struct ClientError {
    kind: ClientErrorKind,
    msg: String,
    backtrace: Backtrace,
}

impl Error for ClientError {}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}\n{}", self.kind, self.msg, self.backtrace)
    }
}

impl From<Box<dyn std::error::Error>> for ClientError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self {
            kind: ClientErrorKind::Internal,
            msg: value.to_string(),
            backtrace: Backtrace::force_capture(),
        }
    }
}

impl From<std::io::Error> for ClientError {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: ClientErrorKind::Io,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<LoadError> for ClientError {
    fn from(value: LoadError) -> Self {
        Self {
            kind: ClientErrorKind::Xr,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<openxr::sys::Result> for ClientError {
    fn from(value: openxr::sys::Result) -> Self {
        Self {
            kind: ClientErrorKind::Xr,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<SetGlobalDefaultError> for ClientError {
    fn from(value: SetGlobalDefaultError) -> Self {
        Self {
            kind: ClientErrorKind::Tracing,
            msg: value.to_string(),
            backtrace: Backtrace::capture(),
        }
    }
}
