use std::{
    fmt::format,
    fs::{File, OpenOptions},
    io,
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
};

use log_rust_proto::protos_log_proto;

use crate::{
    config::Config,
    index::{self, Index},
    log::{self, MutexStore},
};

struct Segment {
    store: MutexStore,
    index: Index,
    base_offset: u64,
    next_offset: u64,
    config: Config,
}

impl Segment {
    fn file_helper(dir: &str, base: u64, ext: &str) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .mode(0o644)
            .open(format(format_args!("{}{}{}", dir, base, ext)))
    }

    pub fn new(dir: &str, base: u64, c: Config) -> io::Result<Self> {
        let file_store = Segment::file_helper(dir, base, ".store")?;
        let m_store = MutexStore::new(file_store)?;

        let index = Index::new(
            PathBuf::from(format(format_args!("{}{}{}", dir, base, ".index"))).into_boxed_path(),
            c,
        )?;

        let next = match index.read(-1) {
            Ok(read) => base + read.0 as u64 + 1,
            Err(_) => base,
        };

        Ok(Segment {
            store: m_store,
            index,
            base_offset: base,
            next_offset: next,
            config: c,
        })
    }

    pub fn append(&self, rec: protos_log_proto::Record) {

    }
}
