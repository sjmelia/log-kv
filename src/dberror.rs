use std::error;
use std::fmt;
use std::io;
use bincode::rustc_serialize::EncodingError;

// https://doc.rust-lang.org/book/error-handling.html#defining-your-own-error-type
#[derive(Debug)]
pub enum DbError {
    Io(io::Error),
    EncodingError(EncodingError)
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::Io(ref err) => write!(f, "IO error: {}", err),
            DbError::EncodingError(ref err) => write!(f, "Encoding error: {}", err),
        }
    }
}

impl error::Error for DbError {
    fn description(&self) -> &str {
        match *self {
            DbError::Io(ref err) => err.description(),
            DbError::EncodingError(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DbError::Io(ref err) => Some(err),
            DbError::EncodingError(ref err) => Some(err),
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

