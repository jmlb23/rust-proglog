use memmap2::{MmapMut, MmapOptions};
use std::fs::{File, OpenOptions};
use std::io::ErrorKind::UnexpectedEof;
use std::io::{self, Error, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

const OFF_WIDTH: u64 = 4;
const POS_WIDTH: u64 = 8;
const ENT_WIDTH: u64 = OFF_WIDTH + POS_WIDTH;

pub struct Index {
    file: Box<Path>,
    size: u64,
    mmap: MmapMut,
}

pub struct Segment {
    max_store_bytes: u64,
    max_index_bytes: u64,
    initial_offset: u64,
}

pub struct Config {
    segment: Segment,
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

impl Index {
    fn file_helper(file: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(file.to_str().unwrap_or(""))
    }
    pub fn new(f: Box<Path>, c: Config) -> io::Result<Self> {
        let size = f.metadata()?.size();
        let file = Index::file_helper(&f)?;
        //unsafe rust can't warrant that other process truncates it
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        //truncate file to index size
        file.set_len(c.segment.max_index_bytes)?;

        let index = Index {
            file: f,
            size,
            mmap,
        };

        Ok(index)
    }

    fn close(&self) -> io::Result<()> {
        self.mmap.flush()?;
        let file = Index::file_helper(&self.file)?;
        file.sync_all()?;
        file.set_len(self.size)?;
        Ok(())
    }

    fn read(&self, n: i64) -> io::Result<(u32, u64)> {
        if self.size == 0 {
            return Err(Error::from(UnexpectedEof));
        }
        let out = if n == -1 {
            (self.size / ENT_WIDTH) - 1
        } else {
            u64::try_from(n).unwrap_or(0)
        };

        let pos = out * ENT_WIDTH;

        if self.size < u64::try_from(pos + ENT_WIDTH).unwrap_or(0) {
            return Err(Error::from(UnexpectedEof));
        }
        let out_endian = u32::from_be_bytes(
            self.mmap[pos as usize..(pos + OFF_WIDTH) as usize]
                .try_into()
                .map_err(|err| Error::other(err))?,
        );
        let pos_endian = u64::from_be_bytes(
            self.mmap[(pos + OFF_WIDTH) as usize..(pos + ENT_WIDTH) as usize]
                .try_into()
                .map_err(|err| Error::other(err))?,
        );

        Ok((out_endian, pos_endian))
    }

    fn write(&mut self, off: u32, pos: u64) -> io::Result<()> {
        if (self.mmap.len() as u64) < self.size + ENT_WIDTH {
            return Err(Error::from(UnexpectedEof));
        }
        self.mmap[self.size as usize..(self.size + ENT_WIDTH) as usize]
            .copy_from_slice(&off.to_be_bytes());
        self.mmap[(self.size + OFF_WIDTH) as usize..(self.size + ENT_WIDTH) as usize]
            .copy_from_slice(&pos.to_be_bytes());
        self.size += ENT_WIDTH;
        Ok(())
    }

    fn name(&self) -> io::Result<Option<&str>> {
        Ok(self.file.to_str())
    }
}
