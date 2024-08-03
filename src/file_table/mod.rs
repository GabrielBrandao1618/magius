use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use crate::segment::BytesSegment;

pub struct FileTable<F: Read + Write> {
    root_dir: MagiusDirectory,
    file: F,
}

impl<F: Read + Write> FileTable<F> {
    pub fn new(f: F) -> Self {
        Self {
            root_dir: MagiusDirectory::new(),
            file: f,
        }
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

#[derive(Default, Debug, PartialEq)]
pub struct MagiusFile {
    pub segments: Vec<BytesSegment>,
}

#[derive(Debug, PartialEq)]
pub enum FtItem {
    Dir(MagiusDirectory),
    File(MagiusFile),
}

#[derive(Debug, PartialEq)]
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
