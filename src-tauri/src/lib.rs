mod db;

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use db::Note;

// ── Managed state ──

struct DbState(Mutex<Connection>);

// ── Tauri commands ──

/// Return all notes ordered by pinned first, then most-recently updated.
#[tauri::command]
fn get_notes(state: tauri::State<DbState>) -> Result<Vec<Note>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_all_notes(&conn).map_err(|e| e.to_string())
}

/// Create or update a note (upsert by primary key).
#[tauri::command]
fn upsert_note(note: Note, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::upsert_note(&conn, &note).map_err(|e| e.to_string())
}

/// Permanently delete a note by ID.
#[tauri::command]
fn delete_note(id: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_note(&conn, &id).map_err(|e| e.to_string())
}

// ── Entry point ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Resolve Tauri's per-app data directory:
            // Linux:   ~/.local/share/<identifier>/
            // macOS:   ~/Library/Application Support/<identifier>/
            // Windows: %APPDATA%\<identifier>\
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            // Make sure the directory exists.
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("notium.db");
            let conn = Connection::open(&db_path)
                .expect("failed to open SQLite database");

            // Run schema migrations.
            db::init_db(&conn).expect("failed to initialise database");

            // Seed default notes on very first launch.
            let count = db::note_count(&conn).unwrap_or(0);
            if count == 0 {
                db::seed_defaults(&conn).expect("failed to seed defaults");
            }

            // Register managed state.
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_notes,
            upsert_note,
            delete_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
