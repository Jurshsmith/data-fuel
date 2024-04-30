use std::ops::Range;

pub struct BlockHeightRangeChunks {
    next_range: Range<u32>,
    chunk_size: u32,
}

impl BlockHeightRangeChunks {
    pub fn new(next_range: Range<u32>, chunk_size: u32) -> Self {
        Self {
            next_range,
            chunk_size,
        }
    }
}

impl Iterator for BlockHeightRangeChunks {
    type Item = Range<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_range.start >= self.next_range.end {
            None
        } else {
            let next_start = self.next_range.start;
            let next_end = next_start + self.chunk_size;

            self.next_range.start = next_end;

            Some(next_start..next_end)
        }
    }
}
