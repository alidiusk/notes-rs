use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotesError {
    #[error("Note ID `{0}` is invalid.")]
    InvalidNoteId(usize),
    #[error("Cannot make new note from file; `{0}` is a directory.")]
    NewNoteFileIsDir(String),
    #[error("Unable to make note from editor: {0}")]
    NewNoteFromEditor(String),
    #[error("Unable to read notes file.")]
    NoteDeserialization,
}
