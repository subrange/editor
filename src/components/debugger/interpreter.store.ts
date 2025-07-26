// Modified brainfuck interpreter with loop support

import {BehaviorSubject, Subscription} from "rxjs";
import {type Line, type Position} from "../editor/editor.store.ts";
import {editorManager} from "../../services/editor-manager.service.ts";

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

const DEFAULT_TAPE_SIZE = 1024 * 1024; // 1 megabyte tape
const DEFAULT_CELL_SIZE = 256; // 8-bit cells
const DEFAULT_LANE_COUNT = 1; // Single lane by default

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

class InterpreterStore {
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

    public currentChar = new BehaviorSubject<Position>({
        line: 0,
        column: 0
    })

    private loopMap: Map<string, Position> = new Map();
    private runInterval: number | null = null;
    private runAnimationFrameId: number | null = null;

    private lastPausedBreakpoint: Position | null = null;

    public tapeSize = new BehaviorSubject<number>(DEFAULT_TAPE_SIZE);
    public cellSize = new BehaviorSubject<number>(DEFAULT_CELL_SIZE);
    public laneCount = new BehaviorSubject<number>(DEFAULT_LANE_COUNT);

    private editorSubscription: Subscription | null = null;
    
    constructor() {
        // Always subscribe to the main editor, not the active editor
        // This ensures debugger always runs code from main editor
        const checkMainEditor = () => {
            const mainEditor = editorManager.getEditor('main');
            if (mainEditor) {
                // Unsubscribe from any previous subscription
                if (this.editorSubscription) {
                    this.editorSubscription.unsubscribe();
                }
                
                // Subscribe to main editor only
                this.editorSubscription = mainEditor.editorState.subscribe(s => {
                    if (JSON.stringify(s.lines) !== JSON.stringify(this.code)) { // Yep, I do not care about performance here
                        this.reset();
                        this.code = s.lines;
                        this.buildLoopMap();
                    }
                });
            } else {
                // Main editor not created yet, check again later
                setTimeout(checkMainEditor, 100);
            }
        };
        
        checkMainEditor();

        // Load tape size from local storage if available
        const storedTapeSize = localStorage.getItem('tapeSize');
        if (storedTapeSize) {
            const size = parseInt(storedTapeSize, 10);
            if (!isNaN(size) && size > 0) {
                this.tapeSize.next(size);
            }
        }

        // Load cell size from local storage if available
        const storedCellSize = localStorage.getItem('cellSize');

        if (storedCellSize) {
            const size = parseInt(storedCellSize, 10);
            if (!isNaN(size) && [256, 65536, 4294967296].includes(size)) {
                this.cellSize.next(size);
            }
        }

        // Load lane count from local storage if available
        const storedLaneCount = localStorage.getItem('brainfuck-ide-lane-count');
        if (storedLaneCount) {
            const count = parseInt(storedLaneCount, 10);
            if (!isNaN(count) && count >= 1 && count <= 10) {
                this.laneCount.next(count);
            }
        }

        // Initialize tape with the correct size
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

    public reset() {
        this.state.next({
            tape: sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue()),
            pointer: 0,
            isRunning: false,
            isPaused: false,
            isStopped: false,
            breakpoints: this.state.getValue().breakpoints, // Keep existing breakpoints
            output: '',
            laneCount: this.laneCount.getValue()
        });
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

        this.lastPausedBreakpoint = null;
    }

    public runFromPosition(position: Position) {
        // Reset the interpreter but keep breakpoints
        this.reset();
        
        // Set the current character position to start from
        this.currentChar.next(position);
        
        // Start running smoothly from this position
        this.runSmooth();
    }

    // Build a map of matching brackets for efficient jumping
    private buildLoopMap() {
        this.loopMap.clear();
        const stack: Position[] = [];

        for (let line = 0; line < this.code.length; line++) {
            const text = this.code[line].text;
            for (let column = 0; column < text.length; column++) {
                const char = text[column];
                const pos = { line, column };

                if (char === '[') {
                    stack.push(pos);
                } else if (char === ']') {
                    if (stack.length === 0) {
                        console.error(`Unmatched ] at line ${line}, column ${column}`);
                        continue;
                    }
                    const openPos = stack.pop()!;
                    // Map both directions
                    this.loopMap.set(this.posToKey(openPos), pos);
                    this.loopMap.set(this.posToKey(pos), openPos);
                }
            }
        }

        if (stack.length > 0) {
            console.error(`Unmatched [ brackets:`, stack);
        }
    }

    private posToKey(pos: Position): string {
        return `${pos.line},${pos.column}`;
    }

    private moveToNextChar() {
        const current = this.currentChar.getValue();

        if (current.column < this.code[current.line].text.length - 1) {
            // Move to next column
            this.currentChar.next({
                line: current.line,
                column: current.column + 1
            });
        } else if (current.line < this.code.length - 1) {
            // Move to next line
            this.currentChar.next({
                line: current.line + 1,
                column: 0
            });
        } else {
            // End of code
            return false;
        }
        return true;
    }

    private moveToNextLine() {
        const current = this.currentChar.getValue();

        if (current.line < this.code.length - 1) {
            // Move to next line
            this.currentChar.next({
                line: current.line + 1,
                column: 0
            });
            return true;
        } else {
            // End of code
            return false;
        }
    }

    private getCharAt(pos: Position): string | null {
        if (pos.line >= this.code.length) return null;
        const line = this.code[pos.line];
        if (pos.column >= line.text.length) return null;
        return line.text[pos.column];
    }

    private getCurrentChar(): string | null {
        return this.getCharAt(this.currentChar.getValue());
    }

    public toggleBreakpoint(position: Position) {
        const currentState = this.state.getValue();
        const breakpoints = [...currentState.breakpoints];

        const index = breakpoints.findIndex(bp => bp.line === position.line && bp.column === position.column);
        if (index !== -1) {
            // Remove breakpoint
            breakpoints.splice(index, 1);
        } else {
            // Add breakpoint
            breakpoints.push(position);
        }

        this.state.next({
            ...currentState,
            breakpoints
        });
    }

    public clearBreakpoints() {
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            breakpoints: []
        });
        this.lastPausedBreakpoint = null; // Clear last paused breakpoint as well
    }

    private shouldPauseAtBreakpoint(position: Position): boolean {
        const currentState = this.state.getValue();
        return currentState.breakpoints.some(
            bp => bp.line === position.line && bp.column === position.column
        );
    }

    public step(): boolean {
        const currentState = {
            ...this.state.getValue()
        };

        const char = this.getCurrentChar();
        const currentPos = this.currentChar.getValue();

        // Check for breakpoint BEFORE executing the instruction
        // But skip if this is the same breakpoint we just paused at
        if (char && '><+-[].,'.includes(char) && this.shouldPauseAtBreakpoint(currentPos)) {
            const isSameBreakpoint = this.lastPausedBreakpoint &&
                this.lastPausedBreakpoint.line === currentPos.line &&
                this.lastPausedBreakpoint.column === currentPos.column;

            if (!isSameBreakpoint) {
                console.log(`Hit breakpoint at line ${currentPos.line}, column ${currentPos.column}`);
                this.lastPausedBreakpoint = { ...currentPos };
                this.pause();
                return true; // Return true to indicate we're not done yet
            }
        }

        // Clear the last paused breakpoint if we've moved away from it
        if (this.lastPausedBreakpoint &&
            (this.lastPausedBreakpoint.line !== currentPos.line ||
                this.lastPausedBreakpoint.column !== currentPos.column)) {
            this.lastPausedBreakpoint = null;
        }

        if (char === '/') {
            const hasMore = this.moveToNextLine();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop();
                return false;
            }
            // Continue processing from the new line
            return this.step();
        }

        // Skip non-command characters
        if (char === null || (char && !'><+-[].,'.includes(char))) {
            const hasMore = this.moveToNextChar();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop();
                return false;
            }
            // If we just skipped a non-command character, try stepping again
            return this.step();
        }

        let shouldMoveNext = true;

        // Execute the instruction
        switch (char) {
            case '>':
                currentState.pointer = (currentState.pointer + 1) % currentState.tape.length;
                break;
            case '<':
                currentState.pointer = (currentState.pointer - 1 + currentState.tape.length) % currentState.tape.length;
                break;
            case '+':
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] + 1) % this.cellSize.getValue();
                break;
            case '-':
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] - 1 + this.cellSize.getValue()) % this.cellSize.getValue();
                break;
            case '[':
                if (currentState.tape[currentState.pointer] === 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        shouldMoveNext = true;
                    } else {
                        console.error(`No matching ] for [ at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                break;
            case ']':
                if (currentState.tape[currentState.pointer] !== 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        shouldMoveNext = true;
                    } else {
                        console.error(`No matching [ for ] at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                break;
            case '.':
                currentState.output += String.fromCharCode(currentState.tape[currentState.pointer]);
                break;
            case ',':
                console.log(`Input requested at position ${currentState.pointer}`);
                break;
        }

        this.state.next(currentState);

        if (shouldMoveNext) {
            const hasMore = this.moveToNextChar();
            if (!hasMore) {
                console.log("Program finished.");
                this.stop();
                return false;
            }
        }

        return true;
    }

    public pause() {
        // Don't clear the interval/animation frame - just set isPaused
        this.state.next({
            ...this.state.getValue(),
            isPaused: true
        });
    }

    public resume() {
        const currentState = this.state.getValue();
        if (!currentState.isRunning || !currentState.isPaused) {
            return;
        }

        this.state.next({
            ...currentState,
            isPaused: false
        });
    }

    public run(delay: number = 100) {
        if (this.runInterval) {
            clearInterval(this.runInterval);
        }

        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        this.runInterval = window.setInterval(() => {
            const state = this.state.getValue();

            // Skip if paused
            if (state.isPaused) {
                return;
            }

            if (!this.step()) {
                this.stop();
            }
        }, delay);
    }

    public runSmooth = () => {
        // Run with requestAnimationFrame for smooth execution
        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        const step = () => {
            const state = this.state.getValue();

            // Keep the animation frame going but don't step if paused
            if (!state.isPaused) {
                const r = this.step();
                if (!r) {
                    return;
                }
            }

            // Continue the animation frame even if paused
            if (state.isRunning) {
                this.runAnimationFrameId = requestAnimationFrame(step);
            }
        };

        this.runAnimationFrameId = requestAnimationFrame(step);
        this.runInterval = null;
    }

    public async runImmediately() {
        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        while (true) {
            const state = this.state.getValue();

            // Check if paused - if so, wait
            if (state.isPaused) {
                await new Promise(resolve => {
                    const unsubscribe = this.state.subscribe(newState => {
                        if (!newState.isPaused || !newState.isRunning) {
                            unsubscribe.unsubscribe();
                            resolve(undefined);
                        }
                    });
                });

                // Re-check if we should continue
                if (!this.state.getValue().isRunning) {
                    break;
                }
            }

            if (!this.step()) {
                break;
            }
        }
    }

    // Optimized step without recursive calls and minimal state updates
    private stepOptimized(): boolean {
        const currentState = this.state.getValue();
        const tape = currentState.tape;
        let pointer = currentState.pointer;
        let outputChanged = false;
        let newOutput = currentState.output;

        while (true) {
            const char = this.getCurrentChar();
            const currentPos = this.currentChar.getValue();

            // Check breakpoints (same as before)
            if (char && '><+-[].,'.includes(char) && this.shouldPauseAtBreakpoint(currentPos)) {
                const isSameBreakpoint = this.lastPausedBreakpoint &&
                    this.lastPausedBreakpoint.line === currentPos.line &&
                    this.lastPausedBreakpoint.column === currentPos.column;

                if (!isSameBreakpoint) {
                    console.log(`Hit breakpoint at line ${currentPos.line}, column ${currentPos.column}`);
                    this.lastPausedBreakpoint = { ...currentPos };
                    this.pause();
                    return true;
                }
            }

            if (this.lastPausedBreakpoint &&
                (this.lastPausedBreakpoint.line !== currentPos.line ||
                    this.lastPausedBreakpoint.column !== currentPos.column)) {
                this.lastPausedBreakpoint = null;
            }

            // Handle special characters
            if (char === '/') {
                const hasMore = this.moveToNextLine();
                if (!hasMore) {
                    this.stop();
                    return false;
                }
                continue; // Loop instead of recursion
            }

            // Skip non-commands
            if (char === null || (char && !'><+-[].,'.includes(char))) {
                const hasMore = this.moveToNextChar();
                if (!hasMore) {
                    this.stop();
                    return false;
                }
                continue; // Loop instead of recursion
            }

            // Execute command
            let shouldMoveNext = true;

            switch (char) {
                case '>':
                    pointer = (pointer + 1) % tape.length;
                    break;
                case '<':
                    pointer = (pointer - 1 + tape.length) % tape.length;
                    break;
                case '+':
                    tape[pointer] = (tape[pointer] + 1) % this.cellSize.getValue();
                    break;
                case '-':
                    tape[pointer] = (tape[pointer] - 1 + this.cellSize.getValue()) % this.cellSize.getValue();
                    break;
                case '[':
                    if (tape[pointer] === 0) {
                        const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                        if (matchingPos) {
                            this.currentChar.next(matchingPos);
                            shouldMoveNext = true;
                        }
                    }
                    break;
                case ']':
                    if (tape[pointer] !== 0) {
                        const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                        if (matchingPos) {
                            this.currentChar.next(matchingPos);
                            shouldMoveNext = true;
                        }
                    }
                    break;
                case '.':
                    newOutput += String.fromCharCode(tape[pointer]);
                    outputChanged = true;
                    break;
                case ',':
                    console.log(`Input requested at position ${pointer}`);
                    break;
            }

            // Update state only if needed
            if (pointer !== currentState.pointer || outputChanged) {
                this.state.next({
                    ...currentState,
                    tape: tape,
                    pointer: pointer,
                    output: newOutput
                });
            }

            if (shouldMoveNext) {
                const hasMore = this.moveToNextChar();
                if (!hasMore) {
                    this.stop();
                    return false;
                }
            }

            return true; // Successfully executed one instruction
        }
    }

// Ultra-fast version for compute-heavy programs
    public async runUltraFast() {
        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        const BATCH_SIZE = 100000; // Execute 100k instructions per batch
        let totalSteps = 0;
        const startTime = performance.now();

        while (this.state.getValue().isRunning && !this.state.getValue().isPaused) {
            // Execute a batch
            let batchSteps = 0;
            for (let i = 0; i < BATCH_SIZE; i++) {
                if (!this.stepOptimized()) {
                    const totalTime = (performance.now() - startTime) / 1000;
                    console.log(`Program completed: ${totalSteps + batchSteps} instructions in ${totalTime}s`);
                    return;
                }
                batchSteps++;
            }

            totalSteps += batchSteps;

            // Update UI with progress
            const elapsed = (performance.now() - startTime) / 1000;
            console.log(`Progress: ${totalSteps} instructions in ${elapsed}s (${Math.round(totalSteps/elapsed)} ops/sec)`);

            // Yield to browser
            await new Promise(resolve => setTimeout(resolve, 0));
        }
    }

    // TODO: Move to webworker or wasm for ultra-fast execution. Later.
    public async runTurbo() {
        console.log('Compiling program for turbo execution...');

        // Pre-compile the program into a flat array of operations
        const ops: Array<{type: string, value?: number}> = [];
        const jumpTable: Map<number, number> = new Map();
        const jumpStack: number[] = [];

        // First pass: compile and build jump table
        let opIndex = 0;
        for (let line = 0; line < this.code.length; line++) {
            const text = this.code[line].text;
            for (let col = 0; col < text.length; col++) {
                const char = text[col];
                if ('><+-[].,'.includes(char)) {
                    if (char === '[') {
                        jumpStack.push(opIndex);
                    } else if (char === ']') {
                        const startIndex = jumpStack.pop();
                        if (startIndex !== undefined) {
                            jumpTable.set(startIndex, opIndex);
                            jumpTable.set(opIndex, startIndex);
                        }
                    }
                    ops.push({ type: char });
                    opIndex++;
                }
            }
        }

        console.log(`Compiled ${ops.length} operations. Starting turbo execution...`);

        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false
        });

        const tape = sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue());
        let pointer = 0;
        let output = '';
        let pc = 0; // Program counter
        const startTime = performance.now();
        const UPDATE_INTERVAL = 500_000_000;
        let opsExecuted = 0;

        while (pc < ops.length) {
            const op = ops[pc];

            switch (op.type) {
                case '>': pointer = (pointer + 1) % this.tapeSize.getValue(); break;
                case '<': pointer = (pointer - 1 + this.tapeSize.getValue()) % this.tapeSize.getValue(); break;
                case '+': tape[pointer] = (tape[pointer] + 1) % this.cellSize.getValue(); break;
                case '-': tape[pointer] = (tape[pointer] - 1 + this.cellSize.getValue()) % this.cellSize.getValue(); break;
                case '[':
                    if (tape[pointer] === 0) {
                        pc = jumpTable.get(pc) || pc;
                    }
                    break;
                case ']':
                    if (tape[pointer] !== 0) {
                        pc = jumpTable.get(pc) || pc;
                    }
                    break;
                case '.': output += String.fromCharCode(tape[pointer]); break;
                case ',': tape[pointer] = 0; break;
            }

            pc++;
            opsExecuted++;

            // Ultra-rare UI updates
            if (opsExecuted % UPDATE_INTERVAL === 0) {
                const elapsed = (performance.now() - startTime) / 1000;
                console.log(`Turbo progress: ${opsExecuted} ops in ${elapsed}s (${Math.round(opsExecuted/elapsed)} ops/sec)`);

                // Update state
                this.state.next({
                    ...this.state.getValue(),
                    tape: tape,
                    pointer: pointer,
                    output: this.state.getValue().output + output
                });
                output = '';

                // Check if should stop
                if (!this.state.getValue().isRunning) {
                    break;
                }

                // Brief yield
                await new Promise(resolve => setTimeout(resolve, 0));
            }
        }

        // Final update
        this.state.next({
            ...this.state.getValue(),
            tape: tape,
            pointer: pointer,
            output: this.state.getValue().output + output,
            isRunning: false
        });

        const totalTime = (performance.now() - startTime) / 1000;
        console.log(`Turbo execution completed: ${opsExecuted} operations in ${totalTime}s (${Math.round(opsExecuted/totalTime)} ops/sec)`);
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

        this.state.next({
            ...this.state.getValue(),
            isRunning: false,
            isPaused: false,
            isStopped: true
        });

        this.lastPausedBreakpoint = null;
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
        this.reset();
        console.log(`Tape size set to ${size} bytes`);
    }

    public setCellSize(size: number) {
        if (![256, 65536, 4294967296].includes(size)) {
            throw new Error("Unsupported cell size. Use 256, 65536, or 4294967296.");
        }
        this.cellSize.next(size);
        localStorage.setItem('cellSize', size.toString());
        this.reset();
    }

    public setLaneCount(count: number) {
        if (count < 1 || count > 10) {
            throw new Error("Lane count must be between 1 and 10");
        }
        this.laneCount.next(count);
        localStorage.setItem('brainfuck-ide-lane-count', count.toString());
        this.state.next({
            ...this.state.getValue(),
            laneCount: count
        });
    }
}

export const interpreterStore = new InterpreterStore();