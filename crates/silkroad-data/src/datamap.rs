use crate::{list_files, parse_file, FileError, ParseError};
use pk2_sync::sync::Pk2;
use std::ops::Deref;
use std::str::FromStr;

pub trait DataEntry: FromStr {
    fn ref_id(&self) -> u32;
    fn code(&self) -> &str;
}

pub struct DataMap<T> {
    items: Vec<T>,
}

impl<T> Deref for DataMap<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DataMap<T> {
    pub fn new(items: Vec<T>) -> DataMap<T> {
        Self { items }
    }
}

impl<T: FromStr<Err = ParseError>> DataMap<T> {
    pub fn from(pk2: &Pk2<impl std::io::Read + std::io::Seek>, main_file: &str) -> Result<DataMap<T>, FileError> {
        let mut file = pk2.open_file(main_file)?;
        let lines = list_files(&mut file)?;
        let all_entries: Result<Vec<Vec<T>>, FileError> = lines
            .into_iter()
            .filter(|name| !name.is_empty())
            .map(|filename| format!("/server_dep/silkroad/textdata/{}", filename))
            .map(|filename| {
                let res: Result<Vec<T>, FileError> = pk2
                    .open_file(&filename)
                    .map_err(FileError::from)
                    .and_then(|mut file| parse_file(&mut file));
                res
            })
            .collect();

        Ok(DataMap::new(all_entries?.into_iter().flatten().collect()))
    }
}

impl<T: DataEntry> DataMap<T> {
    pub fn find_id(&self, id: u32) -> Option<&T> {
        self.items.iter().find(|item| item.ref_id() == id)
    }

    pub fn find_code(&self, code: &str) -> Option<&T> {
        self.items.iter().find(|item| item.code() == code)
    }
}
