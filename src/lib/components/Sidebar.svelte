<script lang="ts">
  import type { Note } from '$lib/types';
  import ThemeToggle from './ThemeToggle.svelte';

  interface Props {
    notes: Note[];
    activeNoteId: string | null;
    searchQuery: string;
    selectedTags: string[];
    onSelectNote: (id: string) => void;
    onNewNote: () => void;
    onDeleteNote: (id: string) => void;
    onTogglePin: (id: string) => void;
    onSearchChange: (q: string) => void;
    onTagToggle: (tag: string) => void;
  }

  let {
    notes,
    activeNoteId,
    searchQuery,
    selectedTags,
    onSelectNote,
    onNewNote,
    onDeleteNote,
    onTogglePin,
    onSearchChange,
    onTagToggle,
  }: Props = $props();

  let contextMenu = $state<{ noteId: string; x: number; y: number } | null>(null);
  let searchInputEl = $state<HTMLInputElement | undefined>();

  // Collect all unique tags
  const allTags = $derived(
    [...new Set(notes.flatMap((n) => n.tags))].sort()
  );

  // Filter notes
  const filteredNotes = $derived(() => {
    let result = [...notes];
    const q = searchQuery.trim().toLowerCase();
    if (q) {
      result = result.filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.content.toLowerCase().includes(q) ||
          n.tags.some((t) => t.toLowerCase().includes(q))
      );
    }
    if (selectedTags.length > 0) {
      result = result.filter((n) =>
        selectedTags.every((t) => n.tags.includes(t))
      );
    }
    // Pinned first, then by updatedAt
    return result.sort((a, b) => {
      if (a.isPinned !== b.isPinned) return a.isPinned ? -1 : 1;
      return b.updatedAt - a.updatedAt;
    });
  });

  function formatDate(ts: number): string {
    const d = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - d.getTime();
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    if (diff < 604800000) return `${Math.floor(diff / 86400000)}d ago`;
    return d.toLocaleDateString();
  }

  function getPreview(content: string): string {
    // Strip markdown for preview
    return content
      .replace(/^#{1,6}\s+/gm, '')
      .replace(/\*\*(.*?)\*\*/g, '$1')
      .replace(/\*(.*?)\*/g, '$1')
      .replace(/`(.*?)`/g, '$1')
      .replace(/\[(.*?)\]\(.*?\)/g, '$1')
      .replace(/\n+/g, ' ')
      .trim()
      .slice(0, 100);
  }

  function openContext(e: MouseEvent, noteId: string) {
    e.preventDefault();
    contextMenu = { noteId, x: e.clientX, y: e.clientY };
  }

  function closeContext() {
    contextMenu = null;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      searchInputEl?.focus();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} onclick={closeContext} />

<aside class="sidebar">
  <!-- Header -->
  <div class="sidebar-header">
    <div class="sidebar-logo">
      <div class="logo-icon">
        <img src="/notium.svg" alt="Notium Logo" class="logo-img" />
      </div>
      <span class="logo-text">Notium</span>
    </div>
    <div class="header-actions">
      <ThemeToggle />
      <button
        id="new-note-btn"
        onclick={onNewNote}
        title="New Note (Ctrl+N)"
        class="new-note-btn"
      >
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Search -->
  <div class="search-wrap">
    <div class="search-inner">
      <svg class="search-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
      <input
        bind:this={searchInputEl}
        id="search-input"
        type="text"
        placeholder="Search notes… (⌘K)"
        value={searchQuery}
        oninput={(e) => onSearchChange((e.target as HTMLInputElement).value)}
        class="search-input"
      />
    </div>
  </div>

  <!-- Tag filters -->
  {#if allTags.length > 0}
    <div class="tags-wrap">
      <div class="tags-list">
        {#each allTags as tag}
          <button
            onclick={() => onTagToggle(tag)}
            class="tag-filter-btn {selectedTags.includes(tag) ? 'active' : ''}"
          >
            #{tag}
          </button>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Note count -->
  <div class="notes-count-row">
    <span class="notes-label">Notes</span>
    <span class="notes-count">{filteredNotes().length}</span>
  </div>

  <!-- Notes list -->
  <div class="notes-list">
    {#each filteredNotes() as note (note.id)}
      <button
        id="note-{note.id}"
        onclick={() => onSelectNote(note.id)}
        oncontextmenu={(e) => openContext(e, note.id)}
        class="note-item {activeNoteId === note.id ? 'active' : ''} animate-slide-in"
      >
        <div class="note-item-row">
          {#if note.isPinned}
            <svg class="pin-icon" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
            </svg>
          {/if}
          <div class="note-meta">
            <p class="note-title {activeNoteId === note.id ? 'active' : ''}">
              {note.title || 'Untitled'}
            </p>
            <p class="note-preview">{getPreview(note.content)}</p>
            <div class="note-footer">
              <span class="note-timestamp">{formatDate(note.updatedAt)}</span>
              {#if note.tags.length > 0}
                <div class="note-tags">
                  {#each note.tags.slice(0, 2) as tag}
                    <span class="note-tag">#{tag}</span>
                  {/each}
                  {#if note.tags.length > 2}
                    <span class="note-tag-more">+{note.tags.length - 2}</span>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        </div>
      </button>
    {/each}

    {#if filteredNotes().length === 0}
      <div class="empty-notes animate-fade-in">
        <div class="empty-notes-icon">
          <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
          </svg>
        </div>
        <p class="empty-notes-text">No notes found</p>
        <button onclick={onNewNote} class="empty-notes-cta">
          Create a new note →
        </button>
      </div>
    {/if}
  </div>
</aside>

<!-- Context menu -->
{#if contextMenu}
  <div
    id="context-menu"
    style="position:fixed; top:{contextMenu.y}px; left:{contextMenu.x}px; z-index:9999"
    class="context-menu animate-fade-in"
    role="menu"
    tabindex="-1"
    onkeydown={(e) => { if (e.key === 'Escape') closeContext(); }}
    onclick={(e) => e.stopPropagation()}
  >
    <button
      class="context-item"
      onclick={() => { if (contextMenu) onTogglePin(contextMenu.noteId); closeContext(); }}
    >
      <svg class="context-item-icon indigo" fill="currentColor" viewBox="0 0 24 24">
        <path d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"/>
      </svg>
      {notes.find(n => n.id === contextMenu?.noteId)?.isPinned ? 'Unpin' : 'Pin to top'}
    </button>
    <div class="context-divider"></div>
    <button
      class="context-item danger"
      onclick={() => { if (contextMenu) onDeleteNote(contextMenu.noteId); closeContext(); }}
    >
      <svg class="context-item-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
      </svg>
      Delete note
    </button>
  </div>
{/if}

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    user-select: none;
    transition: background 0.25s ease, border-color 0.25s ease;
  }

  /* Header */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }
  .sidebar-logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .logo-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .logo-img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .logo-text {
    font-size: 15px;
    font-weight: 600;
    color: var(--logo-text);
    letter-spacing: -0.01em;
    transition: color 0.25s ease;
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .new-note-btn {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: #6366f1;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    color: white;
    padding: 0;
    box-shadow: 0 2px 8px rgba(99,102,241,0.3);
  }
  .new-note-btn:hover { background: #818cf8; transform: scale(1.05); }
  .new-note-btn:active { transform: scale(0.95); }
  .new-note-btn svg { width: 14px; height: 14px; }

  /* Search */
  .search-wrap {
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }
  .search-inner { position: relative; }
  .search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    width: 14px;
    height: 14px;
    color: var(--text-faint);
    pointer-events: none;
  }
  .search-input {
    width: 100%;
    padding: 7px 10px 7px 32px;
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 13px;
    color: var(--text-primary);
    outline: none;
    transition: all 0.15s ease;
  }
  .search-input::placeholder { color: var(--text-placeholder); }
  .search-input:focus {
    border-color: rgba(99,102,241,0.5);
    box-shadow: 0 0 0 2px rgba(99,102,241,0.12);
  }

  /* Tags */
  .tags-wrap {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }
  .tags-list { display: flex; flex-wrap: wrap; gap: 6px; }
  .tag-filter-btn {
    padding: 2px 8px;
    border-radius: 999px;
    border: none;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    background: var(--bg-tag-active);
    color: var(--text-muted);
  }
  .tag-filter-btn:hover { background: var(--bg-tag-hover); color: var(--text-primary); }
  .tag-filter-btn.active { background: #6366f1; color: white; }

  /* Notes count */
  .notes-count-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 16px;
  }
  .notes-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }
  .notes-count { font-size: 10px; color: var(--text-faint); }

  /* Notes list */
  .notes-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 8px 16px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .note-item {
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .note-item:hover:not(.active) {
    background: var(--bg-hover);
    border-color: var(--border);
  }
  .note-item.active {
    background: var(--bg-active);
    border-color: var(--border-active);
    box-shadow: 0 1px 4px rgba(99,102,241,0.08);
  }
  .note-item-row { display: flex; align-items: flex-start; gap: 6px; }
  .pin-icon { width: 12px; height: 12px; color: #818cf8; margin-top: 2px; flex-shrink: 0; }
  .note-meta { flex: 1; min-width: 0; }
  .note-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--note-name);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
    transition: color 0.25s ease;
  }
  .note-title.active { color: var(--note-name-active); }
  .note-preview {
    font-size: 11px;
    color: var(--note-preview);
    margin: 3px 0 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.5;
  }
  .note-footer { display: flex; align-items: center; gap: 6px; margin-top: 5px; }
  .note-timestamp { font-size: 10px; color: var(--note-timestamp); }
  .note-tags { display: flex; gap: 4px; overflow: hidden; }
  .note-tag { font-size: 10px; color: rgba(129,140,248,0.7); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .note-tag-more { font-size: 10px; color: var(--text-faint); }

  /* Empty notes */
  .empty-notes {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 16px;
    text-align: center;
  }
  .empty-notes-icon {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--empty-state-bg);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
  }
  .empty-notes-icon svg { width: 22px; height: 22px; color: var(--text-faint); }
  .empty-notes-text { font-size: 13px; color: var(--text-muted); margin: 0 0 8px; }
  .empty-notes-cta {
    background: none;
    border: none;
    font-size: 12px;
    color: #818cf8;
    cursor: pointer;
    transition: color 0.15s ease;
    padding: 0;
  }
  .empty-notes-cta:hover { color: #6366f1; }

  /* Context menu */
  .context-menu {
    background: var(--bg-dialog);
    border: 1px solid var(--border);
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.18);
    padding: 6px;
    min-width: 160px;
  }
  .context-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-radius: 8px;
    border: none;
    background: transparent;
    font-size: 13px;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.12s ease;
    text-align: left;
  }
  .context-item:hover { background: var(--bg-hover); color: var(--text-primary); }
  .context-item.danger { color: #f87171; }
  .context-item.danger:hover { background: rgba(239,68,68,0.08); color: #f87171; }
  .context-item-icon { width: 15px; height: 15px; flex-shrink: 0; }
  .context-item-icon.indigo { color: #818cf8; }
  .context-divider { height: 1px; background: var(--border); margin: 4px 8px; }
</style>
