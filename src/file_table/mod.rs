use std::collections::BTreeMap;

use crate::segment::BytesSegment;

#[derive(Default, Debug, PartialEq)]
pub struct MagiusFile {
    pub segments: Vec<BytesSegment>,
}

#[derive(Debug, PartialEq)]
pub enum FtItem {
    Dir(MagiusDirectory),
    File(MagiusFile),
}

#[derive(Default, Debug, PartialEq)]
pub struct MagiusDirectory {
    pub files: BTreeMap<String, FtItem>,
}

impl MagiusDirectory {
    pub fn get_by_path(&self, mut path: Vec<&str>) -> Option<&FtItem> {
        let subpath = path.remove(0);
        if path.len() == 0 {
            return self.files.get(subpath);
        }
        match self.files.get(subpath) {
            Some(item) => match item {
                FtItem::Dir(dir) => dir.get_by_path(path),
                FtItem::File(_) => None,
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
