import { BehaviorSubject } from 'rxjs';

export interface MacroSettings {
    stripComments: boolean;
    collapseEmptyLines: boolean;
    autoExpand: boolean;
}

export type DebuggerViewMode = 'normal' | 'compact' | 'lane';

export interface DebuggerSettings {
    compactView: boolean; // Keep for backwards compatibility
    viewMode: DebuggerViewMode;
    useCanvasRenderer: boolean;
}

export interface Settings {
    macro: MacroSettings;
    debugger: DebuggerSettings;
}

class SettingsStore {
    public settings = new BehaviorSubject<Settings>({
        macro: {
            stripComments: this.loadFromStorage('macroStripComments', true),
            collapseEmptyLines: this.loadFromStorage('macroCollapseEmptyLines', true),
            autoExpand: this.loadFromStorage('autoExpandMacros', false)
        },
        debugger: {
            compactView: this.loadFromStorage('debuggerCompactView', false),
            viewMode: this.loadFromStorage('debuggerViewMode', 'normal') as DebuggerViewMode,
            useCanvasRenderer: this.loadFromStorage('debuggerUseCanvasRenderer', false)
        }
    });

    setMacroStripComments(value: boolean) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            macro: {
                ...current.macro,
                stripComments: value
            }
        });
        this.saveToStorage('macroStripComments', value);
    }

    setMacroCollapseEmptyLines(value: boolean) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            macro: {
                ...current.macro,
                collapseEmptyLines: value
            }
        });
        this.saveToStorage('macroCollapseEmptyLines', value);
    }

    setMacroAutoExpand(value: boolean) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            macro: {
                ...current.macro,
                autoExpand: value
            }
        });
        this.saveToStorage('autoExpandMacros', value);
    }

    setDebuggerCompactView(value: boolean) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            debugger: {
                ...current.debugger,
                compactView: value
            }
        });
        this.saveToStorage('debuggerCompactView', value);
    }

    setDebuggerViewMode(value: DebuggerViewMode) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            debugger: {
                ...current.debugger,
                viewMode: value,
                // Update compactView for backwards compatibility
                compactView: value === 'compact'
            }
        });
        this.saveToStorage('debuggerViewMode', value);
        this.saveToStorage('debuggerCompactView', value === 'compact');
    }

    setUseCanvasRenderer(value: boolean) {
        const current = this.settings.value;
        this.settings.next({
            ...current,
            debugger: {
                ...current.debugger,
                useCanvasRenderer: value
            }
        });
        this.saveToStorage('debuggerUseCanvasRenderer', value);
    }

    private loadFromStorage<T>(key: string, defaultValue: T): T {
        const stored = localStorage.getItem(key);
        return stored !== null ? JSON.parse(stored) : defaultValue;
    }

    private saveToStorage<T>(key: string, value: T) {
        localStorage.setItem(key, JSON.stringify(value));
    }
}

export const settingsStore = new SettingsStore();