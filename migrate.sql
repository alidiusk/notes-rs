PRAGMA foreign_keys=off;

BEGIN TRANSACTION;

ALTER TABLE notes RENAME TO _notes_old_2;

CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  created TEXT NOT NULL,
  content TEXT NOT NULL
);

INSERT INTO notes (created, content)
  SELECT created, text
  FROM _notes_old;

COMMIT;

PRAGMA foreign_keys=on;
