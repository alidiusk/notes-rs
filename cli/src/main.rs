// Get rid of dead code warnings for now
#![allow(dead_code)]

use lib::db::{DbContext, Field, Query, Table, Where};
use lib::models::{Note, NoteTable};

use chrono::{DateTime, Local};
use directories::ProjectDirs;
use exitfailure::ExitFailure;
use failure::ResultExt;
use structopt::StructOpt;

use std::path::{Path, PathBuf};
use std::{env, fs, process};

/*

notes new -f note.txt
notes new "Day 12" "Today in my diary..."

*/

#[derive(StructOpt, Debug)]
#[structopt(name = "notes")]
enum Opt {
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
        /// Deletes all notes
        #[structopt(short, long)]
        all: bool,
        /// Only deletes notes that exactly match the designated title
        #[structopt(short, long, requires = "title")]
        exact: bool,
        /// Deletes first note that matches the title, or all notes that match the title if `all` is specified.
        #[structopt(short, long, required_unless = "all")]
        title: Option<String>,
    },
}

fn get_notes_with_title(
    db_context: &DbContext,
    title: &str,
    exact: bool,
) -> Result<Vec<Note>, failure::Error> {
    if exact {
        let field = title.to_string();
        let query = Query::new_get(&db_context.table, vec![title.to_owned()])
            .add_where(Where::Equal("title".to_string(), Field::Str(field)));
        NoteTable::get(&db_context, query)
    } else {
        let field = title.to_string();
        let query = Query::new_get(&db_context.table, vec![title.to_owned()])
            .add_where(Where::Like("title".to_string(), Field::Str(field)));
        NoteTable::get(&db_context, query)
    }
}

fn delete_notes_with_title(
    db_context: &DbContext,
    title: &str,
    exact: bool,
) -> Result<u32, failure::Error> {
    if exact {
        let field = title.to_string();
        let query = Query::new_delete(&db_context.table)
            .add_where(Where::Equal("title".to_string(), Field::Str(field)));
        NoteTable::delete(&db_context, query)
    } else {
        let field = title.to_string();
        let query = Query::new_delete(&db_context.table)
            .add_where(Where::Like("title".to_string(), Field::Str(field)));
        NoteTable::delete(&db_context, query)
    }
}

fn file_is_dir<P: AsRef<Path>>(path: P) -> Result<bool, ExitFailure> {
    let metadata = fs::metadata(path)
        .with_context(|e| format!("could not determine if file is directory: {}", e))?;
    Ok(metadata.is_dir())
}

fn get_time_created<P: AsRef<Path>>(path: P) -> Result<DateTime<Local>, ExitFailure> {
    let metadata =
        fs::metadata(path).with_context(|e| format!("could not get file creation time: {}", e))?;
    let time = metadata.created();

    match time {
        Ok(time) => Ok(DateTime::<Local>::from(time)),
        Err(e) => Err(e.into()),
    }
}

fn get_file_contents<P: AsRef<Path>>(path: P) -> Result<String, ExitFailure> {
    let contents = fs::read_to_string(&path)
        .with_context(|e| format!("could not get file contents: {}", e))?;
    Ok(contents.trim().to_string())
}

fn main() -> Result<(), ExitFailure> {
    let opt = Opt::from_args();

    let project_dir = match ProjectDirs::from("", "", "Notes") {
        None => Err(failure::err_msg("Could not open local data directory."))
            .context("Could not get access to database.")?,
        Some(d) => d,
    };

    let dir = project_dir.data_local_dir();

    if !dir.exists() {
        fs::create_dir(dir).with_context(|_| {
            format!(
                "could not create local data directory: `{}`",
                dir.to_str().unwrap()
            )
        })?;
    }

    let path = dir.join("notes.db");
    let db_context = DbContext::new(path, "notes".to_string())?;

    NoteTable::init_db(&db_context)?;

    match opt {
        Opt::New {
            file: None,
            editor: None,
            title: Some(title),
            text: Some(text),
        } => {
            // let tx = db_context.transaction()?;
            NoteTable::insert(&db_context, Note::new(&title, &text))?;
            // tx.commit()?;
        }
        Opt::New {
            file: Some(file),
            editor: None,
            title: None,
            text: None,
        } => {
            let path = env::current_dir()
                .context("Could not find file")?
                .join(file);

            if file_is_dir(&path)? {
                panic!("File is a directory!");
            }

            // TODO: Remove expects later
            let title = String::from(
                path.file_stem()
                    .expect("Could not get file name.")
                    .to_str()
                    .expect("Could not get file name."),
            );
            let created = get_time_created(&path)?;
            let text = get_file_contents(&path)?;

            // let tx = db_context.transaction()?;
            let note = Note {
                title,
                created,
                text,
            };
            NoteTable::insert(&db_context, note)?;
            // tx.commit()?;
        }
        Opt::New {
            file: None,
            editor: Some(title),
            title: None,
            text: None,
        } => {
            // https://stackoverflow.com/questions/56011927/how-do-i-use-rust-to-open-the-users-default-editor-and-get-the-edited-content
            let editor = env::var("EDITOR")
                .with_context(|_| "could not get default editor from environment")?;
            let mut file_path = env::temp_dir();
            file_path.push("temp");
            fs::File::create(&file_path)
                .with_context(|e| format!("could not create temporary file to edit: {}", e))?;

            process::Command::new(editor)
                .arg(&file_path)
                .status()
                .with_context(|e| format!("error editing file: {}", e))?;

            let contents = fs::read_to_string(&file_path)
                .with_context(|e| format!("could not get file contents: {}", e))?;

            // let tx = db_context.transaction()?;

            let note = Note::new(&title, &contents);
            NoteTable::insert(&db_context, note)?;

            // tx.commit()?;
        }
        // Gets all notes with matching title
        Opt::Get {
            all: true,
            exact,
            title: Some(ref title),
        } => {
            let notes =
                // not sure to keep with_context call heree
                get_notes_with_title(&db_context, title, exact).with_context(|e| {
                    format!("could not get notes matching title `{}`: {}", title, e)
                })?;

            if notes.is_empty() {
                println!("No matches found.");
            }

            for note in notes {
                println!("{}", note);
            }
        }
        // Gets all notes
        Opt::Get {
            all: true,
            exact: false,
            title: None,
        } => {
            for row in NoteTable::get_all(&db_context)? {
                println!("{}", row);
            }
        }
        // Gets first note with matching title
        Opt::Get {
            all: false,
            exact,
            title: Some(ref title),
        } => {
            let notes =
                // not sure to keep with_context call here
                get_notes_with_title(&db_context, title, exact).with_context(|e| {
                    format!("could not get first note matching title `{}`: {}", title, e)
                })?;
            if let Some(note) = notes.get(0) {
                println!("{}", note);
            } else {
                return Err(failure::err_msg(format!(
                    "No notes matching title `{}` found.",
                    title
                )))
                .context("Could not get note.")?;
            }
        }
        Opt::Edit {
            note_title,
            title,
            text,
        } => {
            let params = match (title, text) {
                (Some(tit), Some(tex)) => {
                    vec![("title".to_string(), tit), ("text".to_string(), tex)]
                }
                (Some(tit), None) => vec![("title".to_string(), tit)],
                (None, Some(tex)) => vec![("text".to_string(), tex)],
                (None, None) => {
                    return Err(failure::err_msg("No title or text to change given"))
                        .context("Could not change text.")?;
                }
            };

            let query = Query::new_update(&db_context.table, params)
                .add_where(Where::Equal("title".to_string(), Field::Str(note_title)));

            // let tx = db_context.transaction()?;

            NoteTable::update(&db_context, query)?;

            // tx.commit()?;
        }
        Opt::Delete {
            all: true,
            exact,
            title: Some(title),
        } => {
            let num_deleted =
                // not sure to keep with_context call here
                delete_notes_with_title(&db_context, &title, exact).with_context(|e| {
                    format!("could not delete notes matching title `{}`: {}", title, e)
                })?;

            println!("Deleted {} notes", num_deleted);
        }
        // Gets all notes
        Opt::Delete {
            all: true,
            exact: false,
            title: None,
        } => {
            for row in NoteTable::get_all(&db_context)? {
                println!("{}", row);
            }
        }
        // Gets first note with matching title
        Opt::Delete {
            all: false,
            exact,
            title: Some(title),
        } => {
            // not sure to keep with_context call here
            let num_deleted =
                delete_notes_with_title(&db_context, &title, exact).with_context(|e| {
                    format!(
                        "could not delete first note matching title `{}`: {}",
                        title, e
                    )
                })?;

            println!("Deleted {} notes.", num_deleted);
        }
        _ => Err(failure::err_msg("Could not execute command"))
            .context("Invalid command entered.")?,
    }

    Ok(())
}
