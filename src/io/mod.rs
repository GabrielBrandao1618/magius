use std::{
    cell::RefCell,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
};

pub struct MagiusFsIo<F: Read + Write + Seek> {
    pub file: RefCell<F>,
    block_size: usize,
    blocks_count: u64,
}

impl<F: Read + Write + Seek> MagiusFsIo<F> {
    pub fn new(mut f: F, block_size: usize) -> Self {
        let mut blocks_count = 0;
        if let Ok(file_size) = f.seek(SeekFrom::End(0)) {
            blocks_count = file_size / block_size as u64;
        }
        Self {
            file: RefCell::new(f),
            block_size,
            blocks_count,
        }
    }
    pub fn read_block(&self, block: u64) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0; self.block_size];
        let mut file = self.file.borrow_mut();
        let mut r = BufReader::new(&mut *file);
        r.seek(SeekFrom::Start(block * self.block_size as u64))?;
        r.read_exact(&mut buf)?;
        Ok(buf.into_iter().take_while(|c| *c != 0).collect())
    }
    pub fn write_block(&self, block: u64, data: &[u8]) -> std::io::Result<()> {
        if data.len() > self.block_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not write to block: data size is bigger than block size",
            ));
        }
        let remaining_space = self.block_size - data.len();
        let mut file = self.file.borrow_mut();
        let mut w = BufWriter::new(&mut *file);
        w.seek(SeekFrom::Start(block * self.block_size as u64))?;
        w.write_all(data)?;
        w.write_all(&vec![0; remaining_space])?;
        Ok(())
    }
    fn write_blocks(&self, block: u64, data: &[u8]) -> std::io::Result<Vec<u64>> {
        let mut written_blocks = vec![];
        let data_len = data.len();
        let needed_blocks = data_len.div_ceil(self.block_size);
        for i in 0..needed_blocks {
            let data_offset = self.block_size * i;
            let data_end = data_len.min(self.block_size * (i + 1));
            self.write_block(block + i as u64, &data[data_offset..data_end])?;
            written_blocks.push(block + i as u64);
        }

        Ok(written_blocks)
    }
    pub fn push(&mut self, data: &[u8]) -> std::io::Result<Vec<u64>> {
        let blocks = self.write_blocks(self.blocks_count, data)?;
        self.blocks_count += blocks.len() as u64;
        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_alloc_and_read() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let mut magius_fs = MagiusFsIo::new(v_hd, 1024);
        let a = "testing".as_bytes();
        magius_fs.push(a).unwrap();
        let readed_bytes = magius_fs.read_block(0).unwrap();
        assert_eq!(&readed_bytes[0..a.len()], a);

        let b = "aaaaa".as_bytes();
        magius_fs.push(b).unwrap();
        let readed_bytes = magius_fs.read_block(1).unwrap();
        assert_eq!(&readed_bytes[0..b.len()], b);
    }
    #[test]
    fn test_init_from_already_existing() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let mut magius_fs = MagiusFsIo::new(v_hd, 1024);
        let a = "testing".as_bytes();
        magius_fs.push(a).unwrap();
        let readed_bytes = magius_fs.read_block(0).unwrap();
        assert_eq!(&readed_bytes[0..a.len()], a);

        let mut written_hd = std::mem::replace(
            &mut magius_fs.file,
            RefCell::new(Cursor::<Vec<u8>>::new(Vec::new())),
        );
        drop(magius_fs);

        let mut magius_fs = MagiusFsIo::new(written_hd.get_mut(), 1024);
        let readed_bytes = magius_fs.read_block(0).unwrap();
        assert_eq!(&readed_bytes[0..a.len()], a);

        let b = "foo".as_bytes();
        magius_fs.push(b).unwrap();
        let readed_bytes = magius_fs.read_block(1).unwrap();
        assert_eq!(&readed_bytes[0..b.len()], b);

        let readed_bytes = magius_fs.read_block(0).unwrap();
        assert_eq!(&readed_bytes[0..a.len()], a);
    }
    #[test]
    fn test_write_blocks() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let mut magius_fs = MagiusFsIo::new(v_hd, 4);
        let a = "abcdefghfff".as_bytes();
        magius_fs.push(a).unwrap();
        let block1 = magius_fs.read_block(0).unwrap();
        let block2 = magius_fs.read_block(1).unwrap();
        assert_eq!(block1, &a[0..4]);
        assert_eq!(block2, &a[4..8]);
    }
}
