extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

use bincode::SizeLimit;
use bincode::rustc_serialize::encode_into;
use bincode::rustc_serialize::EncodingError;
use std::error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use uuid::Uuid;

// http://jadpole.github.io/rust/many-error-types
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

pub struct Db {
    file: File,
}

impl Db {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Db, DbError> {
        let file = try!(fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path));
        let db = Db {
            file: file
        };

        Ok(db)
    }

    pub fn put(&mut self, id: Uuid) -> Result<(), DbError> {
        try!(self.file.seek(SeekFrom::End(0)));
        try!(encode_into(&id, &mut self.file, bincode::SizeLimit::Infinite));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Db;
    use uuid::Uuid;

    #[test]
    fn it_works() {
        let mut db = Db::create("file.txt").unwrap();
        db.put(Uuid::new_v4());
    }
}
