use std::fs;
use std::path::{Path, PathBuf};

use dirs::data_dir;

use anyhow::Error;
use clap::{clap_app, value_t, App, ArgMatches};
use dialoguer::{Confirm, Editor};

use crate::build_table;
use crate::errors::NotesError;
use crate::notes::{Note, NoteBuilder, Notes};
use crate::tags::*;
use crate::util::*;

pub fn app() -> App<'static, 'static> {
    clap_app!(notes =>
        (version: "1.1")
        (author: "Liam Woodward")
        (about: "Application for storing short notes.")
        (@arg path: --path [notes] "path to the notes file.")
        (@subcommand new =>
         (about: "creates a new note.")
         (@group new =>
          (@arg content: "content of the note")
          (@arg file: -f --file [file] "file to create a new note from.")
          (@arg editor: -e --editor [editor] "create a new note in an editor")
         )
         (@arg tags: --tags +takes_value +multiple "tags to attach to the note.")
        )
        (@subcommand get =>
         (about: "gets one or more notes.")
         (@group get =>
          (@arg all: -a --all "get all notes.")
          (@arg id: "get the note with the given id.")
         )
         (@arg tags: -t --tags "get notes with the given tags.")
         (@arg desc: -d --desc "print note descriptions.")
        )
        (@subcommand edit =>
         (about: "edits a note")
         (@arg id: * "edit the note with the given id")
         (@arg content: -c --content +takes_value "change the note content.")
         (@arg tags: -t --tags +takes_value +multiple "change the note tags.")
         (@arg desc: -d --desc +takes_value "change the note description.")
        )
        (@subcommand delete =>
         (about: "deletes a note")
         (@arg id: * "delete the note with the given id.")
        )
    )
}

macro_rules! print_notes {
    ([$notes:expr], $err:expr) => {{
        print_notes!([$notes], $err, [(id, "b"), (created, "b"), (tags, ""), (content, "")])
    }};
    ([$notes:expr], $err:expr, $(($name:ident, $style:expr)),+) => {{
        print_notes!([$notes], $err, [(id, "b"), (created, "b"), (tags, ""), (content, ""), $(($name, $style)),+])
    }};
    ([$notes:expr], $err:expr, [$(($name:ident, $style:expr)),+]) => {{
        if let Some(notes) = $notes {
            let table = build_table!(
                vec![notes],
                [$(($name, $style)),+]
            );
            table.printstd();
        } else {
            println!("{}", $err);
        }
    }};
    ($notes:expr, $err:expr) => {{
        print_notes!($notes, $err, [(id, "b"), (created, "b"), (tags, ""), (content, "")])
    }};
    ($notes:expr, $err:expr, $(($name:ident, $style:expr)),+) => {{
        print_notes!($notes, $err, [(id, "b"), (created, "b"), (tags, ""), (content, ""), $(($name, $style)),+])
    }};
    ($notes:expr, $err:expr, [$(($name:ident, $style:expr)),+]) => {{
        if let Some(notes) = $notes {
            let table = build_table!(
                notes,
                [$(($name, $style)),+]
            );
            table.printstd();
        } else {
            println!("{}", $err);
        }
    }};
}

/// Runs the application.
pub fn run_app(app: App) -> anyhow::Result<()> {
    let matches = app.get_matches();
    let path = matches.value_of("path");
    let mut notes = get_notes_from_file(path)?;

    match matches.subcommand() {
        ("new", Some(new)) => run_new_note(&mut notes, new)?,
        ("get", Some(get)) => run_get_note(&notes, get)?,
        ("edit", Some(edit)) => run_edit_note(&mut notes, edit)?,
        ("delete", Some(delete)) => run_delete_note(&mut notes, delete)?,
        _ => print_notes!(notes.get_all_with_id(), "There are no notes."),
    }

    save_notes_to_file(&notes, path)?;

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

/// Creates a new note with valid user-supplied parameters.
fn run_new_note<'a>(notes: &mut Notes, args: &ArgMatches<'a>) -> anyhow::Result<()> {
    let mut note = {
        if let Some(path) = args.value_of("file") {
            new_note_from_file(path)?
        } else if args.is_present("editor") {
            new_note_from_editor(args.value_of("editor"))?
        } else {
            NoteBuilder::new()
                .with_content(&args.value_of("content").unwrap())
                .build()
        }
    };

    if let Some(tags) = args.values_of("tags") {
        note.add_tags(Tags::from(tags.collect::<Vec<&str>>()));
    }

    let id = notes.push(note);

    println!("Note with ID {} created.", id);

    Ok(())
}

/// Processes a user query for note(s) and prints it to stdout.
fn run_get_note<'a>(notes: &Notes, args: &ArgMatches<'a>) -> anyhow::Result<()> {
    let desc = args.is_present("desc");
    let mut all = args.is_present("all");
    let tags = args.values_of("tags");
    let id = args.value_of("id");

    // If `id` is not present, okay for it to be none, as we should never use it. It goes down
    // the `all` branch.
    if id.is_none() {
        all = true;
    }

    if all {
        if desc {
            print_notes!(notes.get_all_with_id(), "There are no notes.", (desc, ""));
        } else {
            print_notes!(notes.get_all_with_id(), "There are no notes.");
        }
    } else if let Some(tags) = tags {
        if desc {
            print_notes!(
                notes.get_all_with_tags(Tags::from(tags.collect::<Vec<&str>>())),
                "There are no notes.",
                (desc, "")
            );
        } else {
            print_notes!(
                notes.get_all_with_tags(Tags::from(tags.collect::<Vec<&str>>())),
                "There are no notes."
            );
        }
    } else if desc {
        print_notes!(
            [notes.get_with_id(id.unwrap().parse::<usize>()?)],
            "No note found.",
            (desc, "")
        );
    } else {
        print_notes!(
            [notes.get_with_id(id.unwrap().parse::<usize>()?)],
            "No note found."
        );
    }

    Ok(())
}

fn run_edit_note<'a>(notes: &mut Notes, args: &ArgMatches<'a>) -> anyhow::Result<()> {
    let id = value_t!(args, "id", usize).unwrap();
    let content = args.value_of("content").map(|s| s.to_string());
    let tags = args
        .values_of("tags")
        .map(|t| Tags::from(t.collect::<Vec<&str>>()));

    let new_note = notes.edit(id, content, tags)?;

    println!("Note {} edited: {}", id, new_note.content);

    Ok(())
}

fn run_delete_note<'a>(notes: &mut Notes, args: &ArgMatches<'a>) -> anyhow::Result<()> {
    // Make sure the note exists and get its content to print
    // the confirmation prompt.
    let id = value_t!(args, "id", usize).unwrap();

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

        Ok(NoteBuilder::new()
            .with_time(created)
            .with_content(&content)
            .build())
    }
}

/// Opens the user's defualt editor to create a note.
fn new_note_from_editor(editor: Option<&str>) -> anyhow::Result<Note> {
    // No edit message because it's annoying from a user viewpoint.
    let content = if let Some(editor) = editor {
        Editor::new().executable(editor).edit("")
    } else {
        Editor::new().edit("")
    };

    if let Ok(Some(content)) = content {
        Ok(NoteBuilder::new().with_content(&content).build())
    } else if content.is_err() {
        // NOTE: errors could be caused by other means.
        // Be aware of this.
        Err(Error::new(NotesError::NewNoteFromEditor(
            "Editor not found.".to_string(),
        )))
    // Ok(None)
    } else {
        Err(Error::new(NotesError::NewNoteFromEditor(
            "File not saved.".to_string(),
        )))
    }
}
