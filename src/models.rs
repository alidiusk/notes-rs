use std::path::Path;

use crate::db::*;

use chrono::{DateTime, Local};
use rusqlite::{params, Connection, Transaction, NO_PARAMS};

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

pub struct DbContext {
    pub table: TableName,
    conn: Connection,
}

impl DbContext {
    pub fn new<P: AsRef<Path>>(path: P, table: TableName) -> Result<Self, TableError> {
        Ok(DbContext {
            table,
            conn: Connection::open(path)?,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn transaction(&mut self) -> Result<Transaction, TableError> {
        Ok(self.conn.transaction()?)
    }
}

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

    pub fn init_db<'a>(conn: &Connection) -> Result<(), TableError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL UNIQUE,
                created TEXT NOT NULL,
                text TEXT NOT NULL
                )",
            NO_PARAMS,
        )?;

        Ok(())
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let date = self.created.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "[{}] {}\n\"{}\"", date, self.title, self.text)
    }
}

impl Table for Note {
    type Row = Note;

    fn get_all(conn: &Connection, table: &TableName) -> Result<Vec<Note>, TableError> {
        Ok(conn
            .prepare(&format!("SELECT * FROM {}", table))?
            .query_map(NO_PARAMS, |row| {
                Ok(Note {
                    title: row.get(1)?,
                    created: row.get(2)?,
                    text: row.get(3)?,
                })
            })?
            .filter_map(Result::ok)
            .collect())
    }

    fn insert(conn: &Connection, table: &TableName, row: Note) -> Result<(), TableError> {
        conn.execute(
            &format!(
                "INSERT INTO {} (title, created, text) VALUES (?1, ?2, ?3)",
                table
            ),
            params![row.title, row.created, row.text],
        )?;
        Ok(())
    }

    fn delete(conn: &Connection, query: Query) -> Result<u32, TableError> {
        let query_string = &query.to_sql();
        Ok(conn.execute(query_string, NO_PARAMS)? as u32)
    }

    fn update(conn: &Connection, query: Query) -> Result<u32, TableError> {
        let query_string = &query.to_sql();

        // let mut set_string = params
        //     .iter()
        //     .enumerate()
        //     .map(|(i, (name, _))| format!("{} = ?{}", name, i + 1))
        //     .collect::<Vec<String>>()
        //     .concat();
        // // Remove trailing comma
        // set_string.truncate(set_string.len() - 1);
        //
        // let stmt_params = params.iter().map(|(_, val)| val).collect::<Vec<&String>>();
        // let stmt_params: &[&String] = stmt_params.as_slice();
        //
        // let final_query = &(String::from("UPDATE notes\nSET ") + &set_string + query_string);
        Ok(conn.execute(query_string, NO_PARAMS)? as u32)
    }

    fn get(conn: &Connection, query: Query) -> Result<Vec<Note>, TableError> {
        let query_string = &query.to_sql();
        Ok(conn
            .prepare(query_string)?
            .query_map(NO_PARAMS, |row| {
                Ok(Note {
                    title: row.get(1)?,
                    created: row.get(2)?,
                    text: row.get(3)?,
                })
            })?
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
        let db_notes = Note::get_all(&tx).unwrap();

        tx.rollback();

        let note = Note::new("Day 12", "Today's diary...");
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
        Note::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
        let db_notes = Note::get_all(&tx).unwrap();

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
        Note::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
        let note_to_delete = Note::get_all(&tx).unwrap().get(0).unwrap().clone();
        Note::delete(&tx, note_to_delete).unwrap();
        let db_notes = Note::get_all(&tx).unwrap();

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
    //     Note::insert(&tx, Note::new("Day 13", "A new diary entry...")).unwrap();
    //     Note::insert(&tx, Note::new("Day 14", "A new diary entry...")).unwrap();
    //     Note::insert(&tx, Note::new("Day 15", "A new diary entry...")).unwrap();
    //     let query = Query::new().add_where(Where::Equal("title", Field::Str("Day 14")));
    //     let db_notes = Note::query(&tx, query).unwrap();
    //
    //     tx.rollback();
    //
    //     let db_note = db_notes.get(0).unwrap();
    //
    //     assert_eq!("Day 14", db_note.title);
    // }
}
