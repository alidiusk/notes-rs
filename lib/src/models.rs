use crate::db::*;

use chrono::{DateTime, Local};
use failure::ResultExt;
use rusqlite::{params, NO_PARAMS};

/*

let query = Query::new()
    .add_where(Equal("title", "Day 12"))
    .add_order(Ascending("noteid"))
    .add_limit(10)

for row in Note::query(conn, query) {
    println("{}", row.unwrap())
}

*/

// NOTE: REMEMBER to row.get(n) for the correct n; n=0 is the primary key row id

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
    pub title: String,
    pub created: DateTime<Local>,
    pub text: String,
}

impl Note {
    pub fn new(title: &str, text: &str) -> Note {
        Note {
            title: title.to_string(),
            created: Local::now(),
            text: text.trim().to_string(),
        }
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let date = self.created.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "[{}] {}\n\"{}\"", date, self.title, self.text)
    }
}

pub struct NoteTable;

impl NoteTable {
    pub fn init_db(db_context: &DbContext) -> Result<(), failure::Error> {
        db_context.conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL UNIQUE,
                created TEXT NOT NULL,
                text TEXT NOT NULL
                )",
            NO_PARAMS,
        )
        .with_context(|e| format!("could not init database: {}", e))?;

        Ok(())
    }
}

impl Table for NoteTable {
    type Row = Note;

    fn get_all(db_context: &DbContext) -> Result<Vec<Note>, failure::Error> {
        Ok(db_context.conn
            .prepare(&format!("SELECT * FROM {}", &db_context.table))?
            .query_map(NO_PARAMS, |row| {
                Ok(Note {
                    // start at 1 because index 0 is the noteid
                    // cannot add context to this apparently because of
                    // rusqlite::error::Error implementation
                    title: row.get(1)?,
                    created: row.get(2)?,
                    text: row.get(3)?,
                })
            })
            .context("could note retrieve note")?
            .filter_map(Result::ok)
            .collect())
    }

    fn insert(db_context: &DbContext, row: Note) -> Result<(), failure::Error> {
        db_context.conn.execute(
            &format!(
                "INSERT INTO {} (title, created, text) VALUES (?1, ?2, ?3)",
                &db_context.table
            ),
            params![row.title, row.created, row.text],
        )
        .context("could not insert note")?;
        Ok(())
    }

    fn delete(db_context: &DbContext, query: Query) -> Result<u32, failure::Error> {
        // Verify this is Query::Delete
        let query_string = &query.to_sql();
        Ok(db_context.conn
            .execute(query_string, NO_PARAMS)
            .context("could not delete note")? as u32)
    }

    fn update(db_context: &DbContext, query: Query) -> Result<u32, failure::Error> {
        let query_string = &query.to_sql();

        Ok(db_context.conn
            .execute(query_string, NO_PARAMS)
            .context("could not update note")? as u32)
    }

    fn get(db_context: &DbContext, query: Query) -> Result<Vec<Note>, failure::Error> {
        let query_string = &query.to_sql();
        Ok(db_context.conn
            .prepare(query_string)?
            .query_map(NO_PARAMS, |row| {
                Ok(Note {
                    // start at 1 because index 0 is the noteid
                    // cannot add context to this apparently because of
                    // rusqlite::error::Error implementation
                    title: row.get(1)?,
                    created: row.get(2)?,
                    text: row.get(3)?,
                })
            })
            .context("could not retrieve note")?
            .filter_map(Result::ok)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mock_db(conn: &Connection) {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL UNIQUE,
                created TEXT NOT NULL,
                text TEXT NOT NULL
                )",
            NO_PARAMS,
        )
        .unwrap();

        conn.execute(
            "INSERT INTO notes (title, created, text) VALUES (?1, ?2, ?3)",
            params![
                String::from("Day 12"),
                Local::now(),
                String::from("Today's diary...")
            ],
        )
        .unwrap();
    }

    #[test]
    fn get_all() {
        let mut conn = Connection::open("test.db").unwrap();
        let tx = conn.transaction().unwrap();

        make_mock_db(&tx);
        let db_notes = NoteTable::get_all(&tx).unwrap();

        tx.rollback();

        let note = NoteTable::new("Day 12", "Today's diary...");
        let db_note = db_notes.get(0).unwrap();

        // Time will be different
        assert_eq!(note.title, db_note.title);
        assert_eq!(note.text, db_note.text);
    }

    #[test]
    fn insert() {
        let mut conn = Connection::open("test.db").unwrap();
        let tx = conn.transaction().unwrap();

        make_mock_db(&tx);
        NoteTable::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
        let db_notes = NoteTable::get_all(&tx).unwrap();

        tx.rollback();

        let db_note = db_notes.get(1).unwrap();

        assert_eq!(2, db_notes.len());
        assert_eq!("Day 13", db_note.title);
    }

    #[test]
    fn delete() {
        let mut conn = Connection::open("test.db").unwrap();
        let tx = conn.transaction().unwrap();

        make_mock_db(&tx);
        NoteTable::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
        let note_to_delete = NoteTable::get_all(&tx).unwrap().get(0).unwrap().clone();
        NoteTable::delete(&tx, note_to_delete).unwrap();
        let db_notes = NoteTable::get_all(&tx).unwrap();

        tx.rollback();

        let db_note = db_notes.get(0).unwrap();

        assert_eq!(1, db_notes.len());
        assert_eq!("Day 13", db_note.title);
    }

    // #[test]
    // fn query() {
    //     let mut conn = Connection::open("test.db").unwrap();
    //     let tx = conn.transaction().unwrap();
    //
    //     make_mock_db(&tx);
    //     NoteTable::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
    //     NoteTable::insert(&tx, Note::new("Day 14", "A new diary entry...")).unwrap();
    //     NoteTable::insert(&tx, Note::new("Day 15", "A new diary entry...")).unwrap();
    //     let query = Query::new().add_where(Where::Equal("title", Field::Str("Day 14")));
    //     let db_notes = NoteTable::query(&tx, query).unwrap();
    //
    //     tx.rollback();
    //
    //     let db_note = db_notes.get(0).unwrap();
    //
    //     assert_eq!("Day 14", db_note.title);
    // }
}