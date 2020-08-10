use std::fs;
use std::path::Path;

use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::errors::NotesError;
use crate::tags::*;

use super::note::{Note, NoteWithId};

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

        bincode::deserialize::<Notes>(buf.as_bytes())
            .map_err(|_| Error::new(NotesError::NoteDeserialization))
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
        if self.0.is_empty() {
            return None;
        }

        let mut notes = vec![];

        for (i, note) in self.0.iter().enumerate() {
            notes.push(NoteWithId(i, note));
        }

        Some(notes)
    }

    /// Gets the note at the given index if it is within bounds; otherwise,
    /// it returns None.
    pub fn get(&self, index: usize) -> Option<&Note> {
        self.0.get(index)
    }

    /// Gets the note and its ID at the given index if it is within bounds;
    /// otherwise, it returns None.
    pub fn get_with_id(&self, index: usize) -> Option<NoteWithId> {
        self.0.get(index).map(|n| NoteWithId(index, n))
    }

    /// Gets all notes that have a given tag.
    pub fn get_all_with_tag(&self, tag: Tag) -> Option<Vec<NoteWithId>> {
        self.get_all_with_tags(Tags(vec![tag]))
    }

    /// Gets all notes that possess all of the listed tags.
    pub fn get_all_with_tags(&self, tags: Tags) -> Option<Vec<NoteWithId>> {
        if self.0.is_empty() {
            return None;
        }

        let mut notes = vec![];
        for (i, note) in self.0.iter().enumerate() {
            if note.has_tags(&tags) {
                notes.push(NoteWithId(i, note));
            }
        }

        if notes.is_empty() {
            return None;
        }

        Some(notes)
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

    /// Edits a note's content or tags without changing the creation time. If the index is
    /// invalid, this returns an error.
    pub fn edit(
        &mut self,
        index: usize,
        content: Option<String>,
        tags: Option<Tags>,
    ) -> anyhow::Result<&Note> {
        if let Some(note) = self.0.get_mut(index) {
            if let Some(content) = content {
                note.content = content;
            }

            if let Some(tags) = tags {
                note.tags = tags;
            }

            // This is safe, as we just confirmed this index exists.
            Ok(self.get(index).unwrap())
        } else {
            Err(Error::new(NotesError::InvalidNoteId(index)))
        }
    }
}
