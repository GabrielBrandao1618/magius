use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use super::segment::BytesSegment;

pub struct MagiusFsIo<F: Read + Write + Seek> {
    file: F,
    current_offset: usize,
}

impl<F: Read + Write + Seek> MagiusFsIo<F> {
    pub fn new(f: F, current_offset: usize) -> Self {
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
        let mut magius_fs = MagiusFsIo::new(v_hd, 0);
        let test_bytes = "testing".as_bytes();
        let written_segment = magius_fs.alloc(test_bytes).unwrap();
        let readed_bytes = magius_fs.read_segment(written_segment).unwrap();
        assert_eq!(readed_bytes, test_bytes);
    }
}
