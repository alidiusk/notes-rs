use std::fs;
use std::path::Path;

use anyhow::Error;
use chrono::{DateTime, Local};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::errors::NotesError;
use crate::table::{Row, Table};

pub type NoteTable = Table<NoteWithId>;

#[derive(Debug, Clone)]
pub struct NoteWithId {
    pub id: usize,
    pub created: DateTime<Local>,
    pub content: String,
}

impl NoteWithId {
    /// Takes a note and it's ID to make a NoteWithId.
    pub fn from_note(id: usize, note: Note) -> NoteWithId {
        NoteWithId {
            id,
            created: note.created,
            content: note.content,
        }
    }

    /// Returns a formatted string of the creation time.
    pub fn created_string(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

impl Row for NoteWithId {
    fn row(&self) -> Vec<ColoredString> {
        vec![
            self.id.to_string().bold(),
            self.created_string().bold(),
            self.content.clone().as_str().into(),
        ]
    }

    fn header() -> Vec<ColoredString> {
        vec![
            "ID".underline(),
            "Created".underline(),
            "Content".underline(),
        ]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub created: DateTime<Local>,
    pub content: String,
}

impl Note {
    /// Creates a new note and sets the creation time to this moment.
    pub fn new(content: String) -> Note {
        Note {
            created: Local::now(),
            content,
        }
    }

    /// Creates a new note with a given creation time.
    pub fn new_with_time(content: String, created: DateTime<Local>) -> Self {
        Note { created, content }
    }

    /// Returns a formatted string of the creation time.
    pub fn created_string(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notes(Vec<Note>);

impl Notes {
    /// Returns a new Notes given a Vec of Notes.
    pub fn new(notes: Vec<Note>) -> Self {
        Notes(notes)
    }

    /// Attempts to read a given file and serialize it into a Notes
    /// struct.
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let buf = fs::read_to_string(path)?;

        Ok(bincode::deserialize::<Notes>(buf.as_bytes())?)
    }

    /// Serializes the structure to bytes and writes it to the
    /// given file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let serialized = bincode::serialize(self)?;

        fs::write(path, serialized.as_slice())?;

        Ok(())
    }

    /// Returns the length of the underlying Vec.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of all of the notes, unless there are no notes,
    /// in which case it returns None.
    pub fn get_all(&self) -> Option<&[Note]> {
        if self.len() > 0 {
            Some(self.0.as_slice())
        } else {
            None
        }
    }

    /// Returns a Vec of note references and their corresponding
    /// index.
    pub fn get_all_with_id(&self) -> Option<Vec<NoteWithId>> {
        if self.len() > 0 {
            let mut notes = vec![];

            for (i, n) in self.0.iter().enumerate() {
                notes.push(NoteWithId::from_note(i, n.clone()));
            }

            Some(notes)
        } else {
            None
        }
    }

    /// Gets the note at the given index if it is within bounds; otherwise,
    /// it returns None.
    pub fn get(&self, index: usize) -> Option<&Note> {
        self.0.get(index)
    }

    /// Gets the note and its ID at the given index if it is within bounds;
    /// otherwise, it returns None.
    pub fn get_with_id(&self, index: usize) -> Option<NoteWithId> {
        self.0
            .get(index)
            .map(|n| NoteWithId::from_note(index, n.clone()))
    }

    /// Pushes a new note onto the Vec and returns the note ID.
    pub fn push(&mut self, note: Note) -> usize {
        self.0.push(note);

        self.len() - 1
    }

    /// Returns an error if index is out of range; otherwise, returns the
    /// deleted note.
    pub fn delete(&mut self, index: usize) -> anyhow::Result<Note> {
        if index >= self.len() {
            Err(Error::new(NotesError::InvalidNoteId(index)))
        } else {
            Ok(self.0.remove(index))
        }
    }

    /// Edits a note's content without changing the creation time. If the index is
    /// invalid, this returns an error.
    pub fn edit(&mut self, index: usize, content: String) -> anyhow::Result<&Note> {
        if let Some(note) = self.0.get_mut(index) {
            note.content = content;

            // This is safe, as we just confirmed this index exists.
            Ok(self.get(index).unwrap())
        } else {
            Err(Error::new(NotesError::InvalidNoteId(index)))
        }
    }
}
