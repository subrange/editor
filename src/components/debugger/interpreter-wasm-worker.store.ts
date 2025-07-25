// WebAssembly-based interpreter store that runs in a Web Worker

import { BehaviorSubject, Subscription } from "rxjs";
import { type Line, type Position } from "../editor/editor.store.ts";
import { editorManager } from "../../services/editor-manager.service.ts";

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

class WasmWorkerInterpreterStore {
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
    private worker: Worker | null = null;
    private workerReady = false;
    private pendingMessages: Array<{ type: string; data?: any }> = [];

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
        this.initializeWorker();
        
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

    private async initializeWorker() {
        try {
            this.worker = new Worker(
                new URL('../../workers/brainfuck.worker.ts', import.meta.url),
                { type: 'module' }
            );

            this.worker.onmessage = (event) => {
                this.handleWorkerMessage(event.data);
            };

            this.worker.onerror = (error) => {
                console.error('Worker error:', error);
            };

            // Initialize the interpreter in the worker
            this.sendMessage('init', {
                tapeSize: this.tapeSize.getValue(),
                cellSize: this.cellSize.getValue()
            });

            console.log('WebAssembly worker initialized');
        } catch (error) {
            console.error('Failed to initialize WebAssembly worker:', error);
        }
    }

    private handleWorkerMessage(message: any) {
        const { type, data, error } = message;

        if (error) {
            console.error('Worker error:', error);
            return;
        }

        switch (type) {
            case 'initialized':
                this.workerReady = true;
                console.log('WebAssembly interpreter ready in worker');
                // Process any pending messages
                while (this.pendingMessages.length > 0) {
                    const msg = this.pendingMessages.shift()!;
                    this.sendMessage(msg.type, msg.data);
                }
                if (this.code.length > 0) {
                    this.updateWasmCode();
                }
                break;

            case 'stepped':
                if (data) {
                    this.updateStateFromWorker(data.state);
                    if (!data.hasMore) {
                        this.stop();
                    }
                }
                break;

            case 'state':
                if (data) {
                    this.updateStateFromWorker(JSON.stringify(data));
                }
                break;

            case 'turboProgress':
                if (data?.state) {
                    this.updateStateFromWorker(data.state);
                    console.log(`Turbo progress: ${data.iterations} operations`);
                }
                break;

            case 'turboComplete':
                if (data?.state) {
                    this.updateStateFromWorker(data.state);
                }
                this.stop();
                console.log('Turbo execution completed');
                break;

            case 'tapeSlice':
                if (data) {
                    this.updateTapeSlice(data);
                }
                break;

            default:
                // Handle other message types
                break;
        }
    }

    private updateStateFromWorker(stateJson: string) {
        try {
            const wasmState = JSON.parse(stateJson);
            const currentState = this.state.getValue();

            // Update current position if available
            if (wasmState.currentPosition) {
                this.currentChar.next(wasmState.currentPosition);
            }

            // Update state without tape (tape will be updated separately)
            this.state.next({
                ...currentState,
                pointer: wasmState.pointer,
                isRunning: currentState.isRunning, // Maintain local running state
                isPaused: currentState.isPaused,   // Maintain local paused state
                isStopped: currentState.isStopped, // Maintain local stopped state
                output: wasmState.output,
                laneCount: currentState.laneCount  // Keep local lane count, don't override from WASM
            });

            // Request tape slice around pointer for visualization
            // Request a larger slice to ensure visible cells are updated
            if (this.worker) {
                const viewStart = Math.max(0, wasmState.pointer - 500);
                const viewEnd = Math.min(this.tapeSize.getValue(), wasmState.pointer + 500);
                this.sendMessage('getTapeSlice', { start: viewStart, end: viewEnd });
            }
        } catch (error) {
            console.error('Failed to update state from worker:', error);
        }
    }

    private updateTapeSlice(data: any) {
        try {
            const { start, slice } = data;
            const currentState = this.state.getValue();
            const tape = currentState.tape;

            // Update the tape with the received slice
            // The slice is always u32 array from Rust, but we need to respect the current tape type
            const sliceArray = Array.isArray(slice) ? slice : Array.from(slice);
            
            for (let i = 0; i < sliceArray.length; i++) {
                if (start + i < tape.length) {
                    tape[start + i] = sliceArray[i];
                }
            }

            // Force a new state update to trigger re-render
            // Using spread to ensure React detects the change
            this.state.next({
                ...currentState,
                tape: [...tape] as typeof tape
            });
        } catch (error) {
            console.error('Failed to update tape slice:', error);
        }
    }

    private sendMessage(type: string, data?: any) {
        if (!this.worker) {
            console.error('Worker not initialized');
            return;
        }

        if (!this.workerReady && type !== 'init') {
            // Queue messages until worker is ready
            this.pendingMessages.push({ type, data });
            return;
        }

        this.worker.postMessage({ type, data });
    }

    private updateWasmCode() {
        if (this.worker && this.workerReady) {
            this.sendMessage('setCode', { code: this.code });
        }
    }

    public reset() {
        this.sendMessage('reset');
        
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

        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            tape: sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue()),
            pointer: 0,
            isRunning: false,
            isPaused: false,
            isStopped: false,
            output: ''
        });
    }

    public step(): boolean {
        this.sendMessage('step');
        // We can't return a synchronous result from an async worker
        // The component will need to handle this differently
        return true; 
    }

    public pause() {
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isPaused: true
        });
        this.sendMessage('pause');
    }

    public resume() {
        const currentState = this.state.getValue();
        if (currentState.isRunning && currentState.isPaused) {
            this.state.next({
                ...currentState,
                isPaused: false
            });
            this.sendMessage('resume');
        }
    }

    public run(delay: number = 100) {
        if (this.runInterval) {
            clearInterval(this.runInterval);
        }

        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        this.sendMessage('resume');

        this.runInterval = window.setInterval(() => {
            const state = this.state.getValue();
            
            if (state.isPaused || !state.isRunning) {
                return;
            }

            this.sendMessage('step');
        }, delay);
    }

    public runSmooth = () => {
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        this.sendMessage('resume');

        const step = () => {
            const state = this.state.getValue();

            if (!state.isPaused && state.isRunning) {
                this.sendMessage('step');
            }

            if (state.isRunning) {
                this.runAnimationFrameId = requestAnimationFrame(step);
            }
        };

        this.runAnimationFrameId = requestAnimationFrame(step);
        this.runInterval = null;
    }

    public async runImmediately() {
        console.log('Running immediately with WASM worker...');
        
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        // For immediate execution, we'll use a rapid interval
        const rapidStep = () => {
            const state = this.state.getValue();
            if (!state.isRunning || state.isPaused) {
                return;
            }
            
            this.sendMessage('step');
            setTimeout(rapidStep, 0);
        };
        
        rapidStep();
    }

    public async runTurbo() {
        console.log('Starting WASM turbo execution in worker...');
        
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        this.sendMessage('runTurbo');
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

        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: false,
            isPaused: false,
            isStopped: true
        });

        this.sendMessage('stop');
    }

    public toggleBreakpoint(position: Position) {
        this.sendMessage('toggleBreakpoint', position);

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
        this.sendMessage('clearBreakpoints');

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
        
        this.sendMessage('setTapeSize', { size });
        this.reset();
        
        console.log(`Tape size set to ${size} bytes`);
    }

    public setCellSize(size: number) {
        if (![256, 65536, 4294967296].includes(size)) {
            throw new Error("Unsupported cell size. Use 256, 65536, or 4294967296.");
        }
        
        this.cellSize.next(size);
        localStorage.setItem('cellSize', size.toString());
        
        this.sendMessage('setCellSize', { size });
        this.reset();
    }

    public setLaneCount(count: number) {
        if (count < 1 || count > 10) {
            throw new Error("Lane count must be between 1 and 10");
        }
        
        this.laneCount.next(count);
        localStorage.setItem('brainfuck-ide-lane-count', count.toString());
        
        this.sendMessage('setLaneCount', { count });
        
        this.state.next({
            ...this.state.getValue(),
            laneCount: count
        });
    }

    public destroy() {
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
    }
}

export const wasmInterpreterStore = new WasmWorkerInterpreterStore();