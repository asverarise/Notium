<script lang="ts">
  import {
    theme,
    fontSize,
    fontSizeMap,
    fontSizeOrder,
    type Theme,
    type FontSize,
  } from "$lib/theme";
  import tauriConf from "../../../src-tauri/tauri.conf.json";

  const appName = tauriConf.productName;
  const appVersion = tauriConf.version;
  const appIdentifier = tauriConf.identifier;
  const idParts = appIdentifier.split(".");
  const appAuthor =
    idParts.length > 1
      ? idParts[1].charAt(0).toUpperCase() + idParts[1].slice(1)
      : "Unknown";

  let open = $state(false);
  let wrapperEl = $state<HTMLDivElement | undefined>();
  let btnEl = $state<HTMLButtonElement | undefined>();

  // Fixed-position coords for the panel (so it escapes sidebar overflow:hidden)
  let panelTop = $state(0);
  let panelLeft = $state(0);
  const PANEL_WIDTH = 228;

  function calcPosition() {
    if (!btnEl) return;
    const r = btnEl.getBoundingClientRect();
    panelTop = r.bottom + 8;
    // keep panel inside viewport on the left side
    panelLeft = Math.min(
      r.right - PANEL_WIDTH,
      window.innerWidth - PANEL_WIDTH - 8,
    );
    if (panelLeft < 8) panelLeft = 8;
  }

  function toggleOpen(e: MouseEvent) {
    e.stopPropagation();
    if (!open) calcPosition();
    open = !open;
  }

  // ── Theme options ──────────────────────────────────────────────────────────
  const themeOptions: { value: Theme; label: string; icon: string }[] = [
    {
      value: "light",
      label: "Light",
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="4"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
      </svg>`,
    },
    {
      value: "dark",
      label: "Dark",
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>`,
    },
    {
      value: "system",
      label: "System",
      icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="2" y="3" width="20" height="14" rx="2"/>
        <path d="M8 21h8M12 17v4"/>
      </svg>`,
    },
  ];

  // ── Helpers ────────────────────────────────────────────────────────────────
  function handleOutsideClick(e: MouseEvent) {
    if (!open) return;
    const target = e.target as Node;
    const panelEl = document.getElementById("settings-panel-portal");
    if (wrapperEl?.contains(target) || panelEl?.contains(target)) return;
    open = false;
  }
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") open = false;
  }
  function handleResize() {
    if (open) calcPosition();
  }

  const currentFontIdx = $derived(fontSizeOrder.indexOf($fontSize));

  function stepFont(dir: -1 | 1) {
    const next = fontSizeOrder[currentFontIdx + dir];
    if (next) fontSize.set(next);
  }
</script>

<svelte:window
  onkeydown={handleKeydown}
  onclick={handleOutsideClick}
  onresize={handleResize}
/>

<!-- Trigger wrapper -->
<div class="settings-wrapper" bind:this={wrapperEl}>
  <button
    bind:this={btnEl}
    id="settings-btn"
    onclick={toggleOpen}
    title="Settings"
    class="settings-btn"
    aria-label="Open settings"
    aria-expanded={open}
  >
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
      />
    </svg>
  </button>
</div>

<!-- Portal: rendered at fixed position so sidebar overflow:hidden can't clip it -->
{#if open}
  <div
    id="settings-panel-portal"
    class="settings-panel animate-fade-in"
    style="top: {panelTop}px; left: {panelLeft}px; width: {PANEL_WIDTH}px;"
    role="dialog"
    aria-label="Settings"
    tabindex="-1"
  >
    <!-- ── Appearance ──────────────────────────────────────────────────── -->
    <p class="section-label">Appearance</p>
    <div class="theme-row">
      {#each themeOptions as opt}
        <button
          class="theme-chip {$theme === opt.value ? 'active' : ''}"
          onclick={() => theme.set(opt.value)}
          title={opt.label}
          aria-label="Theme: {opt.label}"
        >
          <span class="chip-icon">{@html opt.icon}</span>
          <span class="chip-label">{opt.label}</span>
          {#if $theme === opt.value}
            <span class="chip-dot"></span>
          {/if}
        </button>
      {/each}
    </div>

    <div class="divider"></div>

    <!-- ── Text Size ───────────────────────────────────────────────────── -->
    <p class="section-label">Text Size</p>

    <!-- Stepper row -->
    <div class="font-stepper">
      <button
        class="step-btn"
        onclick={() => stepFont(-1)}
        disabled={currentFontIdx === 0}
        aria-label="Decrease text size"
        title="Smaller"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M20 12H4" />
        </svg>
      </button>

      <div class="font-label-area">
        <span
          class="font-sample"
          style="font-size: {fontSizeMap[$fontSize].editor}">Aa</span
        >
        <span class="font-size-name">{fontSizeMap[$fontSize].label}</span>
      </div>

      <button
        class="step-btn"
        onclick={() => stepFont(1)}
        disabled={currentFontIdx === fontSizeOrder.length - 1}
        aria-label="Increase text size"
        title="Larger"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>
    </div>

    <!-- Segmented size picker -->
    <div class="font-segments">
      {#each fontSizeOrder as size, i}
        <button
          class="seg-btn {$fontSize === size ? 'active' : ''}"
          onclick={() => fontSize.set(size)}
          aria-label="Text size {fontSizeMap[size].label}"
          title={fontSizeMap[size].label}
        >
          <span
            style="font-size: {10 + i * 2}px; line-height: 1; font-weight: 600;"
            >A</span
          >
        </button>
      {/each}
    </div>

    <div class="divider"></div>

    <!-- ── About ───────────────────────────────────────────────────────── -->
    <p class="section-label">About</p>
    <div class="about-section">
      <img src="/notium.svg" alt="{appName} Logo" class="about-logo" />
      <div class="about-info">
        <div class="about-header">
          <span class="about-name">{appName}</span>
          <span class="about-version">v{appVersion}</span>
        </div>
        <span class="about-detail">by {appAuthor}</span>
        <span class="about-detail">{appIdentifier}</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-wrapper {
    position: relative;
  }

  /* Trigger button */
  .settings-btn {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
    color: var(--text-faint);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
  }
  .settings-btn svg {
    width: 14px;
    height: 14px;
    pointer-events: none;
  }
  .settings-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: rgba(99, 102, 241, 0.4);
    transform: scale(1.05);
  }

  /* Panel — rendered as a fixed portal to escape sidebar overflow:hidden */
  :global(#settings-panel-portal) {
    position: fixed;
    background: var(--bg-dialog);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow:
      0 16px 48px rgba(0, 0, 0, 0.22),
      0 2px 8px rgba(0, 0, 0, 0.12);
    padding: 10px;
    z-index: 99999;
  }

  .section-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-faint);
    padding: 4px 4px 8px;
    margin: 0;
  }

  .divider {
    height: 1px;
    background: var(--border);
    margin: 8px 0;
  }

  /* Theme chips */
  .theme-row {
    display: flex;
    gap: 4px;
  }
  .theme-chip {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 8px 4px 7px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 11px;
    font-weight: 500;
    transition: all 0.15s ease;
    position: relative;
  }
  .theme-chip:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: rgba(99, 102, 241, 0.35);
  }
  .theme-chip.active {
    background: rgba(99, 102, 241, 0.12);
    border-color: rgba(99, 102, 241, 0.5);
    color: #6366f1;
  }
  .chip-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
  }
  .chip-icon :global(svg) {
    width: 16px;
    height: 16px;
  }
  .chip-label {
    font-size: 10px;
  }
  .chip-dot {
    position: absolute;
    bottom: 5px;
    right: 5px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #6366f1;
  }

  /* Font stepper */
  .font-stepper {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0 10px;
  }
  .step-btn {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg-panel);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    flex-shrink: 0;
    padding: 0;
  }
  .step-btn svg {
    width: 12px;
    height: 12px;
    pointer-events: none;
  }
  .step-btn:hover:not(:disabled) {
    background: rgba(99, 102, 241, 0.12);
    border-color: rgba(99, 102, 241, 0.4);
    color: #6366f1;
  }
  .step-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .font-label-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .font-sample {
    font-family: "Inter", sans-serif;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
    transition:
      font-size 0.2s ease,
      color 0.25s ease;
  }
  .font-size-name {
    font-size: 10px;
    color: var(--text-faint);
  }

  /* Segmented picker */
  .font-segments {
    display: flex;
    gap: 3px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 4px;
  }
  .seg-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 4px;
    border-radius: 7px;
    border: none;
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
    min-height: 32px;
  }
  .seg-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .seg-btn.active {
    background: rgba(99, 102, 241, 0.15);
    color: #6366f1;
    box-shadow: 0 1px 4px rgba(99, 102, 241, 0.15);
  }

  /* About */
  .about-section {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 4px 4px;
  }
  .about-logo {
    width: 36px;
    height: 36px;
    object-fit: contain;
  }
  .about-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .about-header {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  .about-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
  }
  .about-version {
    font-size: 10px;
    color: var(--text-muted);
  }
  .about-detail {
    font-size: 10px;
    color: var(--text-faint);
  }
</style>
