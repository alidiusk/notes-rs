use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotesError {
    #[error("Note ID `{0}` is invalid.")]
    InvalidNoteId(usize),
}
