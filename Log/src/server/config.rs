#[derive(Clone, Copy)]
pub struct Segment {
    pub max_store_bytes: u64,
    pub max_index_bytes: u64,
    pub initial_offset: u64,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub segment: Segment,
}

impl Config {
    pub fn new(seg: (u64, u64, u64)) -> Self {
        Config {
            segment: Segment {
                max_store_bytes: seg.0,
                max_index_bytes: seg.1,
                initial_offset: seg.2,
            },
        }
    }
}

