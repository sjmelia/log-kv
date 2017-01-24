extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode_into, decode_from, EncodingError, DecodingError};
use rustc_serialize::{Encodable, Decodable};
use std::{error, fmt, io};
use std::cmp::Eq;
use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::io::{Read, Write, Seek, SeekFrom, ErrorKind as IoErrorKind};
use std::marker::PhantomData;

#[derive(Debug)]
pub enum LogKvError {
    Io(io::Error),
    EncodingError(EncodingError),
    DecodingError(DecodingError),
}

impl fmt::Display for LogKvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogKvError::Io(ref err) => write!(f, "IO error: {}", err),
            LogKvError::EncodingError(ref err) => write!(f, "Encoding error: {}", err),
            LogKvError::DecodingError(ref err) => write!(f, "Decoding error: {}", err),
        }
    }
}

impl error::Error for LogKvError {
    fn description(&self) -> &str {
        match *self {
            LogKvError::Io(ref err) => err.description(),
            LogKvError::EncodingError(ref err) => err.description(),
            LogKvError::DecodingError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            LogKvError::Io(ref err) => Some(err),
            LogKvError::EncodingError(ref err) => Some(err),
            LogKvError::DecodingError(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for LogKvError {
    fn from(err: io::Error) -> LogKvError {
        LogKvError::Io(err)
    }
}

impl From<EncodingError> for LogKvError {
    fn from(err: EncodingError) -> LogKvError {
        LogKvError::EncodingError(err)
    }
}

impl From<DecodingError> for LogKvError {
    fn from(err: DecodingError) -> LogKvError {
        LogKvError::DecodingError(err)
    }
}

pub struct LogKv<K, V, T> {
    cursor: T,
    index: HashMap<K, u64>,
    _phantom: PhantomData<V>,
}

impl<K, V, T> LogKv<K, V, T>
    where K: Encodable + Decodable + Eq + Hash,
          V: Encodable + Decodable,
          T: Read + Write + Seek
{
    pub fn create(cursor: T) -> Result<LogKv<K, V, T>, LogKvError> {
        let mut logkv = LogKv {
            cursor: cursor,
            index: HashMap::new(),
            _phantom: PhantomData,
        };

        logkv.cursor.seek(SeekFrom::Start(0))?;
        loop {
            let key = match decode_from::<T, K>(&mut logkv.cursor, SizeLimit::Infinite) {
                Ok(key) => key,
                Err(DecodingError::IoError(ref e)) if e.kind() == IoErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(LogKvError::from(e)),
            };

            let position = logkv.cursor.seek(SeekFrom::Current(0))?;
            logkv.index.insert(key, position);
            decode_from::<T, V>(&mut logkv.cursor, SizeLimit::Infinite)?;
        }

        Ok(logkv)
    }

    /// Stores a key-value pair, writing to the given Write.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use seekv::LogKv;
    ///
    /// let mut cursor = Cursor::new(Vec::new());
    /// let mut db = LogKv::create(cursor).unwrap();
    /// let key = "this is a key";
    /// let value = "this is a value";
    /// db.put(String::from(key), String::from(value)).unwrap();
    /// let retrieved = db.get(String::from(key)).unwrap().expect("No value retrieved");
    /// assert_eq!(retrieved, value);
    /// ```
    pub fn put(&mut self, key: K, value: V) -> Result<(), LogKvError> {
        encode_into(&key, &mut self.cursor, SizeLimit::Infinite)?;
        let position = self.cursor.seek(SeekFrom::Current(0))?;
        self.index.insert(key, position);
        encode_into(&value, &mut self.cursor, SizeLimit::Infinite)?;
        Ok(())
    }

    pub fn get(&mut self, key: K) -> Result<Option<V>, LogKvError> {
        return match self.index.get(&key) {
            Some(position) => {
                self.cursor.seek(SeekFrom::Start(*position))?;
                let value = decode_from(&mut self.cursor, SizeLimit::Infinite)?;
                Ok(Some(value))
            }
            None => Ok(None),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::LogKv;
    use uuid::Uuid;
    use std::string::String;
    use std::fs::remove_file;
    use std::fs;
    use std::io::Cursor;

    #[test]
    fn put_twice_then_get_returns_expected() {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("put_twice_then_get_returns_expected")
            .unwrap();

        let mut db = LogKv::create(file).unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(Uuid::new_v4(), String::from("valueA")).unwrap();
        db.put(key, String::from(value)).unwrap();
        let retrieved = db.get(key).unwrap().expect("No value retrieved");
        remove_file("put_twice_then_get_returns_expected").unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn get_returns_not_found() {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("get_returns_not_found")
            .unwrap();

        let mut db = LogKv::create(file).unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(Uuid::new_v4(), String::from("valueA")).unwrap();
        db.put(key, String::from(value)).unwrap();
        let retrieved = db.get(Uuid::new_v4()).unwrap();
        remove_file("get_returns_not_found").unwrap();
        assert_eq!(retrieved.is_some(), false);
    }
}
