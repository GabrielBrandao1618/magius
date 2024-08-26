use std::{
    collections::BTreeMap,
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
};

use serde::{Deserialize, Serialize};

pub struct FileTable<'a, F: Read + Write + Seek> {
    root_dir: MagiusDirectory,
    file: &'a mut F,
}

impl<'a, F: Read + Write + Seek> FileTable<'a, F> {
    pub fn new(f: &'a mut F) -> Self {
        let root_dir = Self::read_root_dir_file(f).unwrap_or(MagiusDirectory::default());
        Self { root_dir, file: f }
    }
    fn read_root_dir_file(f: &mut F) -> std::io::Result<MagiusDirectory> {
        let mut dir_bytes = Vec::new();
        let _ = f.seek(SeekFrom::Start(0));
        let bytes_read = f.read_to_end(&mut dir_bytes)?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(ErrorKind::Other, "Could not read file"));
        }
        match bincode::deserialize::<MagiusDirectory>(&dir_bytes) {
            Ok(decoded) => Ok(decoded),
            Err(_) => Err(std::io::Error::new(
                ErrorKind::Other,
                "Could not parse data",
            )),
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

impl<F: Read + Write + Seek> Drop for FileTable<'_, F> {
    fn drop(&mut self) {
        let encoding_result = bincode::serialize(&self.root_dir);
        if let Ok(encoded) = encoding_result {
            let _ = self.file.write_all(&encoded);
        }
    }
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MagiusFile {
    pub blocks: Vec<u64>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FtItem {
    Dir(MagiusDirectory),
    File(MagiusFile),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MagiusDirectory {
    pub files: BTreeMap<String, FtItem>,
}

impl MagiusDirectory {
    pub fn get_by_path(&self, mut path: Vec<&str>) -> Option<&FtItem> {
        let subpath = path.remove(0);
        if path.is_empty() {
            return self.files.get(subpath);
        }
        if let Some(FtItem::Dir(dir)) = self.files.get(subpath) {
            return dir.get_by_path(path);
        }
        None
    }
    pub fn get_mut_by_path(&mut self, mut path: Vec<&str>) -> Option<&mut FtItem> {
        let subpath = path.remove(0);
        if path.is_empty() {
            return self.files.get_mut(subpath);
        }
        if let Some(FtItem::Dir(dir)) = self.files.get_mut(subpath) {
            return dir.get_mut_by_path(path);
        }
        None
    }
    pub fn insert_in_path(&mut self, mut path: Vec<&str>, item: FtItem) {
        let subpath = path.remove(0);
        if path.is_empty() {
            self.files.insert(subpath.to_owned(), item);
            return;
        }
        if let Some(FtItem::Dir(sub_dir)) = self.files.get_mut(subpath) {
            sub_dir.insert_in_path(path, item);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_get_file() {
        let mut dir = MagiusDirectory::default();
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
        let mut dir = MagiusDirectory::default();
        let mut sub_dir = MagiusDirectory::default();
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
        let mut dir = MagiusDirectory::default();
        dir.insert_in_path(vec!["items"], FtItem::Dir(MagiusDirectory::default()));
        assert_eq!(
            dir.get_by_path(vec!["items"]),
            Some(&FtItem::Dir(MagiusDirectory::default()))
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
        let mut dir = MagiusDirectory::default();
        dir.insert_in_path(vec!["items"], FtItem::Dir(MagiusDirectory::default()));
        dir.insert_in_path(
            vec!["items", "data.txt"],
            FtItem::File(MagiusFile::default()),
        );
        let mut_file = dir.get_mut_by_path(vec!["items", "data.txt"]);
        assert_eq!(mut_file, Some(&mut FtItem::File(MagiusFile::default())));
    }
    #[test]
    fn test_save_file_table() {
        let mut ft_file = Cursor::new(Vec::new());
        let mut file_table = FileTable::new(&mut ft_file);
        file_table.insert_in_path(vec!["data"], FtItem::Dir(MagiusDirectory::default()));
        drop(file_table);

        let file_table = FileTable::new(&mut ft_file);
        let found_item = file_table.get_by_path(vec!["data"]).unwrap();
        assert_eq!(found_item, &FtItem::Dir(MagiusDirectory::default()));
    }
}
