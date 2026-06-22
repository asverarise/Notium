import { invoke } from '@tauri-apps/api/core';
import type { Note } from './types';

// ── Note factory ──────────────────────────────────────────────────────────────

/** Build a new Note object locally; the Rust backend persists it via upsertNote(). */
export function createNote(title: string = 'Untitled Note'): Note {
  const now = Date.now();
  return {
    id: crypto.randomUUID(),
    title,
    content: `# ${title}\n\nStart writing your note here...`,
    tags: [],
    createdAt: now,
    updatedAt: now,
    isPinned: false,
  };
}

// ── Tauri command wrappers ────────────────────────────────────────────────────

/**
 * Fetch all notes from the Rust/SQLite backend.
 * Returns notes ordered by pinned first, then most-recently updated.
 */
export async function loadNotes(): Promise<Note[]> {
  try {
    return await invoke<Note[]>('get_notes');
  } catch (e) {
    console.error('[storage] get_notes failed:', e);
    return [];
  }
}

/**
 * Create or update a single note in SQLite (upsert by id).
 * Call this whenever a note is created or any field changes.
 */
export async function saveNote(note: Note): Promise<void> {
  try {
    await invoke('upsert_note', { note });
  } catch (e) {
    console.error('[storage] upsert_note failed:', e);
  }
}

/**
 * Permanently delete a note from SQLite.
 */
export async function deleteNoteById(id: string): Promise<void> {
  try {
    await invoke('delete_note', { id });
  } catch (e) {
    console.error('[storage] delete_note failed:', e);
  }
}

// ── Image command wrappers ────────────────────────────────────────────────────

export interface StoredImage {
  id: string;
  mimeType: string;
}

/**
 * Opens the native OS file picker, reads the selected image, stores it as
 * Base64 in SQLite (associated with `noteId`), and returns the image ID and
 * MIME type. The caller should insert `![image](notium-img://ID)` into the
 * note content at the cursor position.
 *
 * Throws if the user cancels the picker or if the file is >10 MB.
 */
export async function pickAndStoreImage(noteId: string): Promise<StoredImage> {
  return await invoke<StoredImage>('pick_and_store_image', { noteId });
}

/**
 * Delete all images stored for a note. Call this alongside deleteNoteById
 * so orphaned image blobs are not left in the database.
 */
export async function deleteNoteImages(noteId: string): Promise<void> {
  try {
    await invoke('delete_note_images', { noteId });
  } catch (e) {
    console.error('[storage] delete_note_images failed:', e);
  }
}
