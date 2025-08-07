// Facade that provides a unified interface to either JS or WASM interpreter
import {BehaviorSubject, Subscription} from "rxjs";
import {type Position} from "../editor/stores/editor.store.ts";
import {interpreterStore as jsInterpreter, type TapeSnapshot} from "./interpreter.store.ts";
import type {SourceMap} from "../../services/macro-expander/source-map.ts";

type InterpreterState = {
    tape: Uint8Array | Uint16Array | Uint32Array;
    pointer: number;
    isRunning: boolean;
    isPaused: boolean;
    isStopped: boolean;
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
    private subscriptions: Subscription[] = [];

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
            this.currentInterpreter.state.subscribe(value => this.state.next(value)),
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


    // Delegate all methods to current interpreter
    reset() {
        this.currentInterpreter.reset();
    }

    step() {
        return this.currentInterpreter.step();
    }

    run(delay?: number) {
        this.currentInterpreter.run(delay);
    }

    runSmooth() {
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
        await this.currentInterpreter.runTurbo();
    }

    async resumeTurbo() {
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
}

// Export a single instance that all components can use
export const interpreterStore = new InterpreterFacade();