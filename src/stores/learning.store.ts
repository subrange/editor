import { BehaviorSubject } from 'rxjs';

import { learningContent } from '../learning-content/learning-content.ts';

// RVM content
// import rvmIntro from '../learning-content/rvm/basics/intro.asm?raw';

// Types for the learning system
export interface LearningItemContent {
  mainEditor?: string; // Content for main editor
  macroEditor?: string; // Content for macro editor
  assemblyEditor?: string; // Content for assembly editor
}

export interface EditorConfig {
  showMainEditor?: boolean;
  showMacroEditor?: boolean;
  mainEditorMode?: 'brainfuck' | 'assembly';
}

export interface InterpreterConfig {
  tapeSize?: number; // Required tape size
  cellSize?: 256 | 65536 | 4294967296; // 8-bit, 16-bit, or 32-bit
}

export interface DebuggerConfig {
  viewMode?: 'normal' | 'compact' | 'lane';
  laneCount?: number; // Number of lanes for lane view
}

export interface TapeLabels {
  lanes?: { [key: number]: string }; // Lane labels (for lane view)
  columns?: { [key: number]: string }; // Column/word labels
  cells?: { [key: number]: string }; // Individual cell labels
}

export interface LearningItem {
  id: string;
  name: string;
  description: string;
  editorConfig: EditorConfig;
  interpreterConfig?: InterpreterConfig;
  debuggerConfig?: DebuggerConfig;
  labels?: TapeLabels;
  content: LearningItemContent;
}

export interface LearningSubcategory {
  id: string;
  name: string;
  items: LearningItem[];
}

export interface LearningCategory {
  id: string;
  name: string;
  icon?: string; // Optional emoji or icon identifier
  subcategories: LearningSubcategory[];
}

interface LearningState {
  categories: LearningCategory[];
  selectedItem: LearningItem | null;
}

class LearningStore {
  // Initialize with default learning content
  private defaultCategories: LearningCategory[] = learningContent;

  public state = new BehaviorSubject<LearningState>({
    categories: this.defaultCategories,
    selectedItem: null,
  });

  // Select a learning item
  selectItem(item: LearningItem | null) {
    this.state.next({
      ...this.state.value,
      selectedItem: item,
    });
  }
}

export const learningStore = new LearningStore();
