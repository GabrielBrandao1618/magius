use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use super::segment::BytesSegment;

pub struct MagiusFsIo<F: Read + Write + Seek> {
    pub file: F,
    current_offset: usize,
}

impl<F: Read + Write + Seek> MagiusFsIo<F> {
    pub fn new(mut f: F) -> Self {
        let mut current_offset = 0;
        if let Ok(data_length) = f.seek(SeekFrom::End(0)) {
            current_offset = data_length as usize;
        }
        Self {
            file: f,
            current_offset,
        }
    }
    pub fn alloc(&mut self, data: &[u8]) -> std::io::Result<BytesSegment> {
        let mut w = BufWriter::new(&mut self.file);
        w.seek(SeekFrom::Start(self.current_offset as u64))?;
        let data_length = w.write(&data)?;
        let bytes_segment = (self.current_offset, data_length).into();
        self.current_offset += data_length;
        Ok(bytes_segment)
    }
    pub fn read_segment(&mut self, segment: BytesSegment) -> std::io::Result<Vec<u8>> {
        let mut r = BufReader::new(&mut self.file);
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
        let mut magius_fs = MagiusFsIo::new(v_hd);
        let a = "testing".as_bytes();
        let written_segment = magius_fs.alloc(a).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, a);
        magius_fs.alloc(a).unwrap();
        magius_fs.alloc(a).unwrap();
        magius_fs.alloc(a).unwrap();

        let b = "aaaaa".as_bytes();
        let written_segment = magius_fs.alloc(b).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, b);
    }
    #[test]
    fn test_init_from_already_existing() {
        let v_hd = Cursor::<Vec<u8>>::new(Vec::new());
        let mut magius_fs = MagiusFsIo::new(v_hd);
        let a = "testing".as_bytes();
        let written_segment = magius_fs.alloc(a).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, a);

        let written_hd = std::mem::replace(&mut magius_fs.file, Cursor::<Vec<u8>>::new(Vec::new()));
        drop(magius_fs);

        let mut magius_fs = MagiusFsIo::new(written_hd);
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, a);

        let b = "foo".as_bytes();
        let written_segment = magius_fs.alloc(b).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, b);
    }
}
