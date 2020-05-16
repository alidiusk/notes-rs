use chrono::{DateTime, Local};
use colored::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub title: String,
    pub created: DateTime<Local>,
    pub text: String,
}

impl Note {
    pub fn new(title: String, text: String) -> Note {
        Note {
            title,
            created: Local::now(),
            text,
        }
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let date = "[".to_string() + &self.created.format("%Y-%m-%d %H:%M:%S").to_string() + "]";
        write!(
            f,
            "{} {}: \"{}\"",
            date.bold(),
            self.title.bold(),
            self.text
        )
    }
}
