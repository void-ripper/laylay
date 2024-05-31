use std::fmt::Display;


#[derive(Debug)]
pub enum ServerErrors {
    Io(String),
    Internal(String),
    Db(String),
}

impl Display for ServerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for ServerErrors {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for ServerErrors {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Self::Internal(value.to_string())
    }
}

impl From<rusqlite::Error> for ServerErrors {
    fn from(value: rusqlite::Error) -> Self {
        Self::Db(value.to_string())
    }
}
