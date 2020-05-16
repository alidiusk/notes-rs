use chrono::{Local, DateTime};
use sqlx::sqlite::SqlitePool;

use crate::models::Note;

pub async fn get_notes(pool: &SqlitePool, title: String, exact: bool) -> anyhow::Result<Vec<Note>> {
    let notes: Vec<Note> = if exact {
        let recs = sqlx::query!(
            r#"
    SELECT title, created, text
    FROM notes
    WHERE title = $1
            "#,
            title
        )
        .fetch_all(pool)
        .await?;

    recs.into_iter()
        .map(|rec| {
            Note { 
                title: rec.title,
                created: rec.created.parse::<DateTime<Local>>().unwrap(),
                text: rec.text,
            }
        }).collect()
    } else {
        let recs = sqlx::query!(
            r#"
    SELECT title, created, text
    FROM notes
    WHERE title LIKE $1
            "#,
            title
        )
        .fetch_all(pool)
        .await?;

    recs.into_iter()
        .map(|rec| {
            Note { 
                title: rec.title,
                created: rec.created.parse::<DateTime<Local>>().unwrap(),
                text: rec.text,
            }
        }).collect()
    };

    Ok(notes)
}

pub async fn get_all_notes(pool: &SqlitePool) -> anyhow::Result<Vec<Note>> {
    let notes = sqlx::query!(
        r#"
SELECT title, created, text
FROM notes
        "#
    )
    .fetch_all(pool)
    .await?;

    let notes: Vec<Note> = notes.into_iter()
        .map(|rec| {
            Note { 
                title: rec.title,
                created: rec.created.parse::<DateTime<Local>>().unwrap(),
                text: rec.text,
            }
        }).collect();

    Ok(notes)
}

pub async fn insert_note(pool: &SqlitePool, note: Note) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
INSERT INTO notes (title, created, text)
VALUES ( $1, $2, $3 )
        "#,
        note.title,
        note.created.to_string(),
        note.text,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_notes(pool: &SqlitePool, title: String, exact: bool) -> anyhow::Result<()> {
    if exact {
        sqlx::query!(
            r#"
    DELETE FROM notes
    WHERE title = $1
            "#,
            title
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            r#"
    DELETE FROM notes
    WHERE title LIKE $1
            "#,
            title
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn update_notes(pool: &SqlitePool, title: String, note: Note) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
UPDATE notes
SET title = $1,
    created = $2,
    text = $3
WHERE title = $4
        "#,
        note.title,
        note.created.to_string(),
        note.text,
        title,
    )
    .execute(pool)
    .await?;

    Ok(())
}
