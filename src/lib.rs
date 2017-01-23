extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

mod dberror;

use bincode::SizeLimit;
use bincode::rustc_serialize::encode_into;
use bincode::rustc_serialize::decode_from;
use dberror::DbError;
use rustc_serialize::Encodable;
use rustc_serialize::Decodable;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::marker::PhantomData;
use uuid::Uuid;

pub struct Db<T, K> {
    cursor: K,
    _phantom: PhantomData<T>,
}

impl<T, K> Db<T, K> where
    T: Encodable + Decodable,
    K: Read + Write + Seek {

    pub fn create(mut cursor: K) -> Result<Db<T, K>, DbError> {
        cursor.seek(SeekFrom::Start(0))?;
        let mut buf = vec![0; 1];
        let read = cursor.read(&mut buf)?;

        if read == 0 {
            // file is empty, initialize file with zero records
            cursor.seek(SeekFrom::Start(0))?;
            encode_into(&read, &mut cursor, SizeLimit::Infinite)?;
        }

        let db = Db {
            cursor: cursor,
            _phantom: PhantomData,
        };

        Ok(db)
    }

    pub fn put(&mut self, key: Uuid, value: T) -> Result<(), DbError> {
        self.cursor.seek(SeekFrom::End(0))?;
        encode_into(&key, &mut self.cursor, SizeLimit::Infinite)?;
        encode_into(&value, &mut self.cursor, SizeLimit::Infinite)?;
        self.cursor.seek(SeekFrom::Start(0))?;
        let record_count : u64 = decode_from(&mut self.cursor, SizeLimit::Infinite)?;
        self.cursor.seek(SeekFrom::Start(0))?;
        encode_into(&(record_count + 1), &mut self.cursor, SizeLimit::Infinite)?;
        Ok(())
    }

    pub fn get(&mut self, key: Uuid) -> Result<Option<T>, DbError> {
        self.cursor.seek(SeekFrom::Start(0))?;
        let record_count : u64 = decode_from(&mut self.cursor, SizeLimit::Infinite)?;

        for _ in 0..record_count {
            let read_key : Uuid = decode_from(&mut self.cursor, SizeLimit::Infinite)?;
            let value : T = decode_from(&mut self.cursor, SizeLimit::Infinite)?;

            if key == read_key {
                return Ok(Some(value))
            }
        }

        return Ok(None)
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
