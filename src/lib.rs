extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

mod dberror;

use bincode::SizeLimit;
use bincode::rustc_serialize::encode_into;
use bincode::rustc_serialize::decode_from;
use bincode::rustc_serialize::DecodingError;
use dberror::DbError;
use rustc_serialize::Encodable;
use rustc_serialize::Decodable;
use std::collections::hash_map::HashMap;
use std::io::ErrorKind as IoErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::marker::PhantomData;
use uuid::Uuid;

pub struct Db<T, K> {
    cursor: K,
    index: HashMap<Uuid, u64>,
    _phantom: PhantomData<T>,
}

impl<T, K> Db<T, K> where
    T: Encodable + Decodable,
    K: Read + Write + Seek {

    pub fn create(cursor: K) -> Result<Db<T, K>, DbError> {
        let mut db = Db {
            cursor: cursor,
            index: HashMap::new(),
            _phantom: PhantomData,
        };

        db.cursor.seek(SeekFrom::Start(0))?;
        loop {
            let read_key : Uuid = match decode_from(&mut db.cursor, SizeLimit::Infinite) {
                Ok(read_key) => read_key,
                Err(DecodingError::IoError(ref e)) if e.kind() == IoErrorKind::UnexpectedEof  => {
                    break;
                },
                Err(e) => return Err(DbError::from(e))
            };

            let position = db.cursor.seek(SeekFrom::Current(0))?;
            db.index.insert(read_key, position);
            decode_from::<K, T>(&mut db.cursor, SizeLimit::Infinite)?;

            //println!("Read {}:{}", read_key, value);
            /*match decode_from(&mut db.cursor, SizeLimit::Infinite) {
                Ok(val) =>  println!("Read {}:{}", read_key, val),
                Err(e) => println!("no err")
            };*/
        }

        Ok(db)
    }

    pub fn put(&mut self, key: Uuid, value: T) -> Result<(), DbError> {
        encode_into(&key, &mut self.cursor, SizeLimit::Infinite)?;
        let position = self.cursor.seek(SeekFrom::Current(0))?;
        self.index.insert(key, position);
        encode_into(&value, &mut self.cursor, SizeLimit::Infinite)?;
        Ok(())
    }

    pub fn get(&mut self, key: Uuid) -> Result<Option<T>, DbError> {
        return match self.index.get(&key) {
            Some(position) => {
                self.cursor.seek(SeekFrom::Start(*position))?;
                let value = decode_from(&mut self.cursor, SizeLimit::Infinite)?;
                Ok(Some(value))
            },
            None => Ok(None),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Db;
    use uuid::Uuid;
    use std::string::String;
    use std::fs::remove_file;
    use std::fs;

    #[test]
    fn put_then_get_returns_expected() {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("put_then_get_returns_expected")
            .unwrap();

        let mut db = Db::create(file).unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(key, String::from(value)).unwrap();
        let retrieved = db.get(key).unwrap().expect("No value retrieved");
        remove_file("put_then_get_returns_expected").unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn put_twice_then_get_returns_expected() {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("put_twice_then_get_returns_expected")
            .unwrap();

        let mut db = Db::create(file).unwrap();
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

        let mut db = Db::create(file).unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(Uuid::new_v4(), String::from("valueA")).unwrap();
        db.put(key, String::from(value)).unwrap();
        let retrieved = db.get(Uuid::new_v4()).unwrap();
        remove_file("get_returns_not_found").unwrap();
        assert_eq!(retrieved.is_some(), false);
    }
}
