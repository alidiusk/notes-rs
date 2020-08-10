use chrono::{DateTime, Local};

mod note;
mod notes;

pub use note::{Note, NoteBuilder, NoteWithId};
pub use notes::Notes;

fn format_time(time: &DateTime<Local>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
