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
