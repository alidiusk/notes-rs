#![allow(dead_code)]

mod models;
mod db;
mod print;

use crate::models::Note;
use crate::db::*;
use crate::print::Table;

use chrono::{DateTime, Local};
use directories::ProjectDirs;
use structopt::StructOpt;

use std::path::{Path, PathBuf};
use std::{env, fs, process};

/*

notes new -f note.txt
notes new "Day 12" "Today in my diary..."

*/

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Create a new note.
    #[structopt(name = "new")]
    New {
        /// Create a new note from file
        #[structopt(short, long, conflicts_with = "editor")]
        file: Option<PathBuf>,
        /// Open new file with given title in default editor, use contents to create a new note
        #[structopt(short, long, conflicts_with = "file")]
        editor: bool,
        #[structopt(name = "content", required_unless_one = &["file", "editor"], conflicts_with_all = &["file", "editor"])]
        content: Option<String>,
    },
    /// Retrieve an existing note.
    Get {
        /// Returns all notes
        #[structopt(short, long)]
        all: bool,
        /// Returns the note with the matching id.
        #[structopt(name = "note_id", required_unless = "all")]
        id: Option<i32>,
    },
    /// Edit an existing note.
    Edit {
        #[structopt(name = "note_id")]
        id: i32,
        #[structopt(name = "content")]
        content: String,
    },
    /// Delete an existing note.
    Delete {
        /// Deletes the note with the corresponding id.
        #[structopt(name = "note_id")]
        id: i32,
    },
}

fn file_is_dir<P: AsRef<Path>>(path: P) -> anyhow::Result<bool> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.is_dir())
}

fn get_time_created<P: AsRef<Path>>(path: P) -> anyhow::Result<DateTime<Local>> {
    let metadata = fs::metadata(path)?;
    let time = metadata.created();

    match time {
        Ok(time) => Ok(DateTime::<Local>::from(time)),
        Err(e) => Err(e.into()),
    }
}

fn get_file_contents<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let contents = fs::read_to_string(&path)?;
    Ok(contents.trim().to_string())
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    let project_dir = ProjectDirs::from("", "", "Notes").unwrap();

    let dir = project_dir.data_local_dir();

    if !dir.exists() {
        fs::create_dir(dir)?;
    }

    let path = String::from("sqlite://") 
        + &dir.join(dir).join("notes.db").to_str().unwrap().to_string();
    // let path = std::env::var("DATABASE_URL")?;

    let pool = sqlx::SqlitePool::new(&path).await?;

    match args.cmd {
        Some(Command::New { file, editor, content }) => {
            let note = {
                if let Some(path) = file {
                    if file_is_dir(&path)? {
                        panic!("File is a directory!");
                    }

                    // fakee id
                    let id = 0;
                    let created = get_time_created(&path)?;
                    let content = get_file_contents(&path)?;

                    Note { id, created, content }
                } else if editor {
                    let editor = env::var("EDITOR")?;
                    let mut file_path = env::temp_dir();
                    file_path.push("temp");
                    fs::File::create(&file_path)?;

                    process::Command::new(editor)
                        .arg(&file_path)
                        .status()?;

                    let contents = fs::read_to_string(&file_path)?;

                    // let tx = db_context.transaction()?;

                    Note::new(0, contents)
                } else {
                    Note::new(0, content.unwrap())
                }
            };

            insert_note(&pool, note).await?;
        },
        Some(Command::Get { all, id }) => {
            if all {
                let notes = get_all_notes(&pool).await?;

                println!("{}", Table::new(notes));
            } else {
                let note = get_note(&pool, id.unwrap()).await?;

                if let Some(note) = note {
                    println!("{}", Table::new(vec![note]));
                } else {
                    println!("No note found.");
                }
            }
        },
        Some(Command::Edit { id, content }) => {
            if let Some(original) = get_note(&pool, id).await? {
                let created = original.created;
                let new_note = Note { id, created, content };

                update_note(&pool, id, new_note).await?;
            }
        },
        Some(Command::Delete { id }) => {
            delete_note(&pool, id).await?;
        },
        None => {
            let notes = get_all_notes(&pool).await?;

            println!("{}", Table::new(notes));
        }
    }

    Ok(())
}
