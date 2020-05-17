use std::fmt;

use chrono::{DateTime, Local};
use colored::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub id: i32,
    pub created: DateTime<Local>,
    pub content: String,
}

impl Note {
    pub fn new(id: i32, content: String) -> Note {
        Note {
            id,
            created: Local::now(),
            content,
        }
    }

    pub fn created_string(&self) -> String {
        self.created.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn id_string(&self) -> String {
        "[".to_string() + &self.id.to_string() + "]"
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let date = "[".to_string() + &self.created_string() + "]";
        let id = "[".to_string() + &self.id.to_string() + "]";
        write!(f, "{} {} \"{}\"", id.bold(), date.bold(), self.content)
    }
}
