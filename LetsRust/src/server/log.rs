use std::sync::Mutex;

#[derive(Debug)]
pub enum LogError {
    ReadError,
    WriteError,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) value: Vec<u8>,
    pub(crate) offset: Option<usize>,
}

struct LogState {
    records: Vec<Record>,
}

pub(crate) struct LogMutex {
    state: Mutex<LogState>,
}

impl LogMutex {
    pub(crate) fn new() -> Self {
        LogMutex {
            state: Mutex::new(LogState { records: vec![] }),
        }
    }

    pub(crate) fn append(&self, mut rec: Record) -> Result<usize, LogError> {
        let mut state = self.state.lock().map_err(|_| LogError::WriteError)?;
        let current_offset = state.records.len();
        rec.offset = Some(current_offset);
        state.records.push(rec);
        Ok(current_offset)
    }

    pub(crate) fn read(&self, offset: usize) -> Result<Record, LogError> {
        let state = self.state.lock().map_err(|_| LogError::ReadError)?;
        if offset > state.records.len() {
            return Err(LogError::WriteError);
        }
        let record = state.records[offset].clone();
        return Ok(record);
    }
}
