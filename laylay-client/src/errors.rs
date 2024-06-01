use std::{error::Error, fmt::Display};

use openxr::LoadError;


#[derive(Debug)]
pub enum ClientError {
    Io(String),
    Xr(String),
}

impl Error for ClientError {}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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