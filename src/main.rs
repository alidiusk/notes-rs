#![allow(dead_code)]

mod errors;
mod notes;
mod print;
mod util;

use crate::notes::{Note, Notes};
use crate::print::Table;
use crate::util::*;

use directories::ProjectDirs;
use structopt::StructOpt;

use std::path::PathBuf;
use std::{env, fs, process};

#[derive(StructOpt, Debug)]
struct App {
    #[structopt(subcommand)]
    args: Option<Args>,
}

#[derive(StructOpt, Debug)]
enum Args {
    #[structopt(flatten)]
    Command(Command),
    #[structopt(external_subcommand)]
    Content(Vec<String>),
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Create a new note.
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
        id: Option<usize>,
    },
    /// Edit an existing note.
    Edit {
        #[structopt(name = "note_id")]
        id: usize,
        #[structopt(name = "content")]
        content: String,
    },
    /// Delete an existing note.
    Delete {
        /// Deletes the note with the corresponding id.
        #[structopt(name = "note_id")]
        id: usize,
    },
}

fn main() -> anyhow::Result<()> {
    let app = App::from_args();

    let project_dir = ProjectDirs::from("", "", "Notes").unwrap();

    let dir = project_dir.data_local_dir();

    if !dir.exists() {
        fs::create_dir(dir)?;
    }

    let path = &dir.join(dir).join("notes");
    let path_str = &path.to_str().unwrap().to_string();

    if !path.exists() {
        let notes = Notes::new(vec![]);
        notes.to_file(path_str)?;
    }

    let mut notes = Notes::from_file(path_str)?;

    if let Some(args) = app.args {
        handle_args(args, &mut notes)?;
    } else if let Some(notes) = notes.get_all_with_id() {
        println!("{}", Table::new(notes));
    } else {
        println!("There are no notes.");
    }

    notes.to_file(path_str)?;

    Ok(())
}

fn handle_args(args: Args, notes: &mut Notes) -> anyhow::Result<()> {
    match args {
        Args::Command(Command::New {
            file,
            editor,
            content,
        }) => {
            let note = {
                if let Some(path) = file {
                    if file_is_dir(&path)? {
                        panic!("File is a directory!");
                    }

                    // fakee id
                    let created = get_time_created(&path)?;
                    let content = get_file_contents(&path)?;

                    Note::new_with_time(content, created)
                } else if editor {
                    let editor = env::var("EDITOR")?;
                    let mut file_path = env::temp_dir();
                    file_path.push("temp");
                    fs::File::create(&file_path)?;

                    process::Command::new(editor).arg(&file_path).status()?;

                    let contents = fs::read_to_string(&file_path)?;

                    // let tx = db_context.transaction()?;

                    Note::new(contents)
                } else {
                    Note::new(content.unwrap())
                }
            };

            let id = notes.push(note);

            println!("Note with ID {} created.", id);
        }
        Args::Command(Command::Get { all, id }) => {
            if all {
                if let Some(notes) = notes.get_all_with_id() {
                    println!("{}", Table::new(notes));
                } else {
                    println!("There are no notes.");
                }
            } else {
                let note = notes.get(id.unwrap());

                if let Some(note) = note {
                    println!("{}", Table::new(vec![(id.unwrap(), note)]));
                } else {
                    println!("No note found.");
                }
            }
        }
        Args::Command(Command::Edit { id, content }) => {
            let new_note = notes.edit(id, content)?;

            println!("Note {} edited: {}", id, new_note.content);
        }
        Args::Command(Command::Delete { id }) => {
            let note = notes.delete(id)?;

            println!("Note `{}: {}` deleted.", id, note.content);
        }
        Args::Content(content) => {
            let id = notes.push(Note::new(content.join(" ")));

            println!("Note with ID {} created.", id);
        }
    }

    Ok(())
}
