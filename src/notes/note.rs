use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use super::format_time;
use crate::tags::*;

pub struct NoteWithId<'a>(pub usize, pub &'a Note);

impl<'a> NoteWithId<'a> {
    pub fn id(&self) -> usize {
        self.0
    }

    pub fn created(&self) -> &String {
        &self.1.created
    }

    pub fn tags(&self) -> &Tags {
        &self.1.tags
    }

    pub fn content(&self) -> &String {
        &self.1.content
    }

    pub fn desc(&self) -> &String {
        &self.1.desc
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub created: String,
    pub tags: Tags,
    pub content: String,
    pub desc: String,
}

impl Note {
    pub fn new(created: String, tags: Tags, content: String, desc: String) -> Self {
        Note {
            created,
            tags,
            content,
            desc,
        }
    }

    /// Add a tag list to the note.
    pub fn add_tags(&mut self, tags: Tags) {
        self.tags = tags;
    }

    /// Check if this note has a given tag.
    pub fn has_tag(&self, tag: &Tag) -> bool {
        self.tags.has_tag(tag)
    }

    /// Check if this note has all of the  given tags.
    pub fn has_tags(&self, other: &Tags) -> bool {
        self.tags.contains(other)
    }
}

pub struct NoteBuilder {
    pub created: Option<DateTime<Local>>,
    pub tags: Option<Tags>,
    pub content: Option<String>,
    pub desc: Option<String>,
}

impl NoteBuilder {
    pub fn new() -> Self {
        NoteBuilder {
            created: None,
            tags: None,
            content: None,
            desc: None,
        }
    }

    pub fn with_time(mut self, created: DateTime<Local>) -> Self {
        self.created = Some(created);
        self
    }

    pub fn with_tags(mut self, tags: Tags) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn with_desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    pub fn build(self) -> Note {
        let created = format_time(&self.created.unwrap_or_else(|| Local::now()));
        let tags = self.tags.unwrap_or_else(|| Tags(vec![]));
        let content = self.content.unwrap_or_else(|| "".to_string());
        let desc = self.desc.unwrap_or_else(|| "".to_string());

        Note::new(created, tags, content, desc)
    }
}
