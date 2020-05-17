use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

pub fn file_is_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_dir())
}

pub fn get_time_created<P: AsRef<Path>>(path: P) -> anyhow::Result<DateTime<Local>> {
    let metadata = fs::metadata(path)?;
    let time = metadata.created();

    match time {
        Ok(time) => Ok(DateTime::<Local>::from(time)),
        Err(e) => Err(e.into()),
    }
}

pub fn get_file_contents<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let contents = fs::read_to_string(&path)?;
    Ok(contents.trim().to_string())
}
