// Modified brainfuck interpreter with loop support

import {BehaviorSubject, Subscription} from "rxjs";
import {type Line, type Position} from "../editor/stores/editor.store.ts";
import {editorManager} from "../../services/editor-manager.service.ts";
import type { SourceMap, SourceMapEntry } from "../../services/macro-expander/source-map.ts";
import { SourceMapLookup } from "../../services/macro-expander/source-map.ts";
import { settingsStore } from "../../stores/settings.store.ts";

type InterpreterState = {
    tape: Uint8Array | Uint16Array | Uint32Array;
    pointer: number;

    isRunning: boolean;
    isPaused: boolean;
    isStopped: boolean;
    isWaitingForInput: boolean; // New flag for input state

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

export type TapeSnapshot = {
    id: string;
    name: string;
    timestamp: number;
    tape: number[];
    pointer: number;
    cellSize: number;
    tapeSize: number;
    labels?: {
        lanes: Record<number, string>;
        columns: Record<number, string>;
        cells: Record<number, string>;
    };
}

const DEFAULT_TAPE_SIZE = 30000; // Default tape size
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
        isWaitingForInput: false,
        breakpoints: [],
        sourceBreakpoints: [],
        output: '',
        laneCount: DEFAULT_LANE_COUNT,
        sourceMap: undefined,
        currentSourcePosition: undefined,
        macroContext: undefined
    })

    private code: Array<Line> = [];
    private sourceMapLookup: SourceMapLookup | null = null;
    
    // VM output monitoring callback - called on every instruction during execution
    private vmOutputCallback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null = null;
    private vmOutputConfig: { outCellIndex: number; outFlagCellIndex: number } | null = null;
    private lastVMFlagValue: number = 0;
    
    public setVMOutputCallback(callback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null) {
        this.vmOutputCallback = callback;
        console.log('VM output callback set:', callback ? 'yes' : 'no');
    }
    
    public setVMOutputConfig(config: { outCellIndex: number; outFlagCellIndex: number }) {
        this.vmOutputConfig = config;
        this.lastVMFlagValue = 0;
    }

    public currentChar = new BehaviorSubject<Position>({
        line: 0,
        column: 0
    })
    
    // Position in source (macro) code during debugging
    public currentSourceChar = new BehaviorSubject<Position | null>(null);

    private loopMap: Map<string, Position> = new Map();
    private runInterval: number | null = null;
    private runAnimationFrameId: number | null = null;

    private lastPausedBreakpoint: Position | null = null;
    
    // Execution tracking
    private executionStartTime: number | null = null;
    private operationCount: number = 0;

    public tapeSize = new BehaviorSubject<number>(DEFAULT_TAPE_SIZE);
    public cellSize = new BehaviorSubject<number>(DEFAULT_CELL_SIZE);
    public laneCount = new BehaviorSubject<number>(DEFAULT_LANE_COUNT);

    private editorSubscription: Subscription | null = null;
    
    constructor() {
        // Always subscribe to the main editor, not the active editor
        // This ensures debugger_ui always runs code from main editor
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
            sourceBreakpoints: [],
            output: '',
            laneCount: this.laneCount.getValue()
        });
    }

    public reset() {
        const currentState = this.state.getValue();
        this.state.next({
            tape: sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue()),
            pointer: 0,
            isRunning: false,
            isPaused: false,
            isStopped: false,
            isWaitingForInput: false,
            breakpoints: currentState.breakpoints, // Keep existing breakpoints
            sourceBreakpoints: currentState.sourceBreakpoints, // Keep existing source breakpoints
            output: '',
            laneCount: this.laneCount.getValue(),
            sourceMap: currentState.sourceMap,
            currentSourcePosition: undefined,
            macroContext: undefined,
            lastExecutionTime: undefined,
            lastOperationCount: undefined
        });
        this.currentChar.next({
            line: 0,
            column: 0
        });
        this.currentSourceChar.next(null);
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;
        }

        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }

        this.lastPausedBreakpoint = null;
        this.executionStartTime = null;
        this.operationCount = 0;
    }

    public runFromPosition(position: Position) {
        // Stop any running execution
        if (this.runInterval) {
            clearInterval(this.runInterval);
            this.runInterval = null;
        }
        if (this.runAnimationFrameId) {
            cancelAnimationFrame(this.runAnimationFrameId);
            this.runAnimationFrameId = null;
        }
        
        // Update state to prepare for run, but keep tape and pointer
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: false,
            isPaused: false,
            isStopped: false,
            output: ''  // Clear output for fresh run
        });
        
        // Set the current character position to start from
        this.currentChar.next(position);
        
        // Update source position if we have a source map
        if (this.sourceMapLookup) {
            this.updateSourcePosition();
        }
        
        // Start running smoothly from this position
        this.runSmooth();
    }

    public stepToPosition(targetPosition: Position) {
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            isRunning: false,
            isPaused: false,
            isStopped: false
        });

        const currentChar = this.currentChar.getValue();

        if (targetPosition.line === currentChar.line && targetPosition.column === currentChar.column) {
            return;
        }

        this.currentChar.next(targetPosition);
        
        // Update source position if we have a source map
        if (this.sourceMapLookup) {
            this.updateSourcePosition();
        }

        this.step();
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
        
        // Update source position if we have a source map
        if (this.sourceMapLookup) {
            this.updateSourcePosition();
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
            
            // Update source position if we have a source map
            if (this.sourceMapLookup) {
                this.updateSourcePosition();
            }
            
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
            // Remove breakpoint.rs
            breakpoints.splice(index, 1);
        } else {
            // Add breakpoint.rs
            breakpoints.push(position);
        }

        this.state.next({
            ...currentState,
            breakpoints
        });
    }
    
    public toggleSourceBreakpoint(sourcePosition: Position) {
        const currentState = this.state.getValue();
        const sourceBreakpoints = [...(currentState.sourceBreakpoints || [])];
        const index = sourceBreakpoints.findIndex((bp: Position) => bp.line === sourcePosition.line && bp.column === sourcePosition.column);
        
        if (index !== -1) {
            // Remove source breakpoint.rs
            sourceBreakpoints.splice(index, 1);
        } else {
            // Add source breakpoint.rs
            sourceBreakpoints.push(sourcePosition);
        }
        
        // Update source breakpoints in state
        this.state.next({
            ...currentState,
            sourceBreakpoints
        });
        
        if (!this.sourceMapLookup) {
            // No source map, also update regular breakpoints
            this.toggleBreakpoint(sourcePosition);
            return;
        }
        
        // Get all expanded positions for this source position
        // Note: source map uses 1-based line numbers
        // When setting breakpoints at column 0, try column 1 first since source maps typically start at column 1
        let expandedEntries = this.sourceMapLookup.getExpandedPositions(
            sourcePosition.line + 1,
            sourcePosition.column + 1
        );
        
        // If no entries found at column 0, try column 1
        if (expandedEntries.length === 0 && sourcePosition.column === 0) {
            expandedEntries = this.sourceMapLookup.getExpandedPositions(
                sourcePosition.line + 1,
                1  // Column 1 in source map (1-based)
            );
        }
        
        if (expandedEntries.length === 0) {
            // No direct mapping found. Try to find the nearest Brainfuck command on this line
            // by looking for any source map entry that starts on this line
            const lineKey = `line:${sourcePosition.line + 1}`;
            const lineEntries = (this.sourceMapLookup as any).sourceMap?.sourceToExpanded.get(lineKey) || [];
            
            if (lineEntries.length > 0) {
                // Found entries on this line, use them
                expandedEntries = lineEntries;
            } else {
                // Still no entries found. This might be a comment line or empty line.
                // We'll allow the breakpoint.rs but warn the user
                console.warn('No expanded positions found for source position:', sourcePosition);
                console.warn('This line may not contain executable code (e.g., comment or empty line)');
                
                // Don't set a breakpoint.rs that can't be hit
                return;
            }
        }
        
        const breakpoints = [...currentState.breakpoints];
        
        // Find the entry that represents the full macro expansion (largest range)
        let fullExpansionEntry = expandedEntries[0];
        for (const entry of expandedEntries) {
            const entryLength = entry.expandedRange.end.column - entry.expandedRange.start.column;
            const fullLength = fullExpansionEntry.expandedRange.end.column - fullExpansionEntry.expandedRange.start.column;
            if (entryLength > fullLength) {
                fullExpansionEntry = entry;
            }
        }
        
        // For the full expansion, find the first BF command position
        const expandedPositions: Position[] = [];
        if (fullExpansionEntry) {
            // For now, just use the start position of the full expansion
            // TODO: Get the actual text to find the first BF command
            expandedPositions.push({
                line: fullExpansionEntry.expandedRange.start.line - 1,
                column: fullExpansionEntry.expandedRange.start.column - 1
            });
        }
        
        // If no BF command found in the expansion, fall back to start position
        if (expandedPositions.length === 0 && fullExpansionEntry) {
            expandedPositions.push({
                line: fullExpansionEntry.expandedRange.start.line - 1,
                column: fullExpansionEntry.expandedRange.start.column - 1
            });
        }
        
        // Check if any expanded position already has a breakpoint.rs
        let hasBreakpoint = false;
        for (const expandedPos of expandedPositions) {
            if (breakpoints.some(bp => bp.line === expandedPos.line && bp.column === expandedPos.column)) {
                hasBreakpoint = true;
                break;
            }
        }
        
        if (hasBreakpoint) {
            // Remove all breakpoints at expanded positions
            for (const expandedPos of expandedPositions) {
                const index = breakpoints.findIndex(
                    bp => bp.line === expandedPos.line && bp.column === expandedPos.column
                );
                if (index !== -1) {
                    breakpoints.splice(index, 1);
                }
            }
        } else {
            // Add breakpoints at all expanded positions
            breakpoints.push(...expandedPositions);
        }
        
        this.state.next({
            ...currentState,
            breakpoints,
            sourceBreakpoints  // Preserve the source breakpoints we just set
        });
    }

    public clearBreakpoints() {
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            breakpoints: [],
            sourceBreakpoints: []
        });
        this.lastPausedBreakpoint = null; // Clear last paused breakpoint.rs as well
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

        // Check for $ in-code breakpoint.rs
        if (char === '$') {
            console.log(`Hit in-code breakpoint $ at line ${currentPos.line}, column ${currentPos.column}`);
            this.pause();
            // Move past the $ character before pausing
            const hasMore = this.moveToNextChar();
            if (!hasMore) {
                this.stop();
                return false;
            }
            return true;
        }

        // Check for breakpoint.rs BEFORE executing the instruction
        // But skip if this is the same breakpoint.rs we just paused at
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

        // Clear the last paused breakpoint.rs if we've moved away from it
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
                this.operationCount++;
                break;
            case '<':
                currentState.pointer = (currentState.pointer - 1 + currentState.tape.length) % currentState.tape.length;
                this.operationCount++;
                break;
            case '+':
                const increment = settingsStore.settings.getValue().weird?.doublePlus ? 2 : 1;
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] + increment) % this.cellSize.getValue();
                this.operationCount++;
                break;
            case '-':
                currentState.tape[currentState.pointer] = (currentState.tape[currentState.pointer] - 1 + this.cellSize.getValue()) % this.cellSize.getValue();
                this.operationCount++;
                break;
            case '[':
                if (currentState.tape[currentState.pointer] === 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        if (this.sourceMapLookup) {
                            this.updateSourcePosition();
                        }
                        shouldMoveNext = true;
                    } else {
                        console.error(`No matching ] for [ at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                this.operationCount++;
                break;
            case ']':
                if (currentState.tape[currentState.pointer] !== 0) {
                    const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                    if (matchingPos) {
                        this.currentChar.next(matchingPos);
                        if (this.sourceMapLookup) {
                            this.updateSourcePosition();
                        }
                        shouldMoveNext = true;
                    } else {
                        console.error(`No matching [ for ] at ${currentPos.line}:${currentPos.column}`);
                    }
                }
                this.operationCount++;
                break;
            case '.':
                currentState.output += String.fromCharCode(currentState.tape[currentState.pointer]);
                this.operationCount++;
                break;
            case ',':
                console.log(`Input requested at position ${currentState.pointer}`);
                // Set waiting for input state and pause execution
                currentState.isWaitingForInput = true;
                currentState.isPaused = true;
                this.state.next(currentState);
                this.operationCount++;
                shouldMoveNext = false; // Don't move yet, wait for input
                break;
        }

        // Call VM output callback after each instruction
        if (this.vmOutputCallback) {
            this.vmOutputCallback(currentState.tape, currentState.pointer);
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

        // Clear the last paused breakpoint.rs so we don't immediately break again
        this.lastPausedBreakpoint = null;

        this.state.next({
            ...currentState,
            isPaused: false
        });

        // If there's no active execution loop (e.g., after turbo mode breakpoint.rs), start one
        if (!this.runInterval && !this.runAnimationFrameId) {
            // After turbo mode breakpoint.rs, we must continue in normal mode
            // because turbo mode can't resume from a specific position
            this.runSmooth();
        }
    }

    public run(delay: number = 100) {
        if (this.runInterval) {
            clearInterval(this.runInterval);
        }

        if (!this.executionStartTime) {
            this.executionStartTime = performance.now();
            this.operationCount = 0;
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
        if (!this.executionStartTime) {
            this.executionStartTime = performance.now();
            this.operationCount = 0;
        }
        
        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false,
            lastExecutionMode: 'normal'
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

            // Check for $ in-code breakpoint.rs
            if (char === '$') {
                console.log(`Hit in-code breakpoint $ at line ${currentPos.line}, column ${currentPos.column}`);
                this.pause();
                // Move past the $ character before pausing
                const hasMore = this.moveToNextChar();
                if (!hasMore) {
                    this.stop();
                    return false;
                }
                return true;
            }

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
                    const increment = settingsStore.settings.getValue().weird?.doublePlus ? 2 : 1;
                    tape[pointer] = (tape[pointer] + increment) % this.cellSize.getValue();
                    break;
                case '-':
                    tape[pointer] = (tape[pointer] - 1 + this.cellSize.getValue()) % this.cellSize.getValue();
                    break;
                case '[':
                    if (tape[pointer] === 0) {
                        const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                        if (matchingPos) {
                            this.currentChar.next(matchingPos);
                            if (this.sourceMapLookup) {
                                this.updateSourcePosition();
                            }
                            shouldMoveNext = true;
                        }
                    }
                    break;
                case ']':
                    if (tape[pointer] !== 0) {
                        const matchingPos = this.loopMap.get(this.posToKey(currentPos));
                        if (matchingPos) {
                            this.currentChar.next(matchingPos);
                            if (this.sourceMapLookup) {
                                this.updateSourcePosition();
                            }
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
                    // Need to pause for input
                    this.state.next({
                        ...currentState,
                        tape: tape,
                        pointer: pointer,
                        output: newOutput,
                        isWaitingForInput: true,
                        isPaused: true
                    });
                    return true; // Don't continue processing
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

    private async runTurboFromCurrentPosition() {
        console.log('Resuming turbo execution from current position...');
        
        // Compile the program
        const ops: Array<{type: string, value?: number, position: Position}> = [];
        const jumpTable: Map<number, number> = new Map();
        const jumpStack: number[] = [];
        
        // Build operations and jump table
        let opIndex = 0;
        for (let line = 0; line < this.code.length; line++) {
            const text = this.code[line].text;
            for (let col = 0; col < text.length; col++) {
                const char = text[col];
                if ('><+-[].,$'.includes(char)) {
                    if (char === '[') {
                        jumpStack.push(opIndex);
                    } else if (char === ']') {
                        const startIndex = jumpStack.pop();
                        if (startIndex !== undefined) {
                            jumpTable.set(startIndex, opIndex);
                            jumpTable.set(opIndex, startIndex);
                        }
                    }
                    ops.push({ type: char, position: { line, column: col } });
                    opIndex++;
                }
            }
        }
        
        // Find the operation index for current position
        const currentPos = this.currentChar.getValue();
        let startPc = 0;
        for (let i = 0; i < ops.length; i++) {
            const op = ops[i];
            if (op.position.line === currentPos.line && op.position.column === currentPos.column) {
                startPc = i;
                break;
            }
        }
        
        console.log(`Starting turbo from operation ${startPc} of ${ops.length}`);
        
        // Get current state
        const currentState = this.state.getValue();
        const tape = currentState.tape;
        let pointer = currentState.pointer;
        let output = '';
        let pc = startPc;
        const startTime = performance.now();
        const UPDATE_INTERVAL = 500_000_000;
        let opsExecuted = 0;
        
        while (pc < ops.length) {
            const op = ops[pc];
            
            switch (op.type) {
                case '>': pointer = (pointer + 1) % this.tapeSize.getValue(); break;
                case '<': pointer = (pointer - 1 + this.tapeSize.getValue()) % this.tapeSize.getValue(); break;
                case '+': {
                    const increment = settingsStore.settings.getValue().weird?.doublePlus ? 2 : 1;
                    tape[pointer] = (tape[pointer] + increment) % this.cellSize.getValue();
                    break;
                }
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
                case ',': {
                    // Need to pause turbo mode for input
                    console.log(`Turbo: Input requested at pointer ${pointer}`);
                    
                    // Update state to show we're waiting for input
                    this.state.next({
                        ...this.state.getValue(),
                        tape: tape,
                        pointer: pointer,
                        output: this.state.getValue().output + output,
                        isWaitingForInput: true,
                        isPaused: true,
                        isRunning: true
                    });
                    
                    // Store current position for resuming
                    this.currentChar.next(ops[pc].position);
                    
                    console.log('Pausing turbo mode for input.');
                    return;
                }
                case '$': {
                    // Hit in-code breakpoint.rs
                    console.log(`Turbo: Hit in-code breakpoint $ at operation ${pc}`);
                    
                    // Update position for next resume
                    const nextPc = pc + 1;
                    if (nextPc < ops.length) {
                        this.currentChar.next(ops[nextPc].position);
                    }
                    
                    this.state.next({
                        ...this.state.getValue(),
                        tape: tape,
                        pointer: pointer,
                        output: this.state.getValue().output + output,
                        isPaused: true,
                        isRunning: true
                    });
                    
                    console.log('Pausing turbo mode at breakpoint.rs.');
                    return;
                }
            }
            
            // Check VM output flag more efficiently in turbo mode
            if (this.vmOutputCallback && this.vmOutputConfig) {
                const flagValue = tape[this.vmOutputConfig.outFlagCellIndex];
                if (flagValue === 1 && this.lastVMFlagValue === 0) {
                    this.vmOutputCallback(tape, pointer);
                }
                this.lastVMFlagValue = flagValue;
            }
            
            // Check VM output flag more efficiently in turbo mode
            if (this.vmOutputCallback && this.vmOutputConfig) {
                const flagValue = tape[this.vmOutputConfig.outFlagCellIndex];
                if (flagValue === 1 && this.lastVMFlagValue === 0) {
                    this.vmOutputCallback(tape, pointer);
                }
                this.lastVMFlagValue = flagValue;
            }
            
            pc++;
            opsExecuted++;
            
            // Check for regular breakpoints
            if (pc < ops.length) {
                const nextOp = ops[pc];
                if (this.shouldPauseAtBreakpoint(nextOp.position)) {
                    console.log(`Turbo: Hit breakpoint at operation ${pc}`);
                    this.currentChar.next(nextOp.position);
                    this.lastPausedBreakpoint = { ...nextOp.position };
                    
                    this.state.next({
                        ...this.state.getValue(),
                        tape: tape,
                        pointer: pointer,
                        output: this.state.getValue().output + output,
                        isPaused: true,
                        isRunning: true
                    });
                    
                    return;
                }
            }
            
            // Periodic updates
            if (opsExecuted % UPDATE_INTERVAL === 0) {
                const elapsed = (performance.now() - startTime) / 1000;
                console.log(`Turbo progress: ${opsExecuted} ops in ${elapsed}s`);
                
                this.state.next({
                    ...this.state.getValue(),
                    tape: tape,
                    pointer: pointer,
                    output: this.state.getValue().output + output
                });
                output = '';
                
                if (!this.state.getValue().isRunning) {
                    break;
                }
                
                await new Promise(resolve => setTimeout(resolve, 0));
            }
        }
        
        // Completed
        const totalTime = (performance.now() - startTime) / 1000;
        this.state.next({
            ...this.state.getValue(),
            tape: tape,
            pointer: pointer,
            output: this.state.getValue().output + output,
            isRunning: false,
            lastExecutionTime: totalTime,
            lastOperationCount: opsExecuted
        });
        
        console.log(`Turbo execution completed: ${opsExecuted} operations in ${totalTime}s`);
    }

    public async resumeTurbo() {
        // Clear the last paused breakpoint.rs first
        this.lastPausedBreakpoint = null;
        
        // Mark as running and unpaused
        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            lastExecutionMode: 'turbo'
        });
        
        // Start turbo from current position
        await this.runTurboFromCurrentPosition();
    }

    // TODO: Move to webworker or wasm for ultra-fast execution. Later.
    public async runTurbo() {
        console.log('Compiling program for turbo execution...');

        // Pre-compile the program into a flat array of operations
        const ops: Array<{type: string, value?: number, position: Position}> = [];
        const jumpTable: Map<number, number> = new Map();
        const jumpStack: number[] = [];

        // First pass: compile and build jump table
        let opIndex = 0;
        for (let line = 0; line < this.code.length; line++) {
            const text = this.code[line].text;
            for (let col = 0; col < text.length; col++) {
                const char = text[col];
                if ('><+-[].,$'.includes(char)) {
                    if (char === '[') {
                        jumpStack.push(opIndex);
                    } else if (char === ']') {
                        const startIndex = jumpStack.pop();
                        if (startIndex !== undefined) {
                            jumpTable.set(startIndex, opIndex);
                            jumpTable.set(opIndex, startIndex);
                        }
                    }
                    ops.push({ type: char, position: { line, column: col } });
                    opIndex++;
                }
            }
        }

        console.log(`Compiled ${ops.length} operations. Starting turbo execution...`);

        this.state.next({
            ...this.state.getValue(),
            isRunning: true,
            isPaused: false,
            isStopped: false,
            lastExecutionMode: 'turbo'
        });

        const tape = sizeToTape(this.cellSize.getValue(), this.tapeSize.getValue());
        let pointer = 0;
        let output = '';
        let pc = 0; // Program counter
        const startTime = performance.now();
        const UPDATE_INTERVAL = 500_000_000;
        let opsExecuted = 0;
        
        // Reset VM output tracking
        this.lastVMFlagValue = 0;

        while (pc < ops.length) {
            const op = ops[pc];

            switch (op.type) {
                case '>': pointer = (pointer + 1) % this.tapeSize.getValue(); break;
                case '<': pointer = (pointer - 1 + this.tapeSize.getValue()) % this.tapeSize.getValue(); break;
                case '+': {
                    const increment = settingsStore.settings.getValue().weird?.doublePlus ? 2 : 1;
                    tape[pointer] = (tape[pointer] + increment) % this.cellSize.getValue();
                    break;
                }
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
                case ',': {
                    // Need to pause turbo mode for input
                    console.log(`Turbo: Input requested at pointer ${pointer}`);
                    
                    // Update state to show we're waiting for input
                    this.state.next({
                        ...this.state.getValue(),
                        tape: tape,
                        pointer: pointer,
                        output: this.state.getValue().output + output,
                        isWaitingForInput: true,
                        isPaused: true,
                        isRunning: true
                    });
                    
                    // Store current position for resuming
                    this.currentChar.next(ops[pc].position);
                    
                    console.log('Pausing turbo mode for input.');
                    return;
                }
                case '$': {
                    // Hit in-code breakpoint.rs - update state and pause
                    console.log(`Turbo: Hit in-code breakpoint $ at operation ${pc}`);
                    
                    // Update currentChar to the position after the $ so step works correctly
                    const nextPc = pc + 1;
                    if (nextPc < ops.length) {
                        this.currentChar.next(ops[nextPc].position);
                    } else {
                        // We're at the end of the program
                        const lastOp = ops[pc];
                        this.currentChar.next({
                            line: lastOp.position.line,
                            column: lastOp.position.column + 1
                        });
                    }
                    
                    this.state.next({
                        ...this.state.getValue(),
                        tape: tape,
                        pointer: pointer,
                        output: this.state.getValue().output + output,
                        isPaused: true,
                        isRunning: true  // Keep running state true so resume works
                    });
                    output = '';
                    
                    // Exit turbo mode and return to normal execution
                    console.log('Exiting turbo mode due to breakpoint.rs. Use step or resume to continue.');
                    return;
                }
            }

            // Check VM output flag more efficiently in turbo mode
            if (this.vmOutputCallback && this.vmOutputConfig) {
                const flagValue = tape[this.vmOutputConfig.outFlagCellIndex];
                if (flagValue === 1 && this.lastVMFlagValue === 0) {
                    this.vmOutputCallback(tape, pointer);
                }
                this.lastVMFlagValue = flagValue;
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
        const totalTime = (performance.now() - startTime) / 1000;
        this.state.next({
            ...this.state.getValue(),
            tape: tape,
            pointer: pointer,
            output: this.state.getValue().output + output,
            isRunning: false,
            lastExecutionTime: totalTime,
            lastOperationCount: opsExecuted
        });

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

        // Calculate execution time if we were running
        let executionTime: number | undefined;
        if (this.executionStartTime) {
            executionTime = (performance.now() - this.executionStartTime) / 1000;
            this.executionStartTime = null;
        }

        this.state.next({
            ...this.state.getValue(),
            isRunning: false,
            isPaused: false,
            isStopped: true,
            lastExecutionTime: executionTime || this.state.getValue().lastExecutionTime,
            lastOperationCount: this.operationCount || this.state.getValue().lastOperationCount
        });

        this.lastPausedBreakpoint = null;
    }

    public hasBreakpointAt(position: Position): boolean {
        const currentState = this.state.getValue();
        return currentState.breakpoints.some(
            bp => bp.line === position.line && bp.column === position.column
        );
    }
    
    public hasSourceBreakpointAt(sourcePosition: Position): boolean {
        const currentState = this.state.getValue();
        const sourceBreakpoints = currentState.sourceBreakpoints || [];
        
        // Check if there's a source breakpoint.rs at this exact position
        return sourceBreakpoints.some(
            bp => bp.line === sourcePosition.line && bp.column === sourcePosition.column
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

    public provideInput(char: string) {
        const currentState = this.state.getValue();
        
        if (!currentState.isWaitingForInput) {
            console.warn('Input provided but interpreter is not waiting for input');
            return;
        }
        
        // Get ASCII value of the input character
        const asciiValue = char.charCodeAt(0);
        
        // Place the value in the current cell
        currentState.tape[currentState.pointer] = asciiValue % this.cellSize.getValue();
        
        // Clear waiting state and resume
        currentState.isWaitingForInput = false;
        currentState.isPaused = false;
        
        console.log(`Input received: '${char}' (ASCII ${asciiValue}) placed at position ${currentState.pointer}`);
        
        this.state.next(currentState);
        
        // Move to next instruction after input
        const hasMore = this.moveToNextChar();
        if (!hasMore) {
            this.stop();
        } else if (currentState.isRunning) {
            // If we were running before input, we need to restart the run loop
            // since it was stopped when we paused for input
            if (!this.runInterval && !this.runAnimationFrameId) {
                // Restart with the same mode as before
                const lastMode = currentState.lastExecutionMode;
                if (lastMode === 'turbo') {
                    this.resumeTurbo();
                } else {
                    // Resume normal execution
                    this.resume();
                }
            }
        }
    }

    public loadSnapshot(snapshot: TapeSnapshot) {

        const currentState = this.state.getValue();
        const cellSize = this.cellSize.getValue();
        const tapeSize = this.tapeSize.getValue();

        // First set the tape and cell sizes
        if (snapshot.tapeSize !== tapeSize) {
            this.setTapeSize(snapshot.tapeSize);
        }
        if (snapshot.cellSize !== cellSize) {
            this.setCellSize(snapshot.cellSize);
        }

        // Create new tape array of correct type
        let newTape: Uint8Array | Uint16Array | Uint32Array;
        if (snapshot.cellSize === 256) {
            newTape = new Uint8Array(snapshot.tapeSize);
        } else if (snapshot.cellSize === 65536) {
            newTape = new Uint16Array(snapshot.tapeSize);
        } else {
            newTape = new Uint32Array(snapshot.tapeSize);
        }

        // Copy snapshot data
        for (let i = 0; i < Math.min(snapshot.tape.length, newTape.length); i++) {
            newTape[i] = snapshot.tape[i];
        }

        // Update interpreter state
        this.state.next({
            ...currentState,
            tape: newTape,
            pointer: snapshot.pointer
        });
    }
    
    public setSourceMap(sourceMap: SourceMap | undefined) {
        this.sourceMapLookup = sourceMap ? new SourceMapLookup(sourceMap) : null;
        const currentState = this.state.getValue();
        this.state.next({
            ...currentState,
            sourceMap
        });
        
        // Update current source position if we have a source map
        if (this.sourceMapLookup) {
            this.updateSourcePosition();
        }
    }
    
    public getCode(): Line[] {
        return this.code;
    }
    
    private updateSourcePosition() {
        if (!this.sourceMapLookup) {
            this.currentSourceChar.next(null);
            return;
        }
        
        const currentPos = this.currentChar.getValue();
        let entry: SourceMapEntry | null = null;
        
        // If we're at a breakpoint.rs position, try to find the source map entry
        // that corresponds to the outermost macro (where user likely set the breakpoint.rs)
        const currentState = this.state.getValue();
        const isAtBreakpoint = currentState.breakpoints.some(
            bp => bp.line === currentPos.line && bp.column === currentPos.column
        );
        
        if (isAtBreakpoint && this.sourceMapLookup) {
            // When at a breakpoint.rs, try to use the outermost macro context
            // This will be handled by our updated getMacroContext method
            // which should return the full call stack
        }
        
        // Fall back to normal lookup if not at breakpoint.rs or no entries found
        if (!entry) {
            entry = this.sourceMapLookup.getSourcePosition(
                currentPos.line + 1,
                currentPos.column + 1
            );
        }
        
        if (entry) {
            // Convert back to 0-based for consistency with rest of the codebase
            const sourcePos = {
                line: entry.sourceRange.start.line - 1,
                column: entry.sourceRange.start.column - 1
            };
            this.currentSourceChar.next(sourcePos);
            
            // Update macro context
            const context = this.sourceMapLookup.getMacroContext(
                currentPos.line + 1,
                currentPos.column + 1
            );
            
            console.log('Macro context from source map:', context.length, 'entries');
            context.forEach((e, i) => {
                console.log(`  ${i}: ${e.macroName} ${e.parameterValues ? JSON.stringify(e.parameterValues) : ''}`);
            });
            
            const macroContext = context.map(e => ({
                macroName: e.macroName || '',
                parameters: e.parameterValues
            })).filter(c => c.macroName);
            
            const currentState = this.state.getValue();
            this.state.next({
                ...currentState,
                currentSourcePosition: sourcePos,
                macroContext: macroContext.length > 0 ? macroContext : undefined
            });
            
            // Debug logging
            console.log(`Source position updated: Expanded [${currentPos.line}:${currentPos.column}] -> Source [${sourcePos.line}:${sourcePos.column}] (${entry.macroName || 'direct'})`);
        } else {
            this.currentSourceChar.next(null);
            const currentState = this.state.getValue();
            this.state.next({
                ...currentState,
                currentSourcePosition: undefined,
                macroContext: undefined
            });
            
            // Debug logging
            console.log(`No source position for expanded position [${currentPos.line}:${currentPos.column}]`);
        }
    }
}

export const interpreterStore = new InterpreterStore();