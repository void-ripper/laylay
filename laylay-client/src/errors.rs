use std::{error::Error, fmt::Display};

use openxr::LoadError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Debug)]
pub enum ClientError {
    Io(String),
    Xr(String),
    Internal(String),
    Tracing(String),
}

impl Error for ClientError {}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Box<dyn std::error::Error>> for ClientError {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<std::io::Error> for ClientError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<LoadError> for ClientError {
    fn from(value: LoadError) -> Self {
        Self::Xr(value.to_string())
    }
}

impl From<openxr::sys::Result> for ClientError {
    fn from(value: openxr::sys::Result) -> Self {
        Self::Xr(value.to_string())
    }
}

impl From<SetGlobalDefaultError> for ClientError {
    fn from(value: SetGlobalDefaultError) -> Self {
        Self::Tracing(value.to_string())
    }
}
