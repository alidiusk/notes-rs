#![allow(dead_code)]

mod app;
mod errors;
mod notes;
mod print;
mod util;

use std::fs;
use std::path::PathBuf;

use dirs::data_dir;
use structopt::StructOpt;

use crate::app::{run_app, App};
use crate::notes::Notes;

fn main() -> anyhow::Result<()> {
    let app = App::from_args();
    let path = get_notes_file_path()?;

    let mut notes = Notes::from_file(&path)?;
    run_app(app, &mut notes)?;
    notes.to_file(&path)?;

    Ok(())
}

/// Returns the path of the notes file; creates it if it does not
/// exist, and enters an empty Notes struct into it.
fn get_notes_file_path() -> anyhow::Result<PathBuf> {
    let dir = data_dir().unwrap().join("Notes");

    if !dir.exists() {
        fs::create_dir(&dir)?;
    }

    let path = dir.join("notes");

    if !path.exists() {
        let notes = Notes::new(vec![]);
        notes.to_file(&path)?;
    }

    Ok(path)
}
