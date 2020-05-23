use std::fs;
use std::path::{Path, PathBuf};

use dirs::data_dir;

use anyhow::Error;
use dialoguer::{Confirm, Editor};
use structopt::StructOpt;

use crate::errors::NotesError;
use crate::notes::{Note, NoteTable, Notes};
use crate::util::*;

#[derive(StructOpt, Debug)]
pub struct App {
    #[structopt(subcommand)]
    args: Option<Args>,
    /// Path to the notes file; defaults to the XDG Data Directory.
    #[structopt(long)]
    path: Option<PathBuf>,
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
        /// Create a new note from a file.
        #[structopt(short, long, conflicts_with = "editor")]
        file: Option<PathBuf>,
        /// Create a new note in an editor. Defaults to your default editor if
        /// no arguments are supplied.
        #[structopt(short, long, conflicts_with = "file")]
        editor: Option<Option<String>>,
        /// Create a new note from a string.
        #[structopt(name = "content", required_unless_one = &["file", "editor"], conflicts_with_all = &["file", "editor"])]
        content: Option<String>,
    },
    /// Retrieve an existing note.
    Get {
        /// Return all notes.
        #[structopt(short, long)]
        all: bool,
        /// Return the note with the matching id.
        #[structopt(name = "note_id", required_unless = "all")]
        id: Option<usize>,
    },
    /// Edit an existing note.
    Edit {
        /// Id of the note to edit
        #[structopt(name = "note_id")]
        id: usize,
        /// Content to replace the given note with
        #[structopt(name = "content")]
        content: String,
    },
    /// Delete an existing note.
    Delete {
        /// Id of the note to delete
        #[structopt(name = "note_id")]
        id: usize,
    },
}

/// Runs the application.
pub fn run_app(app: App) -> anyhow::Result<()> {
    let mut notes = get_notes_from_file(app.path.clone())?;

    if let Some(args) = app.args {
        handle_args(args, &mut notes)?;
    } else if let Some(notes) = notes.get_all_with_id() {
        let table = NoteTable::new(notes);
        println!("{}", table);
    } else {
        println!("There are no notes.");
    }

    save_notes_to_file(&notes, app.path)?;

    Ok(())
}

/// Returns the path to the notes directory in XDG Data Directory.
/// Creates it if it does not exist. Does not create the notes file
/// inside the directory if it does not exist.
fn get_xdg_data_dir() -> anyhow::Result<PathBuf> {
    let dir = data_dir().unwrap().join("Notes");

    if !dir.exists() {
        fs::create_dir(&dir)?;
    }

    let path = dir.join("notes");

    Ok(path)
}

/// Takes optional path; if supplied path is None, defaults to the
/// XDG Data Directory path.
fn get_notes_from_file<P: AsRef<Path>>(path: Option<P>) -> anyhow::Result<Notes> {
    if let Some(path) = path {
        init_notes_file(&path)?;
        Notes::from_file(path)
    } else {
        let path = get_xdg_data_dir()?;
        init_notes_file(&path)?;
        Notes::from_file(path)
    }
}

/// Initializes a new notes file if it does not exist.
fn init_notes_file<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    if !path.as_ref().exists() {
        let notes = Notes::new(vec![]);
        notes.to_file(&path)?;
    }

    Ok(())
}

/// Saves notes to file; defaults to XDG Data Directory path if path given
/// is None.
fn save_notes_to_file<P: AsRef<Path>>(notes: &Notes, path: Option<P>) -> anyhow::Result<()> {
    if let Some(path) = path {
        notes.to_file(path)
    } else {
        let path = get_xdg_data_dir()?;
        notes.to_file(path)
    }
}

/// Processes user-provided arguments.
fn handle_args(args: Args, notes: &mut Notes) -> anyhow::Result<()> {
    match args {
        Args::Command(Command::New {
            file,
            editor,
            content,
        }) => {
            run_new_note(notes, file, editor, content)?;
        }
        Args::Command(Command::Get { all, id }) => {
            run_get_note(notes, all, id)?;
        }
        Args::Command(Command::Edit { id, content }) => {
            let new_note = notes.edit(id, content)?;

            println!("Note {} edited: {}", id, new_note.content);
        }
        Args::Command(Command::Delete { id }) => {
            // Make sure the note exists and get its content to print
            // the confirmation prompt.
            let content = if let Some(note) = notes.get(id) {
                note.content.clone()
            } else {
                println!("No note found.");
                return Ok(());
            };

            let confirm = format!("Are you sure that you want to delete `{}: {}`", id, content);

            if Confirm::new().with_prompt(confirm).interact()? {
                notes.delete(id)?;
                println!("Note `{}: {}` deleted.", id, content);
            }
        }
        Args::Content(content) => {
            let id = notes.push(Note::new(content.join(" ")));

            println!("Note with ID {} created.", id);
        }
    }

    Ok(())
}

/// Processes a user query for note(s) and prints it to stdout.
fn run_get_note(notes: &Notes, all: bool, id: Option<usize>) -> anyhow::Result<()> {
    if all {
        if let Some(notes) = notes.get_all_with_id() {
            let table = NoteTable::new(notes);
            println!("{}", table);
        } else {
            println!("There are no notes.");
        }
    } else {
        let note = notes.get_with_id(id.unwrap());

        if let Some(note) = note {
            let table = NoteTable::new(vec![note]);
            println!("{}", table);
        } else {
            println!("No note found.");
        }
    }

    Ok(())
}

/// Creates a new note with valid user-supplied parameters.
fn run_new_note<P: AsRef<Path>>(
    notes: &mut Notes,
    file: Option<P>,
    editor: Option<Option<String>>,
    content: Option<String>,
) -> anyhow::Result<()> {
    let note = {
        if let Some(path) = file {
            new_note_from_file(path)?
        } else if let Some(editor) = editor {
            new_note_from_editor(editor)?
        } else {
            Note::new(content.unwrap())
        }
    };

    let id = notes.push(note);

    println!("Note with ID {} created.", id);

    Ok(())
}

/// Creates a new note from a file.
fn new_note_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Note> {
    if file_is_dir(&path)? {
        Err(Error::new(NotesError::NewNoteFileIsDir(
            // NOTE: could fail if path is not all utf-8
            path.as_ref().to_str().unwrap().to_string(),
        )))
    } else {
        let created = get_time_created(&path)?;
        let content = get_file_contents(&path)?;

        Ok(Note::new_with_time(content, created))
    }
}

/// Opens the user's defualt editor to create a note.
fn new_note_from_editor(editor: Option<String>) -> anyhow::Result<Note> {
    // No edit message because it's annoying from a user viewpoint.
    let content = if let Some(editor) = editor {
        Editor::new().executable(editor).edit("")
    } else {
        Editor::new().edit("")
    };

    if let Ok(Some(content)) = content {
        Ok(Note::new(content))
    } else if let Err(io_error) = content {
        Err(Error::new(NotesError::NewNoteFromEditor(format!(
            "{}",
            io_error
        ))))
    } else {
        Err(Error::new(NotesError::NewNoteFromEditor(
            "File not saved.".to_string(),
        )))
    }
}
