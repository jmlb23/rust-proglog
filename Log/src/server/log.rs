use std::fs::File;
use std::io::{self, BufReader, BufWriter, Error, Read, Seek, Write};
use std::sync::Mutex;

struct Store {
    file: File,
}

pub struct MutexStore {
    mu: Mutex<Store>,
}

impl MutexStore {
    pub fn new(file: File) -> io::Result<MutexStore> {
        Ok(MutexStore {
            mu: Mutex::new(Store { file: file }),
        })
    }

    pub fn append(&self, data: &[u8]) -> io::Result<u64> {
        let guard = self
            .mu
            .lock()
            .map_err(|err| Error::other(err.to_string()))?;

        let mut buff_writter = BufWriter::new(&guard.file);

        let written = buff_writter
            .write_all(&data.len().to_be_bytes())
            .and_then(|_| buff_writter.write_all(data))
            .and_then(|_| buff_writter.flush().map(|_| data.len()))?;
        Ok(written as u64)
    }

    pub fn read(&self, pos: u64) -> io::Result<Vec<u8>> {
        let store = self
            .mu
            .lock()
            .map_err(|err| Error::other(err.to_string()))?;
        let mut buff = &store.file;
        let mut buff_read = BufReader::new(buff);
        let read = buff
            .flush()
            .and_then(|_| buff_read.seek(io::SeekFrom::Start(pos)))
            .and_then(|_| {
                let mut buf = [0u8; 8];
                buff_read
                    .read_exact(&mut buf)
                    .map(|_| u64::from_be_bytes(buf) as usize)
            })
            .and_then(|rec_len| {
                let mut buf = vec![0u8; rec_len];
                buff_read.read_exact(&mut buf).map(|_| buf)
            })?;
        Ok(read)
    }

    pub fn close(&self) -> io::Result<()> {
        let mut store = self
            .mu
            .lock()
            .map_err(|err| Error::other(err.to_string()))?;
        store.file.flush()?;
        store.file.sync_all()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::fs::{self, OpenOptions};

    #[test]
    fn test_append_read() -> Result<(), Error> {
        let path = "/tmp/log.log";
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .and_then(|f| MutexStore::new(f))
            .map_err(|err| Error::other(err.to_string()))?;

        let write_vec = vec![1, 2];

        let write_res = file.append(write_vec.as_slice())?;

        let response = file.read(0)?;

        assert_ne!(
            write_res, 0,
            "should not be equal to 0 if something is written"
        );
        assert_eq!(write_vec, response, "should be the same");

        let _ = fs::remove_file(path);
        Ok(())
    }
}
