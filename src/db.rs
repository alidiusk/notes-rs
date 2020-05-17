use chrono::{Local, DateTime};
use sqlx::sqlite::SqlitePool;

use crate::models::Note;

pub async fn get_note(pool: &SqlitePool, id: i32) -> anyhow::Result<Option<Note>> {
    let rec = sqlx::query!(
        r#"
SELECT id, created, content
FROM notes
WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(rec) = rec {
        Ok(Some(Note {
            id: rec.id.unwrap(),
            created: rec.created.parse::<DateTime<Local>>().unwrap(),
            content: rec.content,
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_all_notes(pool: &SqlitePool) -> anyhow::Result<Vec<Note>> {
    let notes = sqlx::query!(
        r#"
SELECT id, created, content
FROM notes
        "#
    )
    .fetch_all(pool)
    .await?;

    let notes: Vec<Note> = notes.into_iter()
        .map(|rec| {
            Note { 
                id: rec.id.unwrap(),
                created: rec.created.parse::<DateTime<Local>>().unwrap(),
                content: rec.content,
            }
        }).collect();

    Ok(notes)
}

pub async fn insert_note(pool: &SqlitePool, note: Note) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
INSERT INTO notes (created, content)
VALUES ( $1, $2 )
        "#,
        note.created.to_string(),
        note.content,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_note(pool: &SqlitePool, id: i32) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
DELETE FROM notes
WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_note(pool: &SqlitePool, id: i32, note: Note) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
UPDATE notes
SET created = $1,
    content = $2
WHERE id = $3
        "#,
        note.created.to_string(),
        note.content,
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
