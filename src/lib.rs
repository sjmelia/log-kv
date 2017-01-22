extern crate bincode;
extern crate rustc_serialize;
extern crate uuid;

mod dberror;

use bincode::SizeLimit;
use bincode::rustc_serialize::encoded_size;
use bincode::rustc_serialize::encode_into;
use dberror::DbError;
use rustc_serialize::Encodable;
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
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        let db = Db {
            file: file
        };

        Ok(db)
    }

    pub fn put<T: Encodable>(&mut self, key: Uuid, value: T) -> Result<(), DbError> {
        self.file.seek(SeekFrom::End(0))?;
        let len = encoded_size(&key)
            + encoded_size(&value);
        encode_into(&len, &mut self.file, SizeLimit::Infinite)?;
        encode_into(&key, &mut self.file, SizeLimit::Infinite)?;
        encode_into(&value, &mut self.file, SizeLimit::Infinite)?;
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
        db.put(Uuid::new_v4(), "this is a test transmission").unwrap();
    }
}
