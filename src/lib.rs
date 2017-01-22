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
use std::fs;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use uuid::Uuid;

pub struct Db {
    file: File,
}

impl Db {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Db, DbError> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let len = file.metadata()?.len();
        if len == 0 {
            // initialize file with zero records
            encode_into(&len, &mut file, SizeLimit::Infinite)?;
        }

        let db = Db {
            file: file
        };

        Ok(db)
    }

    pub fn put<T: Encodable>(&mut self, key: Uuid, value: T) -> Result<(), DbError> {
        self.file.seek(SeekFrom::End(0))?;
        encode_into(&key, &mut self.file, SizeLimit::Infinite)?;
        encode_into(&value, &mut self.file, SizeLimit::Infinite)?;
        self.file.seek(SeekFrom::Start(0))?;
        let record_count : u64 = decode_from(&mut self.file, SizeLimit::Infinite)?;
        self.file.seek(SeekFrom::Start(0))?;
        encode_into(&(record_count + 1), &mut self.file, SizeLimit::Infinite)?;
        Ok(())
    }

    pub fn get<T: Decodable + std::fmt::Display>(&mut self, key: Uuid) -> Result<Option<T>, DbError> {
        self.file.seek(SeekFrom::Start(0))?;
        let record_count : u64 = decode_from(&mut self.file, SizeLimit::Infinite)?;

        for _ in 0..record_count {
            let read_key : Uuid = decode_from(&mut self.file, SizeLimit::Infinite)?;
            let value : T = decode_from(&mut self.file, SizeLimit::Infinite)?;

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

    #[test]
    fn put_then_get_returns_expected() {
        use std::string::String;
        use std::fs::remove_file;

        let mut db = Db::create("put_then_get_returns_expected").unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(key, value).unwrap();
        let retrieved = db.get::<String>(key).unwrap().expect("No value retrieved");
        remove_file("put_then_get_returns_expected").unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn put_twice_then_get_returns_expected() {
        use std::string::String;
        use std::fs::remove_file;

        let mut db = Db::create("put_put_then_get_returns_expected").unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(Uuid::new_v4(), "valueA").unwrap();
        db.put(key, value).unwrap();
        let retrieved = db.get::<String>(key).unwrap().expect("No value retrieved");
        remove_file("put_put_then_get_returns_expected").unwrap();
        assert_eq!(retrieved, value);
    }

    #[test]
    fn get_returns_not_found() {
        use std::string::String;
        use std::fs::remove_file;

        let mut db = Db::create("get_returns_not_found").unwrap();
        let key = Uuid::new_v4();
        let value = "this is a test transmission";
        db.put(Uuid::new_v4(), "valueA").unwrap();
        db.put(key, value).unwrap();
        let retrieved = db.get::<String>(Uuid::new_v4()).unwrap();
        remove_file("get_returns_not_found").unwrap();
        assert_eq!(retrieved.is_some(), false);
    }

}
