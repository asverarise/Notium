<script lang="ts">
  import { onMount } from 'svelte';
  import type { Note } from '$lib/types';
  import { loadNotes, saveNote, deleteNoteById, deleteNoteImages, createNote } from '$lib/storage';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Editor from '$lib/components/Editor.svelte';

  import '../app.css';

  // ── State ──────────────────────────────────────────────────────────────────
  let notes = $state<Note[]>([]);
  let activeNoteId = $state<string | null>(null);
  let searchQuery = $state('');
  let selectedTags = $state<string[]>([]);
  let view = $state<'editor' | 'preview'>('editor');
  let sidebarWidth = $state(260);
  let isResizing = $state(false);
  let showDeleteConfirm = $state<string | null>(null);
  let mounted = $state(false);
  /** Shows a subtle top-bar indicator while a save is in-flight */
  let isSyncing = $state(false);

  const activeNote = $derived(notes.find((n) => n.id === activeNoteId) ?? null);

  // ── Lifecycle ──────────────────────────────────────────────────────────────
  onMount(async () => {
    notes = await loadNotes();
    if (notes.length > 0) {
      const pinned = notes.find((n) => n.isPinned);
      activeNoteId = pinned?.id ?? notes[0].id;
    }
    mounted = true;
  });

  // ── Keyboard shortcuts ─────────────────────────────────────────────────────
  function handleGlobalKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      newNote();
    }
    const tag = (e.target as HTMLElement).tagName;
    if (tag !== 'INPUT' && tag !== 'TEXTAREA' && e.altKey) {
      if (e.key === 'ArrowLeft')  sidebarWidth = Math.max(200, sidebarWidth - 10);
      if (e.key === 'ArrowRight') sidebarWidth = Math.min(400, sidebarWidth + 10);
    }
  }

  // ── Note CRUD ──────────────────────────────────────────────────────────────
  async function newNote() {
    const n = createNote();
    notes = [n, ...notes];
    activeNoteId = n.id;
    view = 'editor';
    await persist(n);
  }

  function selectNote(id: string) {
    activeNoteId = id;
  }

  async function updateNote(updates: Partial<Note>) {
    const updated = notes.map((n) =>
      n.id === activeNoteId
        ? { ...n, ...updates, updatedAt: Date.now() }
        : n
    );
    notes = updated;
    const note = updated.find((n) => n.id === activeNoteId);
    if (note) await persist(note);
  }

  async function deleteNote(id: string) {
    notes = notes.filter((n) => n.id !== id);
    if (activeNoteId === id) {
      activeNoteId = notes[0]?.id ?? null;
    }
    showDeleteConfirm = null;
    // Delete note and its associated image blobs in parallel
    await Promise.all([deleteNoteById(id), deleteNoteImages(id)]);
  }

  async function togglePin(id: string) {
    const updated = notes.map((n) =>
      n.id === id ? { ...n, isPinned: !n.isPinned, updatedAt: Date.now() } : n
    );
    notes = updated;
    const note = updated.find((n) => n.id === id);
    if (note) await persist(note);
  }

  function addTag(tag: string) {
    if (!activeNote || activeNote.tags.includes(tag)) return;
    updateNote({ tags: [...activeNote.tags, tag] });
  }

  function removeTag(tag: string) {
    if (!activeNote) return;
    updateNote({ tags: activeNote.tags.filter((t) => t !== tag) });
  }

  /** Debounced write-through to SQLite.
   * Uses a ref to always write the most recent version of the note,
   * avoiding stale-closure issues when edits arrive faster than the debounce. */
  const pendingNotes = new Map<string, Note>();
  let saveTimer: ReturnType<typeof setTimeout>;
  async function persist(note: Note) {
    pendingNotes.set(note.id, note);
    clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      isSyncing = true;
      const writes = [...pendingNotes.values()];
      pendingNotes.clear();
      await Promise.all(writes.map(saveNote));
      isSyncing = false;
    }, 400);
  }

  // ── Sidebar resize ─────────────────────────────────────────────────────────
  function startResize(e: MouseEvent) {
    isResizing = true;
    e.preventDefault();
  }
  function onMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    sidebarWidth = Math.max(200, Math.min(400, e.clientX));
  }
  function stopResize() {
    isResizing = false;
  }
</script>

<svelte:window
  onkeydown={handleGlobalKey}
  onmousemove={onMouseMove}
  onmouseup={stopResize}
/>

{#if mounted}
  <div
    class="app-shell"
    class:cursor-col-resize={isResizing}
  >
    <!-- Sidebar -->
    <div class="sidebar-pane" style="width: {sidebarWidth}px">
      <Sidebar
        {notes}
        {activeNoteId}
        {searchQuery}
        {selectedTags}
        onSelectNote={selectNote}
        onNewNote={newNote}
        onDeleteNote={(id) => (showDeleteConfirm = id)}
        onTogglePin={togglePin}
        onSearchChange={(q) => (searchQuery = q)}
        onTagToggle={(t) =>
          (selectedTags = selectedTags.includes(t)
            ? selectedTags.filter((x) => x !== t)
            : [...selectedTags, t])}
      />
    </div>

    <!-- Resize handle -->
    <div
      class="resize-handle {isResizing ? 'active' : ''}"
      role="presentation"
      onmousedown={startResize}
    >
      <div class="resize-handle-hit"></div>
    </div>

    <!-- Editor area -->
    <div class="editor-pane">
      {#if activeNote}
        <Editor
          note={activeNote}
          noteId={activeNote.id}
          {view}
          onUpdate={updateNote}
          onViewChange={(v) => (view = v)}
          onAddTag={addTag}
          onRemoveTag={removeTag}
        />
      {:else}
        <!-- Empty state -->
        <div class="empty-state animate-fade-in">
          <div class="empty-icon animate-pulse-glow">
            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
          </div>
          <h2 class="empty-title">No note selected</h2>
          <p class="empty-desc">
            Select a note from the sidebar or create a new one to start writing.
          </p>
          <button id="empty-new-note-btn" onclick={newNote} class="empty-cta">
            <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4"/>
            </svg>
            New note
          </button>
        </div>
      {/if}
    </div>
  </div>

  <!-- Sync indicator (top-right corner) -->
  {#if isSyncing}
    <div class="sync-badge animate-fade-in" aria-live="polite">
      <span class="sync-dot"></span>
      Saving…
    </div>
  {/if}
{:else}
  <!-- Loading splash -->
  <div class="splash">
    <div class="splash-inner">
      <div class="splash-icon animate-pulse-glow">
        <img src="/notium.svg" alt="Notium Logo" class="splash-img" />
      </div>
      <p class="splash-text">Loading Notium…</p>
    </div>
  </div>
{/if}

<!-- Delete confirm dialog -->
{#if showDeleteConfirm}
  <div
    class="dialog-backdrop animate-fade-in"
    role="dialog"
    aria-modal="true"
    aria-label="Confirm delete"
    tabindex="-1"
  >
    <button
      class="dialog-backdrop-btn"
      aria-label="Close dialog"
      onclick={() => (showDeleteConfirm = null)}
    ></button>

    <div class="dialog-panel animate-fade-in" role="document">
      <div class="dialog-icon">
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
        </svg>
      </div>
      <h3 class="dialog-title">Delete note?</h3>
      <p class="dialog-desc">
        "{notes.find(n => n.id === showDeleteConfirm)?.title ?? 'This note'}" will be permanently deleted. This action cannot be undone.
      </p>
      <div class="dialog-actions">
        <button
          id="cancel-delete-btn"
          onclick={() => (showDeleteConfirm = null)}
          class="dialog-btn cancel"
        >
          Cancel
        </button>
        <button
          id="confirm-delete-btn"
          onclick={() => showDeleteConfirm && deleteNote(showDeleteConfirm)}
          class="dialog-btn danger"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-app);
    transition: background 0.25s ease;
  }
  .sidebar-pane {
    flex-shrink: 0;
    overflow: hidden;
    transition: width 0.05s ease;
  }
  .editor-pane { flex: 1; overflow: hidden; }

  /* Resize handle */
  .resize-handle {
    width: 4px;
    flex-shrink: 0;
    cursor: col-resize;
    background: transparent;
    position: relative;
    transition: background 0.15s ease;
  }
  .resize-handle:hover,
  .resize-handle.active { background: rgba(99,102,241,0.35); }
  .resize-handle-hit {
    position: absolute;
    top: 0;
    bottom: 0;
    left: -4px;
    right: -4px;
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
    background: var(--bg-app);
  }
  .empty-icon {
    width: 72px;
    height: 72px;
    border-radius: 20px;
    background: var(--empty-icon-bg);
    border: 1px solid var(--empty-icon-border);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 20px;
  }
  .empty-icon svg { width: 36px; height: 36px; color: #818cf8; }
  .empty-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--text-title);
    margin: 0 0 8px;
  }
  .empty-desc {
    font-size: 13px;
    color: var(--text-faint);
    margin: 0 0 24px;
    max-width: 280px;
    line-height: 1.6;
  }
  .empty-cta {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 10px 20px;
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    box-shadow: 0 4px 16px rgba(99,102,241,0.3);
  }
  .empty-cta:hover { background: #818cf8; transform: scale(1.04); }
  .empty-cta:active { transform: scale(0.97); }
  .empty-cta svg { width: 14px; height: 14px; }

  /* Loading splash */
  .splash {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    width: 100vw;
    background: var(--bg-app);
  }
  .splash-inner { display: flex; flex-direction: column; align-items: center; gap: 14px; }
  .splash-icon {
    width: 64px;
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .splash-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .splash-text { font-size: 13px; color: var(--text-faint); }

  /* Sync badge */
  .sync-badge {
    position: fixed;
    bottom: 16px;
    right: 16px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 11px;
    color: var(--text-faint);
    box-shadow: 0 2px 8px rgba(0,0,0,0.10);
    z-index: 100;
  }
  .sync-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #818cf8;
    animation: pulse 1.4s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  /* Dialog */
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(4px);
  }
  .dialog-backdrop-btn {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    cursor: default;
    background: transparent;
    border: none;
  }
  .dialog-panel {
    position: relative;
    background: var(--bg-dialog);
    border: 1px solid var(--border);
    border-radius: 20px;
    padding: 24px;
    max-width: 360px;
    width: calc(100% - 32px);
    box-shadow: 0 20px 60px rgba(0,0,0,0.25);
    transition: background 0.25s ease, border-color 0.25s ease;
  }
  .dialog-icon {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    background: rgba(239,68,68,0.1);
    border: 1px solid rgba(239,68,68,0.2);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 14px;
  }
  .dialog-icon svg { width: 22px; height: 22px; color: #f87171; }
  .dialog-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text-title);
    margin: 0 0 6px;
  }
  .dialog-desc {
    font-size: 13px;
    color: var(--text-muted);
    margin: 0 0 20px;
    line-height: 1.55;
  }
  .dialog-actions { display: flex; gap: 10px; }
  .dialog-btn {
    flex: 1;
    padding: 10px 16px;
    border-radius: 12px;
    border: none;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .dialog-btn.cancel {
    background: var(--bg-cancel-btn);
    color: var(--text-secondary);
  }
  .dialog-btn.cancel:hover { background: var(--bg-cancel-hover); }
  .dialog-btn.danger { background: #dc2626; color: white; }
  .dialog-btn.danger:hover { background: #ef4444; transform: scale(1.02); }
  .dialog-btn.danger:active { transform: scale(0.98); }
</style>
