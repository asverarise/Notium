<script lang="ts">
  import { marked } from 'marked';
  import type { Note } from '$lib/types';
  import { pickAndStoreImage } from '$lib/storage';

  interface Props {
    note: Note;
    noteId: string;
    view: 'editor' | 'preview';
    onUpdate: (updates: Partial<Note>) => void;
    onViewChange: (v: 'editor' | 'preview') => void;
    onAddTag: (tag: string) => void;
    onRemoveTag: (tag: string) => void;
  }

  let { note, noteId, view, onUpdate, onViewChange, onAddTag, onRemoveTag }: Props = $props();

  let tagInput = $state('');
  let tagInputEl = $state<HTMLInputElement | undefined>();
  let textareaEl = $state<HTMLTextAreaElement | undefined>();
  let isSaving = $state(false);
  let isInsertingImage = $state(false);
  let imageErrorMsg = $state('');
  let saveTimeout: ReturnType<typeof setTimeout>;
  let errorTimeout: ReturnType<typeof setTimeout>;
  let charCount = $derived(note.content.length);
  let wordCount = $derived(note.content.trim() ? note.content.trim().split(/\s+/).length : 0);

  const renderedContent = $derived(
    marked.parse(note.content, { async: false }) as string
  );

  function handleContentChange(e: Event) {
    const val = (e.target as HTMLTextAreaElement).value;
    scheduleUpdate({ content: val });
  }

  function handleTitleInput(e: Event) {
    const val = (e.target as HTMLElement).innerText;
    scheduleUpdate({ title: val.replace(/\n/g, '').trim() || 'Untitled' });
  }

  function scheduleUpdate(updates: Partial<Note>) {
    onUpdate(updates);
    isSaving = true;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      isSaving = false;
    }, 800);
  }

  function handleTagKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      submitTag();
    } else if (e.key === 'Backspace' && !tagInput && note.tags.length > 0) {
      onRemoveTag(note.tags[note.tags.length - 1]);
    }
  }

  function submitTag() {
    const tag = tagInput.trim().toLowerCase().replace(/[^a-z0-9-]/g, '');
    if (tag && !note.tags.includes(tag)) {
      onAddTag(tag);
    }
    tagInput = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab' && view === 'editor') {
      e.preventDefault();
      const ta = textareaEl;
      if (!ta) return;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      const val = ta.value;
      ta.value = val.substring(0, start) + '  ' + val.substring(end);
      ta.selectionStart = ta.selectionEnd = start + 2;
      scheduleUpdate({ content: ta.value });
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
      e.preventDefault();
      onViewChange(view === 'editor' ? 'preview' : 'editor');
    }
  }

  function formatDate(ts: number): string {
    return new Date(ts).toLocaleString(undefined, {
      month: 'short', day: 'numeric', year: 'numeric',
      hour: '2-digit', minute: '2-digit',
    });
  }

  // ── Image insertion ───────────────────────────────────────────────────────────

  function showError(msg: string) {
    imageErrorMsg = msg;
    clearTimeout(errorTimeout);
    errorTimeout = setTimeout(() => { imageErrorMsg = ''; }, 4000);
  }

  async function insertImage() {
    if (isInsertingImage) return;
    isInsertingImage = true;
    imageErrorMsg = '';
    try {
      const img = await pickAndStoreImage(noteId);
      // Build the markdown snippet
      const snippet = `![image](notium-img://${img.id})`;

      // Insert at cursor position in editor mode; append if in preview mode
      if (view === 'editor' && textareaEl) {
        const ta = textareaEl;
        const start = ta.selectionStart;
        const end = ta.selectionEnd;
        const before = ta.value.substring(0, start);
        const after = ta.value.substring(end);
        // Ensure a newline before and after
        const prefix = before && !before.endsWith('\n') ? '\n' : '';
        const suffix = after && !after.startsWith('\n') ? '\n' : '';
        const newContent = before + prefix + snippet + suffix + after;
        ta.value = newContent;
        const pos = start + prefix.length + snippet.length + suffix.length;
        ta.selectionStart = ta.selectionEnd = pos;
        ta.focus();
        scheduleUpdate({ content: newContent });
      } else {
        // In preview mode, append to the content
        const newContent = note.content + '\n\n' + snippet + '\n';
        scheduleUpdate({ content: newContent });
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      // User cancelled – not an error worth showing
      if (!msg.toLowerCase().includes('no file selected') && msg !== 'No file selected') {
        showError(msg);
      }
    } finally {
      isInsertingImage = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="editor-root">
  <!-- Toolbar -->
  <div class="editor-toolbar">
    <!-- View toggle -->
    <div class="view-toggle">
      <button
        id="editor-view-btn"
        onclick={() => onViewChange('editor')}
        class="view-btn {view === 'editor' ? 'active' : ''}"
      >
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
        </svg>
        Edit
      </button>
      <button
        id="preview-view-btn"
        onclick={() => onViewChange('preview')}
        class="view-btn {view === 'preview' ? 'active' : ''}"
      >
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
          <path stroke-linecap="round" stroke-linejoin="round" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
        </svg>
        Preview
      </button>
    </div>

    <!-- Centre: Image insert button -->
    <div class="toolbar-centre">
      <button
        id="insert-image-btn"
        class="img-btn {isInsertingImage ? 'loading' : ''}"
        onclick={insertImage}
        disabled={isInsertingImage}
        title="Insert image (local file)"
        aria-label="Insert image"
      >
        {#if isInsertingImage}
          <!-- Spinner -->
          <svg class="spin" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
          </svg>
          Embedding…
        {:else}
          <!-- Image icon -->
          <svg fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/>
          </svg>
          Insert Image
        {/if}
      </button>

      <!-- Error toast inline -->
      {#if imageErrorMsg}
        <span class="img-error animate-fade-in">⚠ {imageErrorMsg}</span>
      {/if}
    </div>

    <!-- Status -->
    <div class="editor-status">
      <span>{wordCount} words</span>
      <span>{charCount} chars</span>
      {#if isSaving}
        <span class="status-saving">
          <span class="status-dot saving"></span>
          Saving…
        </span>
      {:else}
        <span class="status-saved">
          <span class="status-dot saved"></span>
          Saved
        </span>
      {/if}
    </div>
  </div>

  <!-- Title -->
  <div class="title-area">
    <div
      id="note-title"
      contenteditable="true"
      role="textbox"
      aria-label="Note title"
      aria-multiline="false"
      tabindex="0"
      oninput={handleTitleInput}
      class="note-title"
      style="word-break: break-word;"
    >{note.title}</div>

    <!-- Meta -->
    <div class="note-meta">
      <span>Created {formatDate(note.createdAt)}</span>
      <span>·</span>
      <span>Modified {formatDate(note.updatedAt)}</span>
    </div>

    <!-- Tags -->
    <div class="tags-row">
      {#each note.tags as tag}
        <span class="tag-pill">
          #{tag}
          <button
            onclick={() => onRemoveTag(tag)}
            class="tag-remove"
            aria-label="Remove tag {tag}"
          >×</button>
        </span>
      {/each}
      <div class="tag-input-wrap">
        <input
          bind:this={tagInputEl}
          bind:value={tagInput}
          onkeydown={handleTagKeydown}
          onblur={submitTag}
          placeholder={note.tags.length === 0 ? 'Add tags…' : '+tag'}
          class="tag-input"
        />
      </div>
    </div>
  </div>

  <div class="editor-body">
    {#if view === 'editor'}
      <!-- Editor -->
      <textarea
        bind:this={textareaEl}
        id="note-content-editor"
        value={note.content}
        oninput={handleContentChange}
        placeholder="Start writing… (Markdown supported)"
        spellcheck="true"
        class="note-textarea animate-fade-in"
        style="font-family: 'JetBrains Mono', monospace;"
      ></textarea>
    {:else}
      <!-- Preview -->
      <div class="markdown-body preview-area animate-fade-in">
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html renderedContent}
      </div>
    {/if}
  </div>
</div>

<style>
  .editor-root {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-app);
    transition: background 0.25s ease;
  }

  /* Toolbar */
  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 24px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: 12px;
    transition: border-color 0.25s ease;
  }
  .view-toggle {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--bg-panel);
    border-radius: 8px;
    padding: 4px;
    transition: background 0.25s ease;
  }
  .view-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 12px;
    border-radius: 6px;
    border: none;
    background: transparent;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    color: var(--text-faint);
    transition: all 0.15s ease;
  }
  .view-btn:hover { color: var(--text-primary); }
  .view-btn svg { width: 13px; height: 13px; }
  .view-btn.active {
    background: var(--toolbar-tab-active-bg);
    color: var(--toolbar-tab-active-text);
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }

  /* ── Image button ── */
  .toolbar-centre {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
  }

  .img-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 13px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    color: var(--text-secondary);
    transition: all 0.18s ease;
    white-space: nowrap;
  }
  .img-btn svg {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }
  .img-btn:hover:not(:disabled) {
    background: rgba(99, 102, 241, 0.12);
    border-color: rgba(99, 102, 241, 0.45);
    color: #818cf8;
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.10);
  }
  .img-btn:active:not(:disabled) {
    transform: scale(0.97);
  }
  .img-btn:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .img-btn.loading {
    color: #818cf8;
    border-color: rgba(99, 102, 241, 0.4);
  }

  /* Spinner animation */
  @keyframes spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }
  .spin {
    animation: spin 0.9s linear infinite;
  }

  /* Inline error */
  .img-error {
    font-size: 11px;
    color: #f87171;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: 6px;
    padding: 3px 8px;
    max-width: 240px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .editor-status {
    display: flex;
    align-items: center;
    gap: 14px;
    font-size: 11px;
    color: var(--text-faint);
    flex-shrink: 0;
  }
  .status-saving {
    display: flex;
    align-items: center;
    gap: 4px;
    color: #818cf8;
  }
  .status-saved {
    display: flex;
    align-items: center;
    gap: 4px;
    color: rgba(52,211,153,0.8);
  }
  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
  .status-dot.saving { background: #818cf8; animation: pulse 1.5s ease-in-out infinite; }
  .status-dot.saved { background: rgba(52,211,153,0.8); }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  /* Title area */
  .title-area {
    padding: 28px 32px 10px;
    flex-shrink: 0;
  }
  .note-title {
    font-size: 24px;
    font-weight: 700;
    color: var(--text-title);
    outline: none;
    width: 100%;
    cursor: text;
    letter-spacing: -0.02em;
    line-height: 1.25;
    transition: color 0.25s ease;
  }
  .note-title:empty::before {
    content: 'Untitled';
    color: var(--text-placeholder);
  }
  .note-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
    font-size: 11px;
    color: var(--text-faint);
  }
  .tags-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
    margin-top: 10px;
  }
  .tag-pill {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 2px 8px;
    background: var(--tag-pill-bg);
    border: 1px solid var(--tag-pill-border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--tag-pill-text);
    transition: all 0.15s ease;
  }
  .tag-remove {
    opacity: 0;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 0;
    font-size: 13px;
    line-height: 1;
    margin-left: 1px;
    transition: all 0.12s ease;
  }
  .tag-pill:hover .tag-remove { opacity: 1; }
  .tag-remove:hover { color: #f87171; }
  .tag-input-wrap { display: flex; align-items: center; }
  .tag-input {
    font-size: 11px;
    color: var(--tag-pill-text);
    background: transparent;
    outline: none;
    border: none;
    width: 80px;
    transition: width 0.2s ease;
  }
  .tag-input::placeholder { color: var(--text-placeholder); }
  .tag-input:focus { width: 120px; }

  /* Editor body */
  .editor-body { flex: 1; overflow: hidden; }
  .note-textarea {
    width: 100%;
    height: 100%;
    resize: none;
    background: transparent;
    padding: 16px 32px;
    font-size: var(--editor-font-size, 14px);
    line-height: 1.75;
    color: var(--text-secondary);
    outline: none;
    overflow-y: auto;
    transition: color 0.25s ease, font-size 0.2s ease;
  }
  .note-textarea::placeholder { color: var(--text-placeholder); }
  .preview-area {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    padding: 16px 32px;
    font-size: var(--editor-font-size, 14px);
    transition: font-size 0.2s ease;
  }

  /* Make images in preview nicely styled */
  :global(.preview-area img) {
    max-width: 100%;
    border-radius: 10px;
    margin: 12px 0;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.18);
    display: block;
  }
</style>
