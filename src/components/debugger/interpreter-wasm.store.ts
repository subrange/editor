// WebAssembly-based interpreter store that maintains API compatibility with the original

import { BehaviorSubject, Subscription } from "rxjs";
import { type Line, type Position } from "../editor/editor.store.ts";
import { editorManager } from "../../services/editor-manager.service.ts";
// @ts-ignore
import init, { BrainfuckInterpreter } from "../../wasm/rust_bf.js";

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

const DEFAULT_TAPE_SIZE = 1024 * 1024;
const DEFAULT_CELL_SIZE = 256;
const DEFAULT_LANE_COUNT = 1;

const sizeToTape = (size: number, tapeSize: number): Uint8Array | Uint16Array | Uint32Array => {
    switch (size) {
        case 256:
            return new Uint8Array(tapeSize).fill(0);
        case 65536:
            return new Uint16Array(tapeSize).fill(0);
        case 4294967296:
            return new Uint32Array(tapeSize).fill(0);
        default:
            throw new Error(`Unsupported cell size: ${size}`);
    }
}

class WasmInterpreterStore {
    public state = new BehaviorSubject<InterpreterState>({
        tape: sizeToTape(DEFAULT_CELL_SIZE, DEFAULT_TAPE_SIZE),
        pointer: 0,
        isRunning: false,
        isPaused: false,
        isStopped: false,
        breakpoints: [],
        output: '',
        laneCount: DEFAULT_LANE_COUNT
    })

    private code: Array<Line> = [];
    private wasmInterpreter: BrainfuckInterpreter | null = null;
    private wasmInitialized = false;

    public currentChar = new BehaviorSubject<Position>({
        line: 0,
        column: 0
    })

    private runInterval: number | null = null;
    private runAnimationFrameId: number | null = null;

    public tapeSize = new BehaviorSubject<number>(DEFAULT_TAPE_SIZE);
    public cellSize = new BehaviorSubject<number>(DEFAULT_CELL_SIZE);
    public laneCount = new BehaviorSubject<number>(DEFAULT_LANE_COUNT);

    private editorSubscription: Subscription | null = null;
    
    constructor() {
        this.initializeWasm();
        
        const checkMainEditor = () => {
            const mainEditor = editorManager.getEditor('main');
            if (mainEditor) {
                if (this.editorSubscription) {
                    this.editorSubscription.unsubscribe();
                }
                
                this.editorSubscription = mainEditor.editorState.subscribe(s => {
                    if (JSON.stringify(s.lines) !== JSON.stringify(this.code)) {
                        this.reset();
                        this.code = s.lines;
                        this.updateWasmCode();
                    }
                });
            } else {
                setTimeout(checkMainEditor, 100);
            }
        };
        
        checkMainEditor();

        const storedTapeSize = localStorage.getItem('tapeSize');
        if (storedTapeSize) {
            const size = parseInt(storedTapeSize, 10);
            if (!isNaN(size) && size > 0) {
                this.tapeSize.next(size);
            }
        }

        const storedCellSize = localStorage.getItem('cellSize');
        if (storedCellSize) {
            const size = parseInt(storedCellSize, 10);
            if (!isNaN(size) && [256, 65536, 4294967296].includes(size)) {
                this.cellSize.next(size);
            }
        }

        const storedLaneCount = localStorage.getItem('brainfuck-ide-lane-count');
        if (storedLaneCount) {
            const count = parseInt(storedLaneCount, 10);
            if (!isNaN(count) && count >= 1 && count <= 10) {
                this.laneCount.next(count);
            }
        }

        this.state.next({
            tape: sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue()),
            pointer: 0,
            isRunning: false,
            isPaused: false,
            isStopped: false,
            breakpoints: [],
            output: '',
            laneCount: this.laneCount.getValue()
        });
    }

    private async initializeWasm() {
        try {
            await init();
            this.wasmInitialized = true;
            this.wasmInterpreter = new BrainfuckInterpreter(
                this.tapeSize.getValue(),
                this.cellSize.getValue()
            );
            
            if (this.code.length > 0) {
                this.updateWasmCode();
            }
            
            console.log('WebAssembly interpreter initialized successfully');
        } catch (error) {
            console.error('Failed to initialize WebAssembly interpreter:', error);
        }
    }

    private updateWasmCode() {
        if (this.wasmInterpreter && this.wasmInitialized) {
            try {
                this.wasmInterpreter.set_code(JSON.stringify(this.code));
            } catch (error) {
                console.error('Failed to update WASM code:', error);
            }
        }
    }

    private syncStateFromWasm() {
        if (!this.wasmInterpreter) return;

        try {
            const stateJson = this.wasmInterpreter.get_state();
            const wasmState = JSON.parse(stateJson);
            
            // Get current position
            const positionJson = this.wasmInterpreter.get_current_position();
            const currentPosition = JSON.parse(positionJson);
            this.currentChar.next(currentPosition);
            
            const currentState = this.state.getValue();
            
            // Get a slice of the tape around the pointer for visualization
            const viewStart = Math.max(0, wasmState.pointer - 100);
            const viewEnd = Math.min(this.tapeSize.getValue(), wasmState.pointer + 100);
            const tapeSlice = this.wasmInterpreter.get_tape_slice(viewStart, viewEnd);
            
            // Update the tape view
            const tape = currentState.tape;
            for (let i = 0; i < tapeSlice.length; i++) {
                if (viewStart + i < tape.length) {
                    tape[viewStart + i] = tapeSlice[i];
                }
            }
            
            // Don't override the running states if we're in the middle of execution
            const isExecuting = currentState.isRunning && !currentState.isPaused;
            
            this.state.next({
                ...currentState,
                tape,
                pointer: wasmState.pointer,
                isRunning: isExecuting ? currentState.isRunning : wasmState.is_running,
                isPaused: isExecuting ? currentState.isPaused : wasmState.is_paused,
                isStopped: isExecuting ? currentState.isStopped : wasmState.is_stopped,
                output: wasmState.output,
                laneCount: wasmState.lane_count
            });
        } catch (error) {
            console.error('Failed to sync state from WASM:', error);
        }
    }

    public reset() {
        if (this.wasmInterpreter) {
            this.wasmInterpreter.reset();
            this.syncStateFromWasm();
        }
        
        this.currentChar.next({
            line: 0,
            column: 0
        });
        
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;
        }

        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }
    }

    public step(): boolean {
        if (!this.wasmInterpreter) {
            console.error('WASM interpreter not initialized');
            return false;
        }

        const hasMore = this.wasmInterpreter.step();
        
        if (!hasMore) {
            console.log("Program finished.");
            this.stop();
            return false;
        }
        
        this.syncStateFromWasm();
        return hasMore;
    }

    public pause() {
        if (this.wasmInterpreter) {
            this.wasmInterpreter.pause();
            this.syncStateFromWasm();
        }
    }

    public resume() {
        if (this.wasmInterpreter) {
            this.wasmInterpreter.resume();
            this.syncStateFromWasm();
        }
    }

    public run(delay: number = 100) {
        if (this.runInterval) {
            clearInterval(this.runInterval);
        }

        // Set running state
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        if (this.wasmInterpreter) {
            this.wasmInterpreter.resume();
        }

        this.runInterval = window.setInterval(() => {
            const state = this.state.getValue();
            
            if (state.isPaused) {
                return;
            }

            if (!this.step()) {
                this.stop();
            }
        }, delay);
    }

    public runSmooth = () => {
        // Set running state
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        if (this.wasmInterpreter) {
            this.wasmInterpreter.resume();
        }

        const step = () => {
            const state = this.state.getValue();

            if (!state.isPaused) {
                const r = this.step();
                if (!r) {
                    this.stop();
                    return;
                }
            }

            if (state.isRunning) {
                this.runAnimationFrameId = requestAnimationFrame(step);
            }
        };

        this.runAnimationFrameId = requestAnimationFrame(step);
        this.runInterval = null;
    }

    public async runImmediately() {
        if (!this.wasmInterpreter) {
            console.error('WASM interpreter not initialized');
            return;
        }

        console.log('Running immediately with WASM interpreter...');
        
        while (true) {
            const state = this.state.getValue();

            if (state.isPaused) {
                await new Promise(resolve => {
                    const unsubscribe = this.state.subscribe(newState => {
                        if (!newState.isPaused || !newState.isRunning) {
                            unsubscribe.unsubscribe();
                            resolve(undefined);
                        }
                    });
                });

                if (!this.state.getValue().isRunning) {
                    break;
                }
            }

            if (!this.step()) {
                break;
            }
        }
    }

    public async runTurbo() {
        if (!this.wasmInterpreter) {
            console.error('WASM interpreter not initialized');
            return;
        }

        console.log('Starting WASM turbo execution...');
        
        try {
            await this.wasmInterpreter.run_turbo();
            this.syncStateFromWasm();
        } catch (error) {
            console.error('Turbo execution failed:', error);
        }
    }

    public stop() {
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;
        }

        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }

        if (this.wasmInterpreter) {
            this.wasmInterpreter.stop();
        }
        
        // Explicitly set the stopped state
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: false,
            isPaused: false,
            isStopped: true
        });
    }

    public toggleBreakpoint(position: Position) {
        if (this.wasmInterpreter) {
            this.wasmInterpreter.toggle_breakpoint(position.line, position.column);
        }

        const currentState = this.state.getValue();
        const breakpoints = [...currentState.breakpoints];

        const index = breakpoints.findIndex(bp => bp.line === position.line && bp.column === position.column);
        if (index !== -1) {
            breakpoints.splice(index, 1);
        } else {
            breakpoints.push(position);
        }

        this.state.next({
            ...currentState,
            breakpoints
        });
    }

    public clearBreakpoints() {
        if (this.wasmInterpreter) {
            this.wasmInterpreter.clear_breakpoints();
        }

        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            breakpoints: []
        });
    }

    public hasBreakpointAt(position: Position): boolean {
        const currentState = this.state.getValue();
        return currentState.breakpoints.some(
            bp => bp.line === position.line && bp.column === position.column
        );
    }

    public setTapeSize(size: number) {
        if (size <= 0) {
            throw new Error("Tape size must be a positive integer");
        }
        
        this.tapeSize.next(size);
        localStorage.setItem('tapeSize', size.toString());
        
        if (this.wasmInterpreter) {
            this.wasmInterpreter.set_tape_size(size);
            this.reset();
        }
        
        console.log(`Tape size set to ${size} bytes`);
    }

    public setCellSize(size: number) {
        if (![256, 65536, 4294967296].includes(size)) {
            throw new Error("Unsupported cell size. Use 256, 65536, or 4294967296.");
        }
        
        this.cellSize.next(size);
        localStorage.setItem('cellSize', size.toString());
        
        if (this.wasmInterpreter) {
            this.wasmInterpreter.set_cell_size(size);
            this.reset();
        }
    }

    public setLaneCount(count: number) {
        if (count < 1 || count > 10) {
            throw new Error("Lane count must be between 1 and 10");
        }
        
        this.laneCount.next(count);
        localStorage.setItem('brainfuck-ide-lane-count', count.toString());
        
        if (this.wasmInterpreter) {
            this.wasmInterpreter.set_lane_count(count);
        }
        
        this.state.next({
            ...this.state.getValue(),
            laneCount: count
        });
    }
}

export const wasmInterpreterStore = new WasmInterpreterStore();