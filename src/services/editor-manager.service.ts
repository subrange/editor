import { BehaviorSubject } from 'rxjs';
import {
  EditorStore,
  type IEditorSettings,
} from '../components/editor/stores/editor.store.ts';
import { Tokenizer } from '../components/editor/services/tokenizer.ts';

export interface ITokenizer {
  reset(): void;
  tokenizeLine(text: string, lineIndex: number, isLastLine?: boolean): any[];
  tokenizeAllLines(lines: string[]): any[][];
}

export interface EditorConfig {
  id: string;
  tokenizer?: ITokenizer;
  settings?: IEditorSettings;
  initialContent?: string;
  mode?: 'normal' | 'insert' | 'command';
}

class EditorManager {
  private editors: Map<string, EditorStore> = new Map();
  private activeEditor$ = new BehaviorSubject<string | null>(null);

  // Observable for active editor changes
  public get activeEditorId$() {
    return this.activeEditor$.asObservable();
  }

  // Get current active editor ID
  public get activeEditorId() {
    return this.activeEditor$.getValue();
  }

  // Get active editor store
  public get activeEditor(): EditorStore | undefined {
    const activeId = this.activeEditor$.getValue();
    return activeId ? this.editors.get(activeId) : undefined;
  }

  public createEditor(config: EditorConfig): EditorStore {
    if (this.editors.has(config.id)) {
      throw new Error(`Editor with id "${config.id}" already exists`);
    }

    // Create new editor instance with config
    const editor = new EditorStore(
      config.id,
      config.tokenizer || new Tokenizer(),
      config.settings || {},
      config.initialContent,
      config.mode,
    );

    this.editors.set(config.id, editor);

    // If this is the first editor, make it active
    if (this.editors.size === 1) {
      this.setActiveEditor(config.id);
    }

    return editor;
  }

  public getEditor(id: string): EditorStore | undefined {
    return this.editors.get(id);
  }

  public setActiveEditor(id: string): void {
    if (!this.editors.has(id)) {
      console.warn(`Editor with id "${id}" does not exist`);
      return;
    }

    const previousActiveId = this.activeEditor$.getValue();

    // Blur previous active editor
    if (previousActiveId && previousActiveId !== id) {
      const previousEditor = this.editors.get(previousActiveId);
      if (previousEditor) {
        previousEditor.blur();
      }
    }

    // Set new active editor
    this.activeEditor$.next(id);

    // Focus new active editor
    const newActiveEditor = this.editors.get(id);
    if (newActiveEditor) {
      newActiveEditor.focus();
    }
  }

  public destroyEditor(id: string): void {
    const editor = this.editors.get(id);
    if (!editor) {
      console.warn(`Editor with id "${id}" does not exist`);
      return;
    }

    // Clean up the editor
    editor.destroy();

    // Remove from map
    this.editors.delete(id);

    // If this was the active editor, set a new active one
    if (this.activeEditor$.getValue() === id) {
      const remainingEditorIds = Array.from(this.editors.keys());
      if (remainingEditorIds.length > 0) {
        this.setActiveEditor(remainingEditorIds[0]);
      } else {
        this.activeEditor$.next(null);
      }
    }
  }

  public getAllEditorIds(): string[] {
    return Array.from(this.editors.keys());
  }

  public getEditorCount(): number {
    return this.editors.size;
  }

  // Clean up all editors
  public destroy(): void {
    // Destroy all editors
    this.editors.forEach((editor) => {
      editor.destroy();
    });

    this.editors.clear();
    this.activeEditor$.next(null);
  }
}

// Export singleton instance
export const editorManager = new EditorManager();
