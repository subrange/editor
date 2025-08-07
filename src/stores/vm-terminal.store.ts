import { BehaviorSubject } from "rxjs";

export interface VMTerminalConfig {
    outCellIndex: number;
    outFlagCellIndex: number;
    clearOnRead: boolean;
}

export interface VMTerminalState {
    config: VMTerminalConfig;
    output: string;
}

const initialState: VMTerminalState = {
    config: {
        outCellIndex: 4,
        outFlagCellIndex: 12,
        clearOnRead: true
    },
    output: ""
};

class VMTerminalStore {
    private state$ = new BehaviorSubject<VMTerminalState>(initialState);
    
    get state() {
        return this.state$;
    }
    
    updateConfig(config: Partial<VMTerminalConfig>) {
        this.state$.next({
            ...this.state$.value,
            config: {
                ...this.state$.value.config,
                ...config
            }
        });
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