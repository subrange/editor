import { BehaviorSubject } from 'rxjs';

export interface MacroSettings {
  stripComments: boolean;
  collapseEmptyLines: boolean;
  autoExpand: boolean;
  useWasmExpander: boolean;
}

export type DebuggerViewMode = 'normal' | 'compact' | 'lane';

export interface DebuggerSettings {
  compactView: boolean; // Keep for backwards compatibility
  viewMode: DebuggerViewMode;
  showDisassembly: boolean;
}

export interface AssemblySettings {
  autoCompile: boolean;
  autoOpenOutput: boolean;
  bankSize: number;
  maxImmediate: number;
  memoryOffset: number; // Offset for data addresses (default 2 for VM special values)
  showWorkspace: boolean; // Show Assembly workspace tab
}

export interface InterpreterSettings {
  wrapCells: boolean;
  wrapTape: boolean;
}

export interface DevelopmentSettings {
  showDevTools: boolean;
}

export interface WeirdSettings {
  doublePlus: boolean;
}

export interface Settings {
  macro: MacroSettings;
  debugger: DebuggerSettings;
  assembly: AssemblySettings;
  interpreter?: InterpreterSettings;
  development?: DevelopmentSettings;
  weird?: WeirdSettings;
}

class SettingsStore {
  public settings = new BehaviorSubject<Settings>({
    macro: {
      stripComments: this.loadFromStorage('macroStripComments', true),
      collapseEmptyLines: this.loadFromStorage(
        'macroCollapseEmptyLines',
        false,
      ),
      autoExpand: this.loadFromStorage('autoExpandMacros', true),
      useWasmExpander: this.loadFromStorage('macroUseWasmExpander', true),
    },
    debugger: {
      compactView: this.loadFromStorage('debuggerCompactView', false),
      viewMode: this.loadFromStorage(
        'debuggerViewMode',
        'normal',
      ) as DebuggerViewMode,
      showDisassembly: this.loadFromStorage('debuggerShowDisassembly', false),
    },
    assembly: {
      autoCompile: this.loadFromStorage('assemblyAutoCompile', false),
      autoOpenOutput: this.loadFromStorage('assemblyAutoOpenOutput', true),
      bankSize: this.loadFromStorage('assemblyBankSize', 64000),
      maxImmediate: this.loadFromStorage('assemblyMaxImmediate', 65535),
      memoryOffset: this.loadFromStorage('assemblyMemoryOffset', 2),
      showWorkspace: this.loadFromStorage('assemblyShowWorkspace', false),
    },
    interpreter: {
      wrapCells: this.loadFromStorage('interpreterWrapCells', true),
      wrapTape: this.loadFromStorage('interpreterWrapTape', true),
    },
    development: {
      showDevTools: this.loadFromStorage('developmentShowDevTools', false),
    },
    weird: {
      doublePlus: this.loadFromStorage('weirdDoublePlus', false),
    },
  });

  setMacroStripComments(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      macro: {
        ...current.macro,
        stripComments: value,
      },
    });
    this.saveToStorage('macroStripComments', value);
  }

  setMacroCollapseEmptyLines(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      macro: {
        ...current.macro,
        collapseEmptyLines: value,
      },
    });
    this.saveToStorage('macroCollapseEmptyLines', value);
  }

  setMacroAutoExpand(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      macro: {
        ...current.macro,
        autoExpand: value,
      },
    });
    this.saveToStorage('autoExpandMacros', value);
  }

  setMacroUseWasmExpander(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      macro: {
        ...current.macro,
        useWasmExpander: value,
      },
    });
    this.saveToStorage('macroUseWasmExpander', value);
  }

  setDebuggerCompactView(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      debugger: {
        ...current.debugger,
        compactView: value,
      },
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
        compactView: value === 'compact',
      },
    });
    this.saveToStorage('debuggerViewMode', value);
    this.saveToStorage('debuggerCompactView', value === 'compact');
  }

  setDebuggerShowDisassembly(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      debugger: {
        ...current.debugger,
        showDisassembly: value,
      },
    });
    this.saveToStorage('debuggerShowDisassembly', value);
  }

  setAssemblyAutoCompile(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        autoCompile: value,
      },
    });
    this.saveToStorage('assemblyAutoCompile', value);
  }

  setAssemblyAutoOpenOutput(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        autoOpenOutput: value,
      },
    });
    this.saveToStorage('assemblyAutoOpenOutput', value);
  }

  setAssemblyBankSize(value: number) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        bankSize: value,
      },
    });
    this.saveToStorage('assemblyBankSize', value);
  }

  setAssemblyMaxImmediate(value: number) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        maxImmediate: value,
      },
    });
    this.saveToStorage('assemblyMaxImmediate', value);
  }

  setAssemblyMemoryOffset(value: number) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        memoryOffset: value,
      },
    });
    this.saveToStorage('assemblyMemoryOffset', value);
  }

  setAssemblyShowWorkspace(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      assembly: {
        ...current.assembly,
        showWorkspace: value,
      },
    });
    this.saveToStorage('assemblyShowWorkspace', value);
  }

  setInterpreterWrapCells(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      interpreter: {
        ...current.interpreter!,
        wrapCells: value,
      },
    });
    this.saveToStorage('interpreterWrapCells', value);
  }

  setInterpreterWrapTape(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      interpreter: {
        ...current.interpreter!,
        wrapTape: value,
      },
    });
    this.saveToStorage('interpreterWrapTape', value);
  }

  setDevelopmentShowDevTools(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      development: {
        ...current.development!,
        showDevTools: value,
      },
    });
    this.saveToStorage('developmentShowDevTools', value);
  }

  setWeirdDoublePlus(value: boolean) {
    const current = this.settings.value;
    this.settings.next({
      ...current,
      weird: {
        ...current.weird!,
        doublePlus: value,
      },
    });
    this.saveToStorage('weirdDoublePlus', value);
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
