export interface Note {
  id: string;
  title: string;
  content: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
  isPinned: boolean;
  color?: string;
}

export interface AppState {
  notes: Note[];
  activeNoteId: string | null;
  searchQuery: string;
  selectedTags: string[];
  view: 'editor' | 'preview';
  sidebarWidth: number;
}
