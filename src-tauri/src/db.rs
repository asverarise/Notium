use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

// ── Note model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    /// Stored as a JSON array string in the DB, serialised/deserialised here.
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_pinned: bool,
    pub color: Option<String>,
}

// ── Image model ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteImage {
    pub id: String,
    pub note_id: String,
    pub mime_type: String,
    /// Raw Base64 data (no data-URI prefix).
    pub data_b64: String,
    pub created_at: i64,
}

// ── Schema migration ───────────────────────────────────────────────────────────

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA temp_store = MEMORY;
         CREATE TABLE IF NOT EXISTS notes (
             id         TEXT    PRIMARY KEY NOT NULL,
             title      TEXT    NOT NULL DEFAULT '',
             content    TEXT    NOT NULL DEFAULT '',
             tags       TEXT    NOT NULL DEFAULT '[]',
             created_at INTEGER NOT NULL,
             updated_at INTEGER NOT NULL,
             is_pinned  INTEGER NOT NULL DEFAULT 0,
             color      TEXT
         );
         CREATE TABLE IF NOT EXISTS note_images (
             id         TEXT    PRIMARY KEY NOT NULL,
             note_id    TEXT    NOT NULL,
             mime_type  TEXT    NOT NULL DEFAULT 'image/png',
             data_b64   TEXT    NOT NULL,
             created_at INTEGER NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_note_images_note_id
             ON note_images (note_id);",
    )
}

// ── Note CRUD ─────────────────────────────────────────────────────────────────

pub fn get_all_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, tags, created_at, updated_at, is_pinned, color
         FROM notes
         ORDER BY is_pinned DESC, updated_at DESC",
    )?;

    let notes = stmt.query_map([], |row| {
        let tags_json: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let is_pinned_int: i64 = row.get(6)?;
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            is_pinned: is_pinned_int != 0,
            color: row.get(7)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(notes)
}

/// INSERT OR REPLACE – works for both create and update.
pub fn upsert_note(conn: &Connection, note: &Note) -> Result<()> {
    let tags_json = serde_json::to_string(&note.tags).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT OR REPLACE INTO notes
             (id, title, content, tags, created_at, updated_at, is_pinned, color)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            note.id,
            note.title,
            note.content,
            tags_json,
            note.created_at,
            note.updated_at,
            note.is_pinned as i64,
            note.color,
        ],
    )?;
    Ok(())
}

pub fn delete_note(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn note_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
}

// ── Image CRUD ────────────────────────────────────────────────────────────────

/// Insert a new image record.
pub fn insert_image(
    conn: &Connection,
    id: &str,
    note_id: &str,
    mime_type: &str,
    data_b64: &str,
    created_at: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO note_images (id, note_id, mime_type, data_b64, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, note_id, mime_type, data_b64, created_at],
    )?;
    Ok(())
}

/// Retrieve (mime_type, data_b64) for a single image by ID.
pub fn get_image(conn: &Connection, id: &str) -> Result<(String, String)> {
    conn.query_row(
        "SELECT mime_type, data_b64 FROM note_images WHERE id = ?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
}

/// Delete all images that belong to a note (called when the note itself is deleted).
pub fn delete_images_for_note(conn: &Connection, note_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM note_images WHERE note_id = ?1",
        params![note_id],
    )?;
    Ok(())
}

// ── Default seed data ─────────────────────────────────────────────────────────

pub fn seed_defaults(conn: &Connection) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();

    let defaults: Vec<Note> = vec![
        Note {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Welcome to Notium 🎉".into(),
            content: r#"# Welcome to Notium 🎉

Notium is your **rich knowledge manager** — fast, local, and beautiful.

## Features

- 📝 **Rich Markdown** editing with live preview
- 🏷️ **Tags** for easy organisation
- 🔍 **Full-text search** across all notes
- 📌 **Pin** important notes to the top
- 🖼️ **Images** — embed images directly into any note
- 💾 **SQLite** — your data lives locally, backed by Rust

## Markdown Cheatsheet

### Text Formatting
**Bold**, *italic*, ~~strikethrough~~, `inline code`

### Code Block
```js
const greeting = "Hello, Notium!";
console.log(greeting);
```

### Blockquote
> "The palest ink is better than the best memory."

---

Happy note-taking! 🚀"#.into(),
            tags: vec!["welcome".into(), "guide".into()],
            created_at: now - 86_400_000,
            updated_at: now - 3_600_000,
            is_pinned: true,
            color: None,
        },
        Note {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Meeting Notes – Q3 Planning".into(),
            content: r#"# Meeting Notes – Q3 Planning

**Attendees:** Alice, Bob, Carol

## Agenda

1. Review Q2 performance metrics
2. Define Q3 OKRs
3. Assign action items

## Key Decisions

- Launch new onboarding flow by end of July
- Increase marketing budget by **15%**
- Weekly sync every Tuesday at 10 AM

## Action Items

| Owner | Task | Due |
|-------|------|-----|
| Alice | Draft OKRs | Jun 20 |
| Bob   | Update roadmap | Jun 22 |
| Carol | Budget proposal | Jun 25 |"#.into(),
            tags: vec!["meetings".into(), "work".into(), "planning".into()],
            created_at: now - 43_200_000,
            updated_at: now - 1_800_000,
            is_pinned: false,
            color: None,
        },
        Note {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Ideas & Brainstorm".into(),
            content: r#"# Ideas & Brainstorm 💡

A running list of ideas to explore.

## App Ideas

- **Notium** – local-first knowledge manager ✅
- Recipe manager with ingredient tracking
- Habit tracker with streak visualisation

## Reading List

- [ ] *Deep Work* by Cal Newport
- [ ] *The Pragmatic Programmer*
- [x] *Atomic Habits* by James Clear

## Code Snippets

```python
# Quick fibonacci
def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```"#.into(),
            tags: vec!["ideas".into(), "personal".into()],
            created_at: now - 7_200_000,
            updated_at: now - 600_000,
            is_pinned: false,
            color: None,
        },
    ];

    for note in &defaults {
        upsert_note(conn, note)?;
    }
    Ok(())
}
