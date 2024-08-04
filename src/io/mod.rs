use std::{
    cell::RefCell,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
};

use crate::segment::BytesSegment;

pub struct MagiusFsIo<F: Read + Write + Seek> {
    pub file: RefCell<F>,
}

impl<F: Read + Write + Seek> MagiusFsIo<F> {
    pub fn new(f: F) -> Self {
        Self {
            file: RefCell::new(f),
        }
    }
    pub fn alloc(&self, data: &[u8]) -> std::io::Result<BytesSegment> {
        let mut file = self.file.borrow_mut();
        let mut w = BufWriter::new(&mut *file);
        let current_offset = w.stream_position()?;
        let data_length = w.write(data)?;
        let bytes_segment = (current_offset as usize, data_length).into();
        Ok(bytes_segment)
    }
    pub fn read_segment(&self, segment: &BytesSegment) -> std::io::Result<Vec<u8>> {
        let mut file = self.file.borrow_mut();
        let mut r = BufReader::new(&mut *file);
        r.seek(SeekFrom::Start(segment.offset as u64))?;
        let mut buf = vec![0; segment.length];
        r.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_alloc_and_read() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let magius_fs = MagiusFsIo::new(v_hd);
        let a = "testing".as_bytes();
        let written_segment = magius_fs.alloc(a).unwrap();
        let readed_bytes = magius_fs.read_segment(&written_segment).unwrap();
        assert_eq!(readed_bytes, a);
        magius_fs.alloc(a).unwrap();
        magius_fs.alloc(a).unwrap();
        magius_fs.alloc(a).unwrap();

        let b = "aaaaa".as_bytes();
        let written_segment = magius_fs.alloc(b).unwrap();
        let readed_bytes = magius_fs.read_segment(&written_segment).unwrap();
        assert_eq!(readed_bytes, b);
    }
    #[test]
    fn test_init_from_already_existing() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let mut magius_fs = MagiusFsIo::new(v_hd);
        let a = "testing".as_bytes();
        let written_segment = magius_fs.alloc(a).unwrap();
        let readed_bytes = magius_fs.read_segment(&written_segment).unwrap();
        assert_eq!(readed_bytes, a);

        let mut written_hd = std::mem::replace(
            &mut magius_fs.file,
            RefCell::new(Cursor::<Vec<u8>>::new(Vec::new())),
        );
        drop(magius_fs);

        let magius_fs = MagiusFsIo::new(written_hd.get_mut());
        let readed_bytes = magius_fs.read_segment(&written_segment).unwrap();
        assert_eq!(readed_bytes, a);

        let b = "foo".as_bytes();
        let second_segment = magius_fs.alloc(b).unwrap();
        let readed_bytes = magius_fs.read_segment(&second_segment).unwrap();
        assert_eq!(readed_bytes, b);

        let readed_bytes = magius_fs.read_segment(&written_segment).unwrap();
        assert_eq!(readed_bytes, a);
    }
}
