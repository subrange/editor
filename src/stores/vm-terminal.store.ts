import { BehaviorSubject } from "rxjs";

export interface VMTerminalConfig {
    outCellIndex: number;
    outFlagCellIndex: number;
    clearOnRead: boolean;
    enabled: boolean;
}

export interface VMTerminalState {
    config: VMTerminalConfig;
    output: string;
}

const DEFAULT_CONFIG: VMTerminalConfig = {
    outCellIndex: 4,
    outFlagCellIndex: 12,
    clearOnRead: true,
    enabled: true
};

const loadConfigFromStorage = (): VMTerminalConfig => {
    try {
        const stored = localStorage.getItem('vm-terminal-config');
        if (stored) {
            return { ...DEFAULT_CONFIG, ...JSON.parse(stored) };
        }
    } catch (e) {
        console.error('Failed to load VM terminal config from localStorage:', e);
    }
    return DEFAULT_CONFIG;
};

const initialState: VMTerminalState = {
    config: loadConfigFromStorage(),
    output: ""
};

class VMTerminalStore {
    private state$ = new BehaviorSubject<VMTerminalState>(initialState);
    
    get state() {
        return this.state$;
    }
    
    updateConfig(config: Partial<VMTerminalConfig>) {
        const newConfig = {
            ...this.state$.value.config,
            ...config
        };
        
        // Save to localStorage
        try {
            localStorage.setItem('vm-terminal-config', JSON.stringify(newConfig));
        } catch (e) {
            console.error('Failed to save VM terminal config to localStorage:', e);
        }
        
        this.state$.next({
            ...this.state$.value,
            config: newConfig
        });
    }
    
    resetConfig() {
        this.updateConfig(DEFAULT_CONFIG);
    }
    
    appendOutput(char: string) {
        this.state$.next({
            ...this.state$.value,
            output: this.state$.value.output + char
        });
    }
    
    clearOutput() {
        this.state$.next({
            ...this.state$.value,
            output: ""
        });
    }
}

export const vmTerminalStore = new VMTerminalStore();