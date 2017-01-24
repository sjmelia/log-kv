extern crate bincode;
extern crate serde;

use bincode::SizeLimit;
use bincode::serde::{DeserializeError, deserialize_from, SerializeError, serialize_into};
use serde::{Serialize, Deserialize};
use std::{error, fmt, io};
use std::cmp::Eq;
use std::collections::hash_map::HashMap;
use std::hash::Hash;
use std::io::{Read, Write, Seek, SeekFrom, ErrorKind as IoErrorKind};
use std::marker::PhantomData;

#[derive(Debug)]
pub enum LogKvError {
    Io(io::Error),
    SerializeError(SerializeError),
    DeserializeError(DeserializeError),
}

impl fmt::Display for LogKvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogKvError::Io(ref err) => write!(f, "IO error: {}", err),
            LogKvError::SerializeError(ref err) => write!(f, "Encoding error: {}", err),
            LogKvError::DeserializeError(ref err) => write!(f, "Decoding error: {}", err),
        }
    }
}

impl error::Error for LogKvError {
    fn description(&self) -> &str {
        match *self {
            LogKvError::Io(ref err) => err.description(),
            LogKvError::SerializeError(ref err) => err.description(),
            LogKvError::DeserializeError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            LogKvError::Io(ref err) => Some(err),
            LogKvError::SerializeError(ref err) => Some(err),
            LogKvError::DeserializeError(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for LogKvError {
    fn from(err: io::Error) -> LogKvError {
        LogKvError::Io(err)
    }
}

impl From<SerializeError> for LogKvError {
    fn from(err: SerializeError) -> LogKvError {
        LogKvError::SerializeError(err)
    }
}

impl From<DeserializeError> for LogKvError {
    fn from(err: DeserializeError) -> LogKvError {
        LogKvError::DeserializeError(err)
    }
}

pub struct LogKv<K, V, T> {
    cursor: T,
    index: HashMap<K, u64>,
    _phantom: PhantomData<V>,
}

impl<K, V, T> LogKv<K, V, T>
    where K: Serialize + Deserialize + Eq + Hash,
          V: Serialize + Deserialize,
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
            let key = match deserialize_from::<T, K>(&mut logkv.cursor, SizeLimit::Infinite) {
                Ok(key) => key,
                Err(DeserializeError::IoError(ref e)) if e.kind() == IoErrorKind::UnexpectedEof => {
                    break;
                }
                Err(e) => return Err(LogKvError::from(e)),
            };

            let position = logkv.cursor.seek(SeekFrom::Current(0))?;
            logkv.index.insert(key, position);
            deserialize_from::<T, V>(&mut logkv.cursor, SizeLimit::Infinite)?;
        }

        Ok(logkv)
    }

    /// Stores a key-value pair, writing to the given Write.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use log_kv::LogKv;
    ///
    /// let mut cursor = Cursor::new(Vec::new());
    /// let mut db : LogKv<String, String, _> = LogKv::create(cursor).unwrap();
    /// db.put("this is a key", "this is a value").unwrap();
    /// let retrieved = db.get("this is a key").unwrap().expect("No value retrieved");
    /// assert_eq!(retrieved, "this is a value");
    /// ```
    pub fn put<L : Into<K>, W : Into<V>>(&mut self, key: L, value: W) -> Result<(), LogKvError> {
        let serialize_key = key.into();
        serialize_into(&mut self.cursor, &serialize_key, SizeLimit::Infinite)?;
        let position = self.cursor.seek(SeekFrom::Current(0))?;
        self.index.insert(serialize_key, position);
        serialize_into(&mut self.cursor, &value.into(), SizeLimit::Infinite)?;
        Ok(())
    }

    /// Retrieves a value previously stored in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use log_kv::LogKv;
    ///
    /// let mut cursor = Cursor::new(Vec::new());
    /// let mut db : LogKv<String, String, _> = LogKv::create(cursor).unwrap();
    /// db.put("A", "a").unwrap();
    /// db.put("B", "b").unwrap();
    /// let retrieved = db.get("A").unwrap().expect("No value
    /// retrieved");
    /// assert_eq!(retrieved, "a");
    ///
    /// let not_found = db.get("C").unwrap();
    /// assert_eq!(not_found.is_some(), false);
    /// ```
    pub fn get<L : Into<K>>(&mut self, key: L) -> Result<Option<V>, LogKvError> {
        return match self.index.get(&key.into()) {
            Some(position) => {
                self.cursor.seek(SeekFrom::Start(*position))?;
                let value = deserialize_from(&mut self.cursor, SizeLimit::Infinite)?;
                Ok(Some(value))
            }
            None => Ok(None),
        };
    }
}
