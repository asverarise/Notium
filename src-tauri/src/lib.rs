mod db;

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use db::Note;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;

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

// ── Image commands ──

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StoredImage {
    id: String,
    mime_type: String,
}

/// Open the native OS file picker, read the selected image, store it as Base64
/// in SQLite, and return the image ID + mime type to the frontend.
/// The frontend uses the ID to build a `notium-img://ID` markdown URL.
#[tauri::command]
async fn pick_and_store_image(
    note_id: String,
    state: tauri::State<'_, DbState>,
    app: tauri::AppHandle,
) -> Result<StoredImage, String> {

    // Open native file dialog – filter to common image types
    let file_path = app
        .dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"])
        .blocking_pick_file()
        .ok_or("No file selected")?;

    // FilePath is an enum: Path(PathBuf) on desktop, Url(Url) on mobile.
    // On desktop we always get the Path variant.
    let path = match file_path {
        FilePath::Path(p) => p,
        FilePath::Url(u) => {
            return Err(format!("Content URI paths are not supported: {u}"));
        }
    };

    // Determine MIME type from extension
    let ext = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("png")
        .to_lowercase();

    let mime_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        "bmp"          => "image/bmp",
        "svg"          => "image/svg+xml",
        _              => "image/png",
    }.to_string();

    // Read bytes and encode to Base64
    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {e}"))?;

    // Guard: reject files over 10 MB (10 * 1024 * 1024 bytes)
    if bytes.len() > 10 * 1024 * 1024 {
        return Err("Image is too large (max 10 MB).".into());
    }

    let data_b64 = BASE64.encode(&bytes);

    // Generate a unique ID for this image
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    // Persist to SQLite
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::insert_image(&conn, &id, &note_id, &mime_type, &data_b64, now)
        .map_err(|e| format!("Failed to save image: {e}"))?;

    Ok(StoredImage { id, mime_type })
}

/// Delete all images associated with a note (call alongside delete_note).
#[tauri::command]
fn delete_note_images(note_id: String, state: tauri::State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_images_for_note(&conn, &note_id).map_err(|e| e.to_string())
}

// ── Entry point ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // ── Custom URI protocol: notium-img://<image-id> ──
        // Reads the image from SQLite and returns the raw bytes with the correct
        // MIME type so <img src="notium-img://uuid"> works in the webview.
        .register_uri_scheme_protocol("notium-img", |ctx, req| {
            // Extract the image ID from the URI host/path
            // URI shape: notium-img://uuid  →  host = uuid, path = "/"
            let uri = req.uri().to_string();
            let id = uri
                .strip_prefix("notium-img://")
                .unwrap_or("")
                .trim_end_matches('/')
                .to_string();

            if id.is_empty() {
                return tauri::http::Response::builder()
                    .status(400)
                    .body(b"Missing image ID".to_vec())
                    .unwrap();
            }

            let state = ctx.app_handle().state::<DbState>();
            let conn = match state.0.lock() {
                Ok(c) => c,
                Err(e) => {
                    return tauri::http::Response::builder()
                        .status(500)
                        .body(format!("DB lock error: {e}").into_bytes())
                        .unwrap();
                }
            };

            match db::get_image(&conn, &id) {
                Ok((mime, b64)) => {
                    let bytes = BASE64.decode(b64).unwrap_or_default();
                    tauri::http::Response::builder()
                        .status(200)
                        .header("Content-Type", mime)
                        .header("Cache-Control", "max-age=31536000, immutable")
                        .body(bytes)
                        .unwrap()
                }
                Err(_) => tauri::http::Response::builder()
                    .status(404)
                    .body(b"Image not found".to_vec())
                    .unwrap(),
            }
        })
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
            pick_and_store_image,
            delete_note_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
