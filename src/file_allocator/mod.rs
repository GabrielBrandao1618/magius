use std::io::{Read, Seek, Write};

use crate::file_table::{FileTable, FtItem, MagiusDirectory, MagiusFile};
use crate::io::MagiusFsIo;

pub struct FileAllocator<'a, F: Read + Write + Seek> {
    fs_io: MagiusFsIo<F>,
    file_table: FileTable<'a, F>,
}

impl<'a, F: Read + Write + Seek> FileAllocator<'a, F> {
    pub fn new(fs_io: MagiusFsIo<F>, file_table: FileTable<'a, F>) -> Self {
        Self { fs_io, file_table }
    }
    pub fn create_file(&mut self, path: Vec<&str>) {
        self.file_table
            .insert_in_path(path, FtItem::File(MagiusFile::default()));
    }
    pub fn create_dir(&mut self, path: Vec<&str>) {
        self.file_table
            .insert_in_path(path, FtItem::Dir(MagiusDirectory::default()));
    }
    pub fn write_file_by_path(&mut self, path: Vec<&str>, data: &[u8]) -> std::io::Result<()> {
        let written_block = self.fs_io.push(data)?;
        let target_file = self.get_file_mut(path).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        ))?;
        if let FtItem::File(file) = target_file {
            file.blocks.push(written_block);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot write to a directory",
            ))
        }
    }
    pub fn read_entire_file_by_path(
        &self,
        path: Vec<&str>,
        buf: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        let found_item = self.get_file(path);
        if let Some(FtItem::File(file)) = found_item {
            for block in &file.blocks {
                let readed = self.fs_io.read_block(*block)?;
                buf.extend(readed);
            }
        }
        Ok(())
    }
    pub fn get_file(&self, path: Vec<&str>) -> Option<&FtItem> {
        self.file_table.get_by_path(path)
    }
    pub fn get_file_mut(&mut self, path: Vec<&str>) -> Option<&mut FtItem> {
        self.file_table.get_mut_by_path(path)
    }
    pub fn read_entire_file(&self, file: &MagiusFile, buf: &mut Vec<u8>) -> std::io::Result<()> {
        for block in &file.blocks {
            let readed = self.fs_io.read_block(*block)?;
            buf.extend(readed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        file_allocator::FileAllocator,
        file_table::{FileTable, FtItem},
        io::MagiusFsIo,
    };

    #[test]
    fn test_rw_file() {
        let f = Cursor::<Vec<u8>>::new(vec![]);
        let mut table_file = Cursor::new(Vec::new());
        let file_table = FileTable::new(&mut table_file);
        let mut magius = FileAllocator::new(MagiusFsIo::new(f, 1024), file_table);
        magius.create_dir(vec!["items"]);
        magius.create_file(vec!["items", "data.txt"]);

        let content = "content";

        magius
            .write_file_by_path(vec!["items", "data.txt"], content.as_bytes())
            .unwrap();
        let found_file = magius.get_file(vec!["items", "data.txt"]).unwrap();
        if let FtItem::File(f) = found_file {
            let mut buf = Vec::new();
            magius.read_entire_file(f, &mut buf).unwrap();
            let parsed_readed = String::from_utf8(buf).unwrap();
            assert_eq!(parsed_readed[0..content.len()], content.to_owned());
        } else {
            assert!(false);
        }
    }
}
