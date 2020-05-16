#![allow(dead_code)]

mod models;
mod queries;

use crate::models::Note;
use crate::queries::*;

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
        editor: Option<String>,
        #[structopt(name = "title", required_unless_one = &["file", "editor"], conflicts_with_all = &["file", "editor"])]
        title: Option<String>,
        #[structopt(name = "text", required_unless_one = &["file", "editor"], conflicts_with_all = &["file", "editor"])]
        text: Option<String>,
    },
    /// Retrieve an existing note.
    Get {
        /// Returns all notes
        #[structopt(short, long)]
        all: bool,
        /// Only return notes that exactly match the designated title
        #[structopt(short, long, requires = "title")]
        exact: bool,
        /// Returns first note that matches the title, or all notes if `all` is specified.
        #[structopt(short, long, required_unless = "all")]
        title: Option<String>,
    },
    /// Edit an existing note.
    Edit {
        #[structopt(name = "note_title")]
        note_title: String,
        #[structopt(long, required_unless = "text")]
        title: Option<String>,
        #[structopt(long, required_unless = "title")]
        text: Option<String>,
    },
    /// Delete an existing note.
    Delete {
        /// Only deletes notes that exactly match the designated title
        #[structopt(short, long, requires = "title")]
        exact: bool,
        /// Deletes first note that matches the title, or all notes that match the title if `all` is specified.
        #[structopt(short, long)]
        title: String,
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
        Some(Command::New { file, editor, title, text }) => {
            let note = {
                if let Some(path) = file {
                    if file_is_dir(&path)? {
                        panic!("File is a directory!");
                    }

                    let title = String::from(
                        path.file_stem()
                            .expect("Could not get file name")
                            .to_str()
                            .expect("Could not get file name")
                    );
                    let created = get_time_created(&path)?;
                    let text = get_file_contents(&path)?;

                    Note { title, created, text }
                } else if let Some(path) = editor {
                    let editor = env::var("EDITOR")?;
                    let mut file_path = env::temp_dir();
                    file_path.push("temp");
                    fs::File::create(&file_path)?;

                    process::Command::new(editor)
                        .arg(&file_path)
                        .status()?;

                    let contents = fs::read_to_string(&file_path)?;

                    // let tx = db_context.transaction()?;

                    Note::new(path, contents)
                } else {
                    Note::new(title.unwrap(), text.unwrap())
                }
            };

            insert_note(&pool, note).await?;
        },
        Some(Command::Get { all, exact, title }) => {
            if all {
                for note in get_all_notes(&pool).await? {
                    println!("{}", note);
                }
            } else {
                let notes = get_notes(&pool, title.unwrap(), exact).await?;

                for note in notes {
                    println!("{}", note);
                }
            }
        },
        Some(Command::Edit { note_title, title, text }) => {
            if let Some(original) = get_notes(&pool, note_title.clone(), true).await?.get(0) {
                let title = title.unwrap_or(original.title.clone());
                let text = text.unwrap_or(original.text.clone());
                let created = original.created;
                let new_note = Note { title, created, text };

                update_notes(&pool, note_title, new_note).await?;
            }
        },
        Some(Command::Delete { exact, title }) => {
            delete_notes(&pool, title, exact).await?;
        },
        None => {
            let notes = get_all_notes(&pool).await?;

            for note in notes {
                println!("{}", note);
            }
        }
    }

    Ok(())
}
