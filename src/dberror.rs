use std::error;
use std::fmt;
use std::io;
use bincode::rustc_serialize::EncodingError;
use bincode::rustc_serialize::DecodingError;

// https://doc.rust-lang.org/book/error-handling.html#defining-your-own-error-type
#[derive(Debug)]
pub enum DbError {
    Io(io::Error),
    EncodingError(EncodingError),
    DecodingError(DecodingError),
    NotFoundError(String),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::Io(ref err) => write!(f, "IO error: {}", err),
            DbError::EncodingError(ref err) => write!(f, "Encoding error: {}", err),
            DbError::DecodingError(ref err) => write!(f, "Decoding error: {}", err),
            DbError::NotFoundError(ref err) => write!(f, "Not Found error: {}", err),
        }
    }
}

impl error::Error for DbError {
    fn description(&self) -> &str {
        match *self {
            DbError::Io(ref err) => err.description(),
            DbError::EncodingError(ref err) => err.description(),
            DbError::DecodingError(ref err) => err.description(),
            DbError::NotFoundError(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DbError::Io(ref err) => Some(err),
            DbError::EncodingError(ref err) => Some(err),
            DbError::DecodingError(ref err) => Some(err),
            DbError::NotFoundError(_) => None,
        }
    }
}

impl From<io::Error> for DbError {
    fn from(err: io::Error) -> DbError {
        DbError::Io(err)
    }
}

impl From<EncodingError> for DbError {
    fn from(err: EncodingError) -> DbError {
        DbError::EncodingError(err)
    }
}

impl From<DecodingError> for DbError {
    fn from(err: DecodingError) -> DbError {
        DbError::DecodingError(err)
    }
}

