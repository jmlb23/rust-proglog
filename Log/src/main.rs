use std::fs::OpenOptions;

use server::log::MutexStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "/tmp/log.log";
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(path)
        .and_then(|f| MutexStore::new(f))?;

    let write_vec = vec![1, 2];

    let write_res = file.append(write_vec.as_slice())?;
    Ok(())
}
