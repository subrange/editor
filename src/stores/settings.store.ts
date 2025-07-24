import { BehaviorSubject } from 'rxjs';

export interface MacroSettings {
    stripComments: boolean;
    collapseEmptyLines: boolean;
}

export interface DebuggerSettings {
    compactView: boolean;
}

export interface Settings {
    macro: MacroSettings;
    debugger: DebuggerSettings;
}

class SettingsStore {
    public settings = new BehaviorSubject<Settings>({
        macro: {
            stripComments: this.loadFromStorage('macroStripComments', true),
            collapseEmptyLines: this.loadFromStorage('macroCollapseEmptyLines', true)
        },
        debugger: {
            compactView: this.loadFromStorage('debuggerCompactView', false)
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

    private loadFromStorage(key: string, defaultValue: boolean): boolean {
        const stored = localStorage.getItem(key);
        return stored !== null ? JSON.parse(stored) : defaultValue;
    }

    private saveToStorage(key: string, value: boolean) {
        localStorage.setItem(key, JSON.stringify(value));
    }
}

export const settingsStore = new SettingsStore();