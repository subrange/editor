// Facade that provides a unified interface to either JS or WASM interpreter
import { BehaviorSubject, Subscription } from "rxjs";
import { type Line, type Position } from "../editor/editor.store.ts";
import { interpreterStore as jsInterpreter } from "./interpreter.store.ts";
import { useWasmInterpreter } from "./use-wasm-interpreter.ts";

type InterpreterState = {
    tape: Uint8Array | Uint16Array | Uint32Array;
    pointer: number;
    isRunning: boolean;
    isPaused: boolean;
    isStopped: boolean;
    breakpoints: Position[];
    output: string;
    laneCount: number;
}

type InterpreterInterface = {
    state: BehaviorSubject<InterpreterState>;
    currentChar: BehaviorSubject<Position>;
    tapeSize: BehaviorSubject<number>;
    cellSize: BehaviorSubject<number>;
    laneCount: BehaviorSubject<number>;
    
    reset(): void;
    step(): boolean;
    run(delay?: number): void;
    runSmooth(): void;
    runImmediately(): Promise<void>;
    runTurbo(): Promise<void>;
    runUltraFast?(): Promise<void>;
    pause(): void;
    resume(): void;
    stop(): void;
    
    toggleBreakpoint(position: Position): void;
    clearBreakpoints(): void;
    hasBreakpointAt(position: Position): boolean;
    
    setTapeSize(size: number): void;
    setCellSize(size: number): void;
    setLaneCount(count: number): void;
}

class InterpreterFacade implements InterpreterInterface {
    private currentInterpreter: InterpreterInterface = jsInterpreter;
    private wasmInterpreter: InterpreterInterface | null = null;
    private subscriptions: Subscription[] = [];
    
    // Proxy all the observables
    public state = new BehaviorSubject<InterpreterState>(jsInterpreter.state.getValue());
    public currentChar = new BehaviorSubject<Position>(jsInterpreter.currentChar.getValue());
    public tapeSize = new BehaviorSubject<number>(jsInterpreter.tapeSize.getValue());
    public cellSize = new BehaviorSubject<number>(jsInterpreter.cellSize.getValue());
    public laneCount = new BehaviorSubject<number>(jsInterpreter.laneCount.getValue());
    
    constructor() {
        this.setupProxying();
        
        // Listen for WASM toggle changes
        useWasmInterpreter.subscribe(async (useWasm) => {
            if (useWasm) {
                await this.switchToWasm();
            } else {
                this.switchToJs();
            }
        });
        
        // Try to load WASM if it's enabled
        if (useWasmInterpreter.getValue()) {
            this.switchToWasm();
        }
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
    }
    
    private async switchToWasm() {
        try {
            if (!this.wasmInterpreter) {
                const { wasmInterpreterStore } = await import("./interpreter-wasm.store.ts");
                this.wasmInterpreter = wasmInterpreterStore;
            }
            
            this.currentInterpreter = this.wasmInterpreter;
            this.setupProxying();
            console.log('Switched to WebAssembly interpreter');
        } catch (error) {
            console.error('Failed to load WASM interpreter:', error);
            useWasmInterpreter.next(false);
        }
    }
    
    private switchToJs() {
        this.currentInterpreter = jsInterpreter;
        this.setupProxying();
        console.log('Switched to JavaScript interpreter');
    }
    
    // Delegate all methods to current interpreter
    reset() { this.currentInterpreter.reset(); }
    step() { return this.currentInterpreter.step(); }
    run(delay?: number) { this.currentInterpreter.run(delay); }
    runSmooth() { this.currentInterpreter.runSmooth(); }
    async runImmediately() { await this.currentInterpreter.runImmediately(); }
    async runTurbo() { await this.currentInterpreter.runTurbo(); }
    async runUltraFast() { 
        if ('runUltraFast' in this.currentInterpreter) {
            await (this.currentInterpreter as any).runUltraFast();
        }
    }
    pause() { this.currentInterpreter.pause(); }
    resume() { this.currentInterpreter.resume(); }
    stop() { this.currentInterpreter.stop(); }
    
    toggleBreakpoint(position: Position) { this.currentInterpreter.toggleBreakpoint(position); }
    clearBreakpoints() { this.currentInterpreter.clearBreakpoints(); }
    hasBreakpointAt(position: Position) { return this.currentInterpreter.hasBreakpointAt(position); }
    
    setTapeSize(size: number) { this.currentInterpreter.setTapeSize(size); }
    setCellSize(size: number) { this.currentInterpreter.setCellSize(size); }
    setLaneCount(count: number) { this.currentInterpreter.setLaneCount(count); }
}

// Export a single instance that all components can use
export const interpreterStore = new InterpreterFacade();