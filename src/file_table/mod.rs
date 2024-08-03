use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::segment::BytesSegment;

pub struct FileTable<F: Read + Write> {
    root_dir: MagiusDirectory,
    file: F,
}

impl<F: Read + Write> FileTable<F> {
    pub fn new(mut f: F) -> Self {
        let root_dir = Self::read_root_dir_file(&mut f).unwrap_or(MagiusDirectory::new());
        Self { root_dir, file: f }
    }
    fn read_root_dir_file(f: &mut F) -> Option<MagiusDirectory> {
        let mut dir_bytes = Vec::new();
        let _ = f.read_to_end(&mut dir_bytes);
        let decoding_result = bincode::deserialize::<MagiusDirectory>(&dir_bytes);
        if let Ok(decoded) = decoding_result {
            return Some(decoded);
        }
        None
    }
    pub fn insert_in_path(&mut self, path: Vec<&str>, item: FtItem) {
        self.root_dir.insert_in_path(path, item);
    }
    pub fn get_by_path(&self, path: Vec<&str>) -> Option<&FtItem> {
        self.root_dir.get_by_path(path)
    }
    pub fn get_mut_by_path(&mut self, path: Vec<&str>) -> Option<&mut FtItem> {
        self.root_dir.get_mut_by_path(path)
    }
}

impl<F: Read + Write> Drop for FileTable<F> {
    fn drop(&mut self) {
        let encoding_result = bincode::serialize(&self.root_dir);
        if let Ok(encoded) = encoding_result {
            let _ = self.file.write_all(&encoded);
        }
    }
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MagiusFile {
    pub segments: Vec<BytesSegment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FtItem {
    Dir(MagiusDirectory),
    File(MagiusFile),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MagiusDirectory {
    pub files: BTreeMap<String, FtItem>,
}

impl MagiusDirectory {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }
    pub fn get_by_path(&self, mut path: Vec<&str>) -> Option<&FtItem> {
        let subpath = path.remove(0);
        if path.len() == 0 {
            return self.files.get(subpath);
        }
        if let Some(item) = self.files.get(subpath) {
            if let FtItem::Dir(dir) = item {
                return dir.get_by_path(path);
            }
        }
        None
    }
    pub fn get_mut_by_path(&mut self, mut path: Vec<&str>) -> Option<&mut FtItem> {
        let subpath = path.remove(0);
        if path.len() == 0 {
            return self.files.get_mut(subpath);
        }
        if let Some(item) = self.files.get_mut(subpath) {
            if let FtItem::Dir(dir) = item {
                return dir.get_mut_by_path(path);
            }
        }
        None
    }
    pub fn insert_in_path(&mut self, mut path: Vec<&str>, item: FtItem) {
        let subpath = path.remove(0);
        if path.len() == 0 {
            self.files.insert(subpath.to_owned(), item);
            return;
        }
        if let Some(sub_item) = self.files.get_mut(subpath) {
            if let FtItem::Dir(sub_dir) = sub_item {
                sub_dir.insert_in_path(path, item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file() {
        let mut dir = MagiusDirectory::new();
        dir.files
            .insert("data.txt".to_owned(), FtItem::File(MagiusFile::default()));
        let found = dir.get_by_path(vec!["data.txt"]);
        assert_ne!(found, None);
        assert_eq!(found, Some(&FtItem::File(MagiusFile::default())));

        let found = dir.get_by_path(vec!["items"]);
        assert_eq!(found, None);

        let found = dir.get_by_path(vec!["items", "data.txt"]);
        assert_eq!(found, None);

        let found = dir.get_by_path(vec!["data.txt", "sub.txt"]);
        assert_eq!(found, None);
    }

    #[test]
    fn test_get_dir() {
        let mut dir = MagiusDirectory::new();
        let mut sub_dir = MagiusDirectory::new();
        sub_dir
            .files
            .insert("data.txt".to_owned(), FtItem::File(MagiusFile::default()));
        dir.files.insert("items".to_owned(), FtItem::Dir(sub_dir));

        let found = dir.get_by_path(vec!["items", "data.txt"]);
        assert_ne!(found, None);
        assert_eq!(found, Some(&FtItem::File(MagiusFile::default())));
    }

    #[test]
    fn test_insert_in_dir() {
        let mut dir = MagiusDirectory::new();
        dir.insert_in_path(vec!["items"], FtItem::Dir(MagiusDirectory::new()));
        assert_eq!(
            dir.get_by_path(vec!["items"]),
            Some(&FtItem::Dir(MagiusDirectory::new()))
        );

        dir.insert_in_path(
            vec!["items", "data.txt"],
            FtItem::File(MagiusFile::default()),
        );
        assert_eq!(
            dir.get_by_path(vec!["items", "data.txt"]),
            Some(&FtItem::File(MagiusFile::default()))
        );
    }
    #[test]
    fn test_get_mut_file() {
        let mut dir = MagiusDirectory::new();
        dir.insert_in_path(vec!["items"], FtItem::Dir(MagiusDirectory::new()));
        dir.insert_in_path(
            vec!["items", "data.txt"],
            FtItem::File(MagiusFile::default()),
        );
        let mut_file = dir.get_mut_by_path(vec!["items", "data.txt"]);
        assert_eq!(mut_file, Some(&mut FtItem::File(MagiusFile::default())));
    }
}
