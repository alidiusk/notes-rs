use serde::{Deserialize, Serialize};

use std::fmt;

/// A list of tags to be attached to a note.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tags(pub Vec<Tag>);

impl Tags {
    pub fn push(&mut self, tag: Tag) {
        self.0.push(tag);
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.0
    }

    pub fn has_tag(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<Vec<String>>::into(self).join(","))
    }
}

impl Into<Vec<String>> for &Tags {
    fn into(self) -> Vec<String> {
        let mut strings = vec![];

        for tag in self.tags() {
            strings.push(tag.into());
        }

        strings
    }
}

impl From<Vec<String>> for Tags {
    fn from(list: Vec<String>) -> Tags {
        let mut tags = Tags(vec![]);

        for string in list {
            tags.push(string.into());
        }

        tags
    }
}

/// A tag that describes a note.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tag {
    name: String,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Tag { name }
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.name
    }
}

impl Into<String> for &Tag {
    fn into(self) -> String {
        self.name.clone()
    }
}

impl From<String> for Tag {
    fn from(name: String) -> Tag {
        Tag { name }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
