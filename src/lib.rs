use std::io::{self, BufReader, BufWriter, Read, Seek, Write};

pub struct BytesSegment {
    pub offset: usize,
    pub length: usize,
}

impl From<(usize, usize)> for BytesSegment {
    fn from(value: (usize, usize)) -> Self {
        let (offset, length) = value;
        Self { offset, length }
    }
}

pub struct MagiusFs<F: Read + Write + Seek> {
    file: F,
    current_offset: usize,
}

impl<F: Read + Write + Seek> MagiusFs<F> {
    pub fn new(f: F, current_offset: usize) -> Self {
        Self {
            file: f,
            current_offset,
        }
    }
    pub fn alloc(&mut self, data: &[u8]) -> io::Result<BytesSegment> {
        let mut w = BufWriter::new(&mut self.file);
        w.seek(io::SeekFrom::Start(self.current_offset as u64))?;
        let data_length = w.write(&data)?;
        let bytes_segment = (self.current_offset, data_length).into();
        self.current_offset += data_length;
        Ok(bytes_segment)
    }
    pub fn read_segment(&mut self, segment: BytesSegment) -> io::Result<Vec<u8>> {
        let mut r = BufReader::new(&mut self.file);
        r.seek(io::SeekFrom::Start(segment.offset as u64))?;
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
        let mut magius_fs = MagiusFs::new(v_hd, 0);
        let test_bytes = "testing".as_bytes();
        let written_segment = magius_fs.alloc(test_bytes).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, test_bytes);
    }
}
