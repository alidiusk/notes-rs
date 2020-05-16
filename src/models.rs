use chrono::{DateTime, Local};

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
        let date = self.created.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "[{}] {}\n\"{}\"", date, self.title, self.text)
    }
}

// #[derive(Debug, PartialEq, Clone)]
// pub struct SqliteNote {
//     pub title: String,
//     pub created: i32,
//     pub text: String,
// }
//
// impl From<SqliteNote> for Note {
//     fn from(note: SqliteNote) -> Self {
//         let SqliteNote {
//             title,
//             created,
//             text,
//         } = note;
//         Note {
//             title,
//             created: Local.timestamp(created as _, 0),
//             text,
//         }
//     }
// }
//
// // https://github.com/launchbadge/sqlx/blob/master/examples/realworld/src/db/model.rs
//
// pub type NotesResult<T> = anyhow::Result<T, NotesError>;
//
// #[derive(Debug, thiserror::Error)]
// pub enum NotesError {
//     /// The requested entity does not exist
//     #[error("Entity does not exist")]
//     NotFound,
//     /// The operation violates a uniqueness constraint
//     #[error("{0}")]
//     UniqueViolation(String),
//     /// The requested operation violates the data model
//     #[error("{0}")]
//     ModelViolation(String),
//     #[error(transparent)]
//     /// A generic unhandled error
//     Provider(sqlx::Error),
// }
//
// impl From<sqlx::Error> for NotesError {
//     /// Convert a SQLx error into a provider error
//     ///
//     /// For Database errors we attempt to downcast
//     fn from(e: sqlx::Error) -> Self {
//         match e {
//             sqlx::Error::RowNotFound => NotesError::NotFound,
//             sqlx::Error::Database(db_err) => {
//                 {
//                     if let Some(sqlite_err) = db_err.try_downcast_ref::<sqlx::sqlite::SqliteError>()
//                     {
//                         if let Ok(provide_err) = NotesError::try_from(sqlite_err) {
//                             return provide_err;
//                         }
//                     }
//                 }
//
//                 NotesError::Provider(sqlx::Error::Database(db_err))
//             }
//             _ => NotesError::Provider(e),
//         }
//     }
// }
