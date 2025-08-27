// Facade that provides a unified interface to either JS or WASM interpreter
import {BehaviorSubject, Subscription} from "rxjs";
import {type Position} from "../editor/stores/editor.store.ts";
import {interpreterStore as jsInterpreter, type TapeSnapshot} from "./interpreter.store.ts";
import type {SourceMap} from "../../services/macro-expander/source-map.ts";
import {InterpreterWorkerStore} from "../../services/interpreter/interpreter-worker-store.ts";

type InterpreterState = {
    tape: Uint8Array | Uint16Array | Uint32Array;
    pointer: number;
    isRunning: boolean;
    isPaused: boolean;
    isStopped: boolean;
    isWaitingForInput?: boolean;
    breakpoints: Position[];
    sourceBreakpoints?: Position[]; // Breakpoints set in source (macro) code
    output: string;
    laneCount: number;
    
    // Source map support
    sourceMap?: SourceMap;
    currentSourcePosition?: Position;
    macroContext?: Array<{
        macroName: string;
        parameters?: Record<string, string>;
    }>;
    
    // Execution mode tracking
    lastExecutionMode?: 'normal' | 'turbo';
    
    // Execution metrics
    lastExecutionTime?: number; // Time in seconds
    lastOperationCount?: number; // Number of operations executed
}

type InterpreterInterface = {
    state: BehaviorSubject<InterpreterState>;
    currentChar: BehaviorSubject<Position>;
    currentSourceChar?: BehaviorSubject<Position | null>;
    tapeSize: BehaviorSubject<number>;
    cellSize: BehaviorSubject<number>;
    laneCount: BehaviorSubject<number>;

    reset(): void;
    step(): boolean;
    run(delay?: number): void;
    runSmooth(): void;
    runFromPosition(position: Position): void;
    stepToPosition(position: Position): void;
    runImmediately(): Promise<void>;
    runTurbo(): Promise<void>;
    resumeTurbo?(): Promise<void>;
    runUltraFast?(): Promise<void>;
    pause(): void;
    resume(): void;
    stop(): void;

    toggleBreakpoint(position: Position): void;
    toggleSourceBreakpoint?(position: Position): void;
    clearBreakpoints(): void;
    hasBreakpointAt(position: Position): boolean;
    hasSourceBreakpointAt?(position: Position): boolean;

    setTapeSize(size: number): void;
    setCellSize(size: number): void;
    setLaneCount(count: number): void;
    setSourceMap?(sourceMap: SourceMap | undefined): void;

    loadSnapshot(snapshot: TapeSnapshot): void;
}

class InterpreterFacade implements InterpreterInterface {
    private currentInterpreter: InterpreterInterface = jsInterpreter;
    private workerInterpreter: InterpreterWorkerStore | null = null;
    private subscriptions: Subscription[] = [];
    private isUsingWorker = false;
    private pendingVMOutputCallback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null = null;
    private pendingVMOutputConfig: { outCellIndex: number; outFlagCellIndex: number; sparseCellPattern?: any } | null = null;

    // Proxy all the observables
    public state = new BehaviorSubject<InterpreterState>({
        ...jsInterpreter.state.getValue(),
        sourceBreakpoints: jsInterpreter.state.getValue().sourceBreakpoints || []
    });
    public currentChar = new BehaviorSubject<Position>(jsInterpreter.currentChar.getValue());
    public currentSourceChar = new BehaviorSubject<Position | null>(jsInterpreter.currentSourceChar?.getValue() || null);
    public tapeSize = new BehaviorSubject<number>(jsInterpreter.tapeSize.getValue());
    public cellSize = new BehaviorSubject<number>(jsInterpreter.cellSize.getValue());
    public laneCount = new BehaviorSubject<number>(jsInterpreter.laneCount.getValue());

    constructor() {
        this.setupProxying();
    }

    private setupProxying() {
        // Clear existing subscriptions
        this.subscriptions.forEach(sub => sub.unsubscribe());
        this.subscriptions = [];

        // Proxy observables from current interpreter
        this.subscriptions.push(
            this.currentInterpreter.state.subscribe(value => {
                console.log('Facade proxying state:', {
                    isRunning: value.isRunning,
                    isPaused: value.isPaused,
                    isStopped: value.isStopped,
                    lastExecutionTime: value.lastExecutionTime,
                    lastOperationCount: value.lastOperationCount
                });
                this.state.next(value);
            }),
            this.currentInterpreter.currentChar.subscribe(value => this.currentChar.next(value)),
            this.currentInterpreter.tapeSize.subscribe(value => this.tapeSize.next(value)),
            this.currentInterpreter.cellSize.subscribe(value => this.cellSize.next(value)),
            this.currentInterpreter.laneCount.subscribe(value => this.laneCount.next(value))
        );
        
        // Proxy currentSourceChar if it exists
        if (this.currentInterpreter.currentSourceChar) {
            this.subscriptions.push(
                this.currentInterpreter.currentSourceChar.subscribe(value => this.currentSourceChar.next(value))
            );
        }
    }

    private async switchToWorker() {
        if (this.isUsingWorker && this.workerInterpreter) {
            return;
        }

        console.log('Switching to worker-based interpreter for turbo mode');
        
        // Create worker interpreter if not exists
        if (!this.workerInterpreter) {
            this.workerInterpreter = new InterpreterWorkerStore();
        }

        // Get current state from JS interpreter
        const currentState = jsInterpreter.state.getValue();
        const currentCode = jsInterpreter.getCode();
        
        // Initialize worker with current code (may be empty on initial load)
        this.workerInterpreter.setCode(currentCode);
        
        // Copy breakpoints
        if (currentState.breakpoints.length > 0 || (currentState.sourceBreakpoints?.length || 0) > 0) {
            // Use the proper method to set breakpoints
            currentState.breakpoints.forEach(bp => {
                this.workerInterpreter.toggleBreakpoint(bp);
            });
            if (currentState.sourceBreakpoints) {
                currentState.sourceBreakpoints.forEach(bp => {
                    this.workerInterpreter.toggleSourceBreakpoint(bp);
                });
            }
        }

        // Copy source map if exists
        if (currentState.sourceMap) {
            this.workerInterpreter.setSourceMap(currentState.sourceMap);
        }

        // Switch current interpreter
        this.currentInterpreter = this.workerInterpreter as any;
        this.isUsingWorker = true;
        this.setupProxying();
        
        // Apply pending VM output config and callback if any
        if (this.pendingVMOutputConfig && 'setVMOutputConfig' in this.workerInterpreter) {
            (this.workerInterpreter as any).setVMOutputConfig(this.pendingVMOutputConfig);
        }
        
        if (this.pendingVMOutputCallback && 'setVMOutputCallback' in this.workerInterpreter) {
            (this.workerInterpreter as any).setVMOutputCallback(this.pendingVMOutputCallback);
        }
    }

    private switchToJS() {
        if (!this.isUsingWorker) {
            return;
        }

        console.log('Switching back to JS interpreter');
        
        this.currentInterpreter = jsInterpreter;
        this.isUsingWorker = false;
        this.setupProxying();
    }


    // Delegate all methods to current interpreter
    reset() {
        this.currentInterpreter.reset();
    }

    step() {
        return this.currentInterpreter.step();
    }

    run(delay?: number) {
        // Switch back to JS for normal run
        this.switchToJS();
        this.currentInterpreter.run(delay);
    }

    runSmooth() {
        // Switch back to JS for smooth run
        this.switchToJS();
        this.currentInterpreter.runSmooth();
    }

    runFromPosition(position: Position) {
        this.currentInterpreter.runFromPosition(position);
    }

    stepToPosition(position: Position) {
        this.currentInterpreter.stepToPosition(position);
    }

    async runImmediately() {
        await this.currentInterpreter.runImmediately();
    }

    async runTurbo() {
        // Switch to worker for turbo mode
        await this.switchToWorker();
        await this.currentInterpreter.runTurbo();
    }

    async resumeTurbo() {
        // If we're in turbo mode, use worker
        if (this.state.getValue().lastExecutionMode === 'turbo') {
            await this.switchToWorker();
        }
        
        if (this.currentInterpreter.resumeTurbo) {
            await this.currentInterpreter.resumeTurbo();
        }
    }

    async runUltraFast() {
        if ('runUltraFast' in this.currentInterpreter) {
            await (this.currentInterpreter as any).runUltraFast();
        }
    }

    pause() {
        this.currentInterpreter.pause();
    }

    resume() {
        this.currentInterpreter.resume();
    }

    stop() {
        this.currentInterpreter.stop();
    }

    toggleBreakpoint(position: Position) {
        this.currentInterpreter.toggleBreakpoint(position);
    }

    clearBreakpoints() {
        this.currentInterpreter.clearBreakpoints();
    }

    hasBreakpointAt(position: Position) {
        return this.currentInterpreter.hasBreakpointAt(position);
    }

    setTapeSize(size: number) {
        this.currentInterpreter.setTapeSize(size);
    }

    setCellSize(size: number) {
        this.currentInterpreter.setCellSize(size);
    }

    setLaneCount(count: number) {
        this.currentInterpreter.setLaneCount(count);
    }

    loadSnapshot(snapshot: TapeSnapshot) {
        this.currentInterpreter.loadSnapshot(snapshot);
    }
    
    provideInput(char: string) {
        // Delegate to the current interpreter
        if ('provideInput' in this.currentInterpreter) {
            (this.currentInterpreter as any).provideInput(char);
        }
    }
    
    toggleSourceBreakpoint(position: Position) {
        if (this.currentInterpreter.toggleSourceBreakpoint) {
            this.currentInterpreter.toggleSourceBreakpoint(position);
        }
    }
    
    hasSourceBreakpointAt(position: Position) {
        if (this.currentInterpreter.hasSourceBreakpointAt) {
            return this.currentInterpreter.hasSourceBreakpointAt(position);
        }
        return false;
    }
    
    setSourceMap(sourceMap: SourceMap | undefined) {
        if (this.currentInterpreter.setSourceMap) {
            this.currentInterpreter.setSourceMap(sourceMap);
        }
    }
    
    async setVMOutputCallback(callback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null) {
        // Store the callback to be set when we switch to worker
        this.pendingVMOutputCallback = callback;
        
        // If we're already using the worker, set it immediately
        if (this.isUsingWorker && this.workerInterpreter) {
            if ('setVMOutputCallback' in this.workerInterpreter) {
                (this.workerInterpreter as any).setVMOutputCallback(callback);
            }
        }
        // Otherwise, the callback will be set when we switch to worker for execution
    }
    
    async setVMOutputConfig(config: { outCellIndex: number; outFlagCellIndex: number; clearOnRead?: boolean; sparseCellPattern?: any }) {
        // Store the config to be set when we switch to worker
        this.pendingVMOutputConfig = config;
        
        // If we're already using the worker, set it immediately
        if (this.isUsingWorker && this.workerInterpreter) {
            if ('setVMOutputConfig' in this.workerInterpreter) {
                (this.workerInterpreter as any).setVMOutputConfig(config);
            }
        }
        // Otherwise, the config will be set when we switch to worker for execution
    }
}

// Export a single instance that all components can use
export const interpreterStore = new InterpreterFacade();