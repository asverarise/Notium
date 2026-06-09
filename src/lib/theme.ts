import { writable } from 'svelte/store';

// ── Theme ──────────────────────────────────────────────────────────────────

export type Theme = 'light' | 'dark' | 'system';

const THEME_KEY = 'notium-theme';

function getInitialTheme(): Theme {
  if (typeof localStorage === 'undefined') return 'light';
  const stored = localStorage.getItem(THEME_KEY) as Theme | null;
  if (stored === 'light' || stored === 'dark' || stored === 'system') return stored;
  return 'light';
}

function resolveTheme(t: Theme): 'light' | 'dark' {
  if (t === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return t;
}

function applyTheme(t: Theme) {
  document.documentElement.setAttribute('data-theme', resolveTheme(t));
}

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>(getInitialTheme());
  return {
    subscribe,
    set(t: Theme) {
      set(t);
      localStorage.setItem(THEME_KEY, t);
      applyTheme(t);
    },
    init() {
      const t = getInitialTheme();
      set(t);
      applyTheme(t);
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
        update((current) => { if (current === 'system') applyTheme('system'); return current; });
      });
    },
  };
}

export const theme = createThemeStore();

// ── Font Size ──────────────────────────────────────────────────────────────

export type FontSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

const FONT_KEY = 'notium-font-size';

/** Maps each size token to actual px values for the editor text */
export const fontSizeMap: Record<FontSize, { label: string; editor: string; ui: string }> = {
  xs: { label: 'XS',     editor: '12px', ui: '11px' },
  sm: { label: 'Small',  editor: '13px', ui: '12px' },
  md: { label: 'Normal', editor: '14px', ui: '13px' },
  lg: { label: 'Large',  editor: '16px', ui: '14px' },
  xl: { label: 'XL',     editor: '18px', ui: '15px' },
};

export const fontSizeOrder: FontSize[] = ['xs', 'sm', 'md', 'lg', 'xl'];

function getInitialFontSize(): FontSize {
  if (typeof localStorage === 'undefined') return 'md';
  const stored = localStorage.getItem(FONT_KEY) as FontSize | null;
  if (stored && stored in fontSizeMap) return stored;
  return 'md';
}

function applyFontSize(size: FontSize) {
  const { editor, ui } = fontSizeMap[size];
  document.documentElement.style.setProperty('--editor-font-size', editor);
  document.documentElement.style.setProperty('--ui-font-size', ui);
}

function createFontSizeStore() {
  const { subscribe, set } = writable<FontSize>(getInitialFontSize());
  return {
    subscribe,
    set(size: FontSize) {
      set(size);
      localStorage.setItem(FONT_KEY, size);
      applyFontSize(size);
    },
    init() {
      const size = getInitialFontSize();
      set(size);
      applyFontSize(size);
    },
  };
}

export const fontSize = createFontSizeStore();
