use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::config::Config;

use failure::ResultExt;
use rusqlite::{Connection, Transaction};

/*

 - Tables
    - Each table struct will contain the fields
    - Associated functions to insert into table
    - Actual struct instances will be of individual rows

 - Considerations
    - How to implement queries?

*/

pub type TableName = String;
pub type ColumnName = String;
pub type ColumnValue = String;

pub type Params = Vec<(String, String)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    Str(ColumnValue),
    Int(i32),
}

impl Field {
    pub fn to_sql(&self) -> String {
        match *self {
            Field::Str(ref s) => format!("'{}'", s),
            Field::Int(ref n) => n.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Where {
    Equal(ColumnName, Field),
    In(ColumnName, Field),
    Like(ColumnName, Field),
    Between(ColumnName, Field, Field),
}

#[derive(Debug, PartialEq)]
pub enum Order {
    Ascending(ColumnName),
    Descending(ColumnName),
}

#[derive(Debug, PartialEq)]
pub struct Limit(u32, u32);

// #[derive(Debug, PartialEq)]
// pub struct Query<'a> {
//     where_clause: Option<Vec<Where<'a>>>,
//     order_clause: Option<Order<'a>>,
//     limit_clause: Option<u32>,
// }

#[derive(Debug, PartialEq)]
pub enum Query {
    Get {
        table: TableName,
        columns: Vec<ColumnName>,
        where_clause: Option<Vec<Where>>,
        order_clause: Option<Order>,
        limit_clause: Option<Limit>,
    },
    Update {
        table: TableName,
        set_clause: Params,
        where_clause: Option<Vec<Where>>,
        order_clause: Option<Order>,
        limit_clause: Option<Limit>,
    },
    Delete {
        table: TableName,
        where_clause: Option<Vec<Where>>,
        order_clause: Option<Order>,
        limit_clause: Option<Limit>,
    },
}

impl Query {
    pub fn new_get(table: &TableName, columns: Vec<ColumnName>) -> Self {
        Query::Get {
            table: table.to_owned(),
            columns,
            where_clause: None,
            order_clause: None,
            limit_clause: None,
        }
    }

    pub fn new_update(table: &TableName, set_clause: Params) -> Self {
        Query::Update {
            table: table.to_owned(),
            set_clause,
            where_clause: None,
            order_clause: None,
            limit_clause: None,
        }
    }

    pub fn new_delete(table: &TableName) -> Self {
        Query::Delete {
            table: table.to_owned(),
            where_clause: None,
            order_clause: None,
            limit_clause: None,
        }
    }

    pub fn add_where(mut self, new_where: Where) -> Self {
        match self {
            Self::Get {
                ref mut where_clause,
                ..
            } => {
                if let Some(wheres) = where_clause {
                    wheres.push(new_where);
                } else {
                    *where_clause = Some(vec![new_where]);
                }
            }
            Self::Update {
                ref mut where_clause,
                ..
            } => {
                if let Some(wheres) = where_clause {
                    wheres.push(new_where);
                } else {
                    *where_clause = Some(vec![new_where]);
                }
            }
            Self::Delete {
                ref mut where_clause,
                ..
            } => {
                if let Some(wheres) = where_clause {
                    wheres.push(new_where);
                } else {
                    *where_clause = Some(vec![new_where]);
                }
            }
        }

        self
    }

    pub fn add_order(mut self, order: Order) -> Self {
        match self {
            Self::Get {
                ref mut order_clause,
                ..
            } => {
                *order_clause = Some(order);
            }
            Self::Update {
                ref mut order_clause,
                ..
            } => {
                *order_clause = Some(order);
            }
            Self::Delete {
                ref mut order_clause,
                ..
            } => {
                *order_clause = Some(order);
            }
        }

        self
    }

    pub fn add_limit(mut self, limit: Limit) -> Self {
        match self {
            Self::Get {
                ref mut limit_clause,
                ..
            } => {
                *limit_clause = Some(limit);
            }
            Self::Update {
                ref mut limit_clause,
                ..
            } => {
                *limit_clause = Some(limit);
            }
            Self::Delete {
                ref mut limit_clause,
                ..
            } => {
                *limit_clause = Some(limit);
            }
        }

        self
    }

    pub fn to_sql(&self) -> String {
        let (where_clause, order_clause, limit_clause) = match self {
            Self::Get {
                ref where_clause,
                ref order_clause,
                ref limit_clause,
                ..
            } => (where_clause, order_clause, limit_clause),
            Self::Update {
                ref where_clause,
                ref order_clause,
                ref limit_clause,
                ..
            } => (where_clause, order_clause, limit_clause),
            Self::Delete {
                ref where_clause,
                ref order_clause,
                ref limit_clause,
                ..
            } => (where_clause, order_clause, limit_clause),
        };

        let where_clause = match where_clause {
            None => String::from(""),
            Some(wheres) => {
                String::from("WHERE ")
                    + &wheres
                        .iter()
                        .map(|clause| match clause {
                            Where::Equal(ref s1, ref s2) => format!("{} = {}", s1, s2.to_sql()),
                            Where::In(ref s1, ref s2) => format!("{} IN {}", s1, s2.to_sql()),
                            Where::Like(ref s1, ref s2) => format!("{} LIKE {}", s1, s2.to_sql()),
                            Where::Between(ref s1, ref s2, ref s3) => {
                                format!("{} BETWEEN {} AND {}", s1, s2.to_sql(), s3.to_sql())
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(" AND ")
            }
        };

        let order_clause = match order_clause {
            None => String::from(""),
            Some(Order::Ascending(ref s)) => format!("ORDER BY {} ASC", s),
            Some(Order::Descending(ref s)) => format!("ORDER BY {} DESC", s),
        };

        let limit_clause = match limit_clause {
            None => String::from(""),
            Some(Limit(limit, offset)) => format!("LIMIT {} OFFSET {}", limit, offset),
        };

        match self {
            Self::Get { ref table, .. } => {
                let select = format!("SELECT * FROM {}", table);

                [select, where_clause, order_clause, limit_clause].join("\n")
            }
            Self::Update {
                ref table,
                ref set_clause,
                ..
            } => {
                let update = format!("UPDATE {}", table);

                let mut set = set_clause
                    .iter()
                    .map(|(name, val)| format!("{} = {},", name, val))
                    .collect::<Vec<String>>()
                    .concat();
                // Remove trailing comma
                set.truncate(set.len() - 1);

                [update, set, where_clause, order_clause, limit_clause].join("\n")
            }
            Self::Delete { ref table, .. } => {
                let delete = format!("DELETE FROM {}", table);

                [delete, where_clause, order_clause, limit_clause].join("\n")
            }
        }
    }
}

pub struct DbContext {
    pub table: TableName,
    pub conn: Connection,
}

impl DbContext {
    pub fn new<P: AsRef<Path>>(path: P, table: TableName, config: Config) -> Result<Self, failure::Error> {
        Ok(DbContext {
            table,
            conn: Connection::open(path)
                .with_context(|e| format!("Error establishing connection to the database: {}", e))?,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn transaction(&mut self) -> Result<Transaction, failure::Error> {
        Ok(self
            .conn
            .transaction()
            .with_context(|e| format!("could not create database transaction: {}", e))?)
    }
}

pub trait Table {
    type Row;

    fn get_all<'a>(db_context: &DbContext) -> Result<Vec<Self::Row>, failure::Error>;
    fn insert<'a>(
        db_context: &DbContext,
        row: Self::Row,
    ) -> Result<(), failure::Error>;
    fn delete<'a>(db_context: &DbContext, query: Query) -> Result<u32, failure::Error>;
    fn update<'a>(db_context: &DbContext, query: Query) -> Result<u32, failure::Error>;
    fn get<'a>(db_context: &DbContext, query: Query) -> Result<Vec<Self::Row>, failure::Error>;
}

#[derive(Debug)]
pub enum TableError {
    GetAllError(String),
    InsertionError(String),
    DeletionError(String),
    QueryError(Query),
    TxError,
    TxCommitError,
    SqliteError(rusqlite::Error),
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TableError::GetAllError(ref table) => {
                write!(f, "Error getting all rows from table: {}", table)
            }
            TableError::InsertionError(ref table) => {
                write!(f, "Error inserting into table: {}", table)
            }
            TableError::DeletionError(ref row) => write!(f, "Error deleting row: {}", row),
            TableError::QueryError(ref query) => write!(f, "Error executing query: {:?}", query),
            TableError::TxError => write!(f, "No transaction initialized"),
            TableError::TxCommitError => write!(f, "No transaction to commit"),
            TableError::SqliteError(ref e) => write!(f, "Sqlite Error: {}", e),
        }
    }
}

impl Error for TableError {
    fn description(&self) -> &str {
        match *self {
            // Refactor -- format! leads to borrow issues, but make it work anyway.
            TableError::GetAllError(_) => "Error getting all rows from table",
            TableError::InsertionError(_) => "Error inserting into table",
            TableError::DeletionError(_) => "Error deleting row",
            TableError::QueryError(_) => "Error executing query",
            TableError::TxError => "No transaction initialized",
            TableError::TxCommitError => "No transaction to commit",
            TableError::SqliteError(_) => "Sqlite Error",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            TableError::SqliteError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for TableError {
    fn from(error: rusqlite::Error) -> Self {
        TableError::SqliteError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_to_sql() {
        assert_eq!(Field::Int(64).to_sql(), "64");
        assert_eq!(Field::Str("title".to_string()).to_sql(), "'title'");
    }

    //     #[test]
    //     fn query_builder() {
    //         let query_built = Query::new()
    //             .add_where(Where::Equal("title".to_string(), Field::Str("Day 12")))
    //             .add_order(Order::Ascending("noteid".to_string()))
    //             .add_limit(10);
    //
    //         let query = Query {
    //             where_clause: Some(vec![Where::Equal("title".to_string(), Field::Str("Day 12"))]),
    //             order_clause: Some(Order::Ascending("noteid".to_string())),
    //             limit_clause: Some(10),
    //         };
    //
    //         assert_eq!(query_built, query);
    //     }
    //
    //     #[test]
    //     fn query_to_sql() {
    //         let query = Query::new()
    //             .add_where(Where::Equal("title".to_string(), Field::Str("Day 12".to_string())))
    //             .add_where(Where::Like("text".to_string(), Field::Str("Diary%".to_string())))
    //             .add_order(Order::Ascending("noteid".to_string()))
    //             .add_limit(10);
    //
    //         let query_stmt =
    //             "WHERE title = 'Day 12' AND text LIKE 'Diary%'\nORDER BY noteid ASC\nLIMIT 10";
    //
    //         assert_eq!(query.to_sql(), query_stmt);
    //     }
}
