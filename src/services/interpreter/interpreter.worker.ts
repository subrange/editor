console.log('Interpreter worker loading...');

import type {
  Line,
  Position,
} from '../../components/editor/stores/editor.store';
import type { SourceMap, SourceMapEntry } from '../macro-expander/source-map';
import { SourceMapLookup } from '../macro-expander/source-map';

console.log('Interpreter worker imports completed');

// Message types for communication with main thread
interface InitMessage {
  type: 'init';
  code: Line[];
  tapeSize: number;
  cellSize: number;
  laneCount: number;
  sourceMap?: SourceMap;
  sharedTapeBuffer?: SharedArrayBuffer;
}

interface ProvideInputMessage {
  type: 'provideInput';
  char: string;
}

interface ResetMessage {
  type: 'reset';
}

interface StepMessage {
  type: 'step';
}

interface RunTurboMessage {
  type: 'runTurbo';
}

interface ResumeTurboMessage {
  type: 'resumeTurbo';
}

interface PauseMessage {
  type: 'pause';
}

interface StopMessage {
  type: 'stop';
}

interface SetBreakpointsMessage {
  type: 'setBreakpoints';
  breakpoints: Position[];
  sourceBreakpoints: Position[];
}

interface SetPositionMessage {
  type: 'setPosition';
  position: Position;
}

interface SetVMOutputConfigMessage {
  type: 'setVMOutputConfig';
  config: {
    outCellIndex: number;
    outFlagCellIndex: number;
    sparseCellPattern?: {
      start: number; // Starting index (e.g., 4)
      step: number; // Step between indices (e.g., 8)
      count?: number; // Max number of cells to send (default 1024)
    };
  };
}

type WorkerMessage =
  | InitMessage
  | ResetMessage
  | StepMessage
  | RunTurboMessage
  | ResumeTurboMessage
  | PauseMessage
  | StopMessage
  | SetBreakpointsMessage
  | SetPositionMessage
  | SetVMOutputConfigMessage
  | ProvideInputMessage;

// State update message sent to main thread
interface StateUpdateMessage {
  type: 'stateUpdate';
  // Without SharedArrayBuffer, we need to send tape data on important updates
  tapeData?: ArrayBuffer; // Optional - only sent when needed
  pointer: number;
  isRunning: boolean;
  isPaused: boolean;
  isStopped: boolean;
  isWaitingForInput?: boolean;
  output: string;
  currentChar: Position;
  currentSourcePosition?: Position;
  macroContext?: Array<{
    macroName: string;
    parameters?: Record<string, string>;
  }>;
  lastExecutionTime?: number;
  lastOperationCount?: number;
}

interface ErrorMessage {
  type: 'error';
  error: string;
}

interface LogMessage {
  type: 'log';
  message: string;
}

// Worker-side interpreter state
class WorkerInterpreter {
  private code: Line[] = [];
  private tape: Uint8Array | Uint16Array | Uint32Array;
  private pointer = 0;
  private currentChar: Position = { line: 0, column: 0 };
  private loopMap: Map<string, Position> = new Map();
  private isRunning = false;
  private isPaused = false;
  private isStopped = false;
  private isWaitingForInput = false;
  private output = '';
  private breakpoints: Position[] = [];
  private sourceBreakpoints: Position[] = [];
  private lastPausedBreakpoint: Position | null = null;
  private tapeSize: number;
  private cellSize: number;
  private laneCount: number;
  private sourceMapLookup: SourceMapLookup | null = null;
  private currentSourcePosition?: Position;
  private macroContext?: Array<{
    macroName: string;
    parameters?: Record<string, string>;
  }>;
  private vmOutputConfig: {
    outCellIndex: number;
    outFlagCellIndex: number;
    clearOnRead?: boolean;
    sparseCellPattern?: {
      start: number;
      step: number;
      count?: number;
    };
  } | null = null;
  private lastVMFlagValue = 0;
  private lastOutputLength = 0;
  private outputUpdateInterval = 100; // Send output updates every 100 chars
  private lastOutputUpdateTime = 0;
  private outputUpdateTimeInterval = 100; // Send output updates every 100ms (10fps) to avoid interfering with VM output

  // Execution tracking
  private lastExecutionTime?: number;
  private lastOperationCount?: number;
  private wasInTurboMode = false;

  constructor() {
    this.tapeSize = 1024 * 1024;
    this.cellSize = 256;
    this.laneCount = 1;
    this.tape = new Uint8Array(this.tapeSize);
  }

  init(message: InitMessage) {
    this.code = message.code;
    this.tapeSize = message.tapeSize;
    this.cellSize = message.cellSize;
    this.laneCount = message.laneCount;

    // Use SharedArrayBuffer if provided, otherwise create a new tape
    if (message.sharedTapeBuffer) {
      this.tape = this.createTapeFromSharedBuffer(
        message.sharedTapeBuffer,
        message.cellSize,
      );
    } else {
      this.tape = this.createTape(message.cellSize, message.tapeSize);
    }

    this.sourceMapLookup = message.sourceMap
      ? new SourceMapLookup(message.sourceMap)
      : null;
    this.buildLoopMap();
    // Send initial state with tape data
    this.sendStateUpdate(true);
  }

  private createTapeFromSharedBuffer(
    buffer: SharedArrayBuffer,
    cellSize: number,
  ): Uint8Array | Uint16Array | Uint32Array {
    switch (cellSize) {
      case 256:
        return new Uint8Array(buffer);
      case 65536:
        return new Uint16Array(buffer);
      case 4294967296:
        return new Uint32Array(buffer);
      default:
        throw new Error(`Unsupported cell size: ${cellSize}`);
    }
  }

  private createTape(
    cellSize: number,
    tapeSize: number,
  ): Uint8Array | Uint16Array | Uint32Array {
    switch (cellSize) {
      case 256:
        return new Uint8Array(tapeSize).fill(0);
      case 65536:
        return new Uint16Array(tapeSize).fill(0);
      case 4294967296:
        return new Uint32Array(tapeSize).fill(0);
      default:
        throw new Error(`Unsupported cell size: ${cellSize}`);
    }
  }

  reset() {
    // Check if SharedArrayBuffer exists before using instanceof
    const isSharedArrayBuffer =
      typeof SharedArrayBuffer !== 'undefined' &&
      this.tape.buffer instanceof SharedArrayBuffer;

    // If we're using SharedArrayBuffer, just clear it instead of creating new
    if (isSharedArrayBuffer) {
      this.tape.fill(0);
    } else {
      this.tape = this.createTape(this.cellSize, this.tapeSize);
    }
    this.pointer = 0;
    this.currentChar = { line: 0, column: 0 };
    this.isRunning = false;
    this.isPaused = false;
    this.isStopped = false;
    this.isWaitingForInput = false;
    this.output = '';
    this.lastPausedBreakpoint = null;
    this.currentSourcePosition = undefined;
    this.macroContext = undefined;
    this.lastOutputLength = 0;
    this.lastOutputUpdateTime = 0;
    this.lastExecutionTime = undefined;
    this.lastOperationCount = undefined;
    this.wasInTurboMode = false;

    // Reset VM output state
    this.lastVMFlagValue = 0;
    // Note: We intentionally keep vmOutputConfig as it's a configuration that should persist

    // Rebuild loop map to ensure it's fresh
    this.buildLoopMap();

    // Send tape data after reset
    this.sendStateUpdate(true);
  }

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
            this.log(`Unmatched ] at line ${line}, column ${column}`);
            continue;
          }
          const openPos = stack.pop()!;
          this.loopMap.set(this.posToKey(openPos), pos);
          this.loopMap.set(this.posToKey(pos), openPos);
        }
      }
    }

    if (stack.length > 0) {
      this.log(`Unmatched [ brackets: ${stack.length}`);
    }
  }

  private posToKey(pos: Position): string {
    return `${pos.line},${pos.column}`;
  }

  private getCharAt(pos: Position): string | null {
    if (pos.line >= this.code.length) return null;
    const line = this.code[pos.line];
    if (pos.column >= line.text.length) return null;
    return line.text[pos.column];
  }

  private getCurrentChar(): string | null {
    return this.getCharAt(this.currentChar);
  }

  private moveToNextChar(): boolean {
    if (
      this.currentChar.column <
      this.code[this.currentChar.line].text.length - 1
    ) {
      this.currentChar = {
        line: this.currentChar.line,
        column: this.currentChar.column + 1,
      };
    } else if (this.currentChar.line < this.code.length - 1) {
      this.currentChar = {
        line: this.currentChar.line + 1,
        column: 0,
      };
    } else {
      return false;
    }

    if (this.sourceMapLookup) {
      this.updateSourcePosition();
    }

    return true;
  }

  private shouldPauseAtBreakpoint(position: Position): boolean {
    return this.breakpoints.some(
      (bp) => bp.line === position.line && bp.column === position.column,
    );
  }

  step(): boolean {
    const char = this.getCurrentChar();
    const currentPos = this.currentChar;

    // Check for $ in-code breakpoint.rs
    if (char === '$') {
      this.log(
        `Hit in-code breakpoint $ at line ${currentPos.line}, column ${currentPos.column}`,
      );
      this.pause();
      const hasMore = this.moveToNextChar();
      if (!hasMore) {
        this.stop();
        return false;
      }
      return true;
    }

    // Check for breakpoint.rs
    if (
      char &&
      '><+-[].,'.includes(char) &&
      this.shouldPauseAtBreakpoint(currentPos)
    ) {
      const isSameBreakpoint =
        this.lastPausedBreakpoint &&
        this.lastPausedBreakpoint.line === currentPos.line &&
        this.lastPausedBreakpoint.column === currentPos.column;

      if (!isSameBreakpoint) {
        this.log(
          `Hit breakpoint at line ${currentPos.line}, column ${currentPos.column}`,
        );
        this.lastPausedBreakpoint = { ...currentPos };
        this.pause();
        return true;
      }
    }

    if (
      this.lastPausedBreakpoint &&
      (this.lastPausedBreakpoint.line !== currentPos.line ||
        this.lastPausedBreakpoint.column !== currentPos.column)
    ) {
      this.lastPausedBreakpoint = null;
    }

    // Skip non-command characters
    if (char === null || (char && !'><+-[].,'.includes(char))) {
      const hasMore = this.moveToNextChar();
      if (!hasMore) {
        this.log('Program finished.');
        this.stop();
        return false;
      }
      return this.step();
    }

    let shouldMoveNext = true;

    // Execute the instruction
    switch (char) {
      case '>':
        this.pointer = (this.pointer + 1) % this.tape.length;
        break;
      case '<':
        this.pointer = (this.pointer - 1 + this.tape.length) % this.tape.length;
        break;
      case '+':
        this.tape[this.pointer] = (this.tape[this.pointer] + 1) % this.cellSize;
        break;
      case '-':
        this.tape[this.pointer] =
          (this.tape[this.pointer] - 1 + this.cellSize) % this.cellSize;
        break;
      case '[':
        if (this.tape[this.pointer] === 0) {
          const matchingPos = this.loopMap.get(this.posToKey(currentPos));
          if (matchingPos) {
            this.currentChar = matchingPos;
            if (this.sourceMapLookup) {
              this.updateSourcePosition();
            }
            shouldMoveNext = true;
          }
        }
        break;
      case ']':
        if (this.tape[this.pointer] !== 0) {
          const matchingPos = this.loopMap.get(this.posToKey(currentPos));
          if (matchingPos) {
            this.currentChar = matchingPos;
            if (this.sourceMapLookup) {
              this.updateSourcePosition();
            }
            shouldMoveNext = true;
          }
        }
        break;
      case '.':
        this.output += String.fromCharCode(this.tape[this.pointer]);
        break;
      case ',':
        this.log(`Input requested at position ${this.pointer}`);
        this.isWaitingForInput = true;
        this.isPaused = true;
        this.sendStateUpdate(true);
        shouldMoveNext = false; // Don't move yet, wait for input
        return true; // Keep running but paused
    }

    // Check VM output flag
    if (this.vmOutputConfig) {
      const flagValue = this.tape[this.vmOutputConfig.outFlagCellIndex];
      if (flagValue === 1) {
        this.sendVMOutput();
        // Don't clear the flag here - let the callback do it to avoid race condition
        // The callback will clear it after reading the output character
      }
    }

    this.sendStateUpdate();

    if (shouldMoveNext) {
      const hasMore = this.moveToNextChar();
      if (!hasMore) {
        this.log('Program finished.');
        this.stop();
        return false;
      }
    }

    return true;
  }

  async runTurbo() {
    this.log('Compiling program for turbo execution...');
    this.wasInTurboMode = true;

    const ops: Array<{ type: string; position: Position }> = [];
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

    this.log(`Compiled ${ops.length} operations. Starting turbo execution...`);

    // Reset ALL state for a fresh execution
    this.isRunning = true;
    this.isPaused = false;
    this.isStopped = false;
    this.isWaitingForInput = false;
    this.tape.fill(0);
    this.pointer = 0;
    this.output = '';
    this.currentChar = { line: 0, column: 0 };
    this.lastPausedBreakpoint = null;
    this.currentSourcePosition = undefined;
    this.macroContext = undefined;
    this.lastOutputLength = 0;
    this.lastOutputUpdateTime = performance.now();
    this.lastVMFlagValue = 0;
    this.lastExecutionTime = undefined;
    this.lastOperationCount = undefined;
    let pc = 0;
    const startTime = performance.now();
    const UPDATE_INTERVAL = 50_000_000; // Match main thread interval
    let opsExecuted = 0;

    // Cache frequently accessed values as locals for performance
    let pointer = this.pointer;
    const tape = this.tape;
    const tapeSize = this.tapeSize;
    const cellSize = this.cellSize;
    let output = this.output;
    // const vmOutputConfig = this.vmOutputConfig;

    while (pc < ops.length && this.isRunning && !this.isPaused) {
      const op = ops[pc];

      switch (op.type) {
        case '>':
          pointer = (pointer + 1) % tapeSize;
          break;
        case '<':
          pointer = (pointer - 1 + tapeSize) % tapeSize;
          break;
        case '+':
          tape[pointer] = (tape[pointer] + 1) % cellSize;
          break;
        case '-':
          tape[pointer] = (tape[pointer] - 1 + cellSize) % cellSize;
          break;
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
        case '.': {
          output += String.fromCharCode(tape[pointer]);
          this.output = output;
          // this.sendStateUpdate();
          // Check if we should send an output update (but not if VM output is active)
          // const now = performance.now();
          // const vmOutputActive = vmOutputConfig && tape[vmOutputConfig.outFlagCellIndex] === 1;
          // if (!vmOutputActive &&
          //     (output.length - this.lastOutputLength >= this.outputUpdateInterval ||
          //      now - this.lastOutputUpdateTime >= this.outputUpdateTimeInterval)) {
          //   this.output = output;
          //   this.lastOutputLength = output.length;
          //   this.lastOutputUpdateTime = now;
          this.sendStateUpdate(false); // Don't send tape data for output updates
          // }
          break;
        }
        case ',': {
          this.log(`Turbo: Input requested at pointer ${pointer}`);
          // Update instance variables before pausing
          this.pointer = pointer;
          this.output = output;
          this.currentChar = op.position; // IMPORTANT: Update position so provideInput knows where we are
          this.isWaitingForInput = true;
          this.isPaused = true;
          // Keep isRunning = true so we know to resume after input
          this.sendStateUpdate(true);
          return;
        }
        case '$': {
          this.log(`Turbo: Hit in-code breakpoint $ at operation ${pc}`);
          const nextPc = pc + 1;
          if (nextPc < ops.length) {
            this.currentChar = ops[nextPc].position;
          }
          this.isPaused = true;
          // Update instance variables before returning
          this.pointer = pointer;
          this.output = output;
          this.sendStateUpdate();
          return;
        }
      }

      pc++;
      opsExecuted++;

      // Check VM output flag in turbo mode AFTER executing the instruction
      // if (vmOutputConfig) {
      //   const flagValue = tape[vmOutputConfig.outFlagCellIndex];
      //   if (flagValue === 1) {
      //     // Update instance variables before sending VM output
      //     this.pointer = pointer;
      //     this.output = output;
      //     this.sendVMOutput();
      //     // Don't clear the flag here - let the callback do it to avoid race condition
      //     // The callback will clear it after reading the output character
      //   }
      // }

      // Check for regular breakpoints
      if (pc < ops.length) {
        const nextOp = ops[pc];
        if (this.shouldPauseAtBreakpoint(nextOp.position)) {
          this.log(`Turbo: Hit breakpoint at operation ${pc}`);
          this.currentChar = nextOp.position;
          this.lastPausedBreakpoint = { ...nextOp.position };
          this.isPaused = true;
          // Update instance variables before returning
          this.pointer = pointer;
          this.output = output;
          this.sendStateUpdate();
          return;
        }
      }

      // Periodic updates
      if (opsExecuted % UPDATE_INTERVAL === 0) {
        const elapsed = (performance.now() - startTime) / 1000;
        this.log(`Turbo progress: ${opsExecuted} ops in ${elapsed}s`);
        // Don't send tape data during execution for performance
        this.sendStateUpdate(false);

        // Yield to allow message processing
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }

    // Update instance variables with final state
    this.pointer = pointer;
    this.output = output;

    this.isRunning = false;
    this.isStopped = true; // Mark as finished/stopped
    // Send final state with tape data
    this.sendStateUpdate(true);

    const totalTime = (performance.now() - startTime) / 1000;
    this.lastExecutionTime = totalTime;
    this.lastOperationCount = opsExecuted;
    this.log(
      `Turbo execution completed: ${opsExecuted} operations in ${totalTime}s`,
    );
  }

  async resumeTurbo() {
    this.lastPausedBreakpoint = null;
    this.isPaused = false;
    this.isRunning = true;
    this.isStopped = false; // Clear stopped state when resuming
    this.isWaitingForInput = false;
    this.wasInTurboMode = true;

    this.log('Resuming turbo execution from current position...');

    // Compile the program
    const ops: Array<{ type: string; position: Position }> = [];
    const jumpTable: Map<number, number> = new Map();
    const jumpStack: number[] = [];

    let output = this.output;

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
    const currentPos = this.currentChar;
    let startPc = 0;
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i];
      if (
        op.position.line === currentPos.line &&
        op.position.column === currentPos.column
      ) {
        startPc = i;
        break;
      }
    }

    this.log(`Starting turbo from operation ${startPc} of ${ops.length}`);

    // Continue execution from current position
    let pc = startPc;
    const startTime = performance.now();
    const UPDATE_INTERVAL = 50_000_000; // Match main thread interval
    let opsExecuted = 0;

    // Reset output tracking for resume
    this.lastOutputLength = this.output.length;
    this.lastOutputUpdateTime = performance.now();

    // Cache frequently accessed values as locals for performance
    let pointer = this.pointer;
    const tape = this.tape;
    const tapeSize = this.tapeSize;
    const cellSize = this.cellSize;

    while (pc < ops.length && this.isRunning && !this.isPaused) {
      const op = ops[pc];

      switch (op.type) {
        case '>':
          pointer = (pointer + 1) % tapeSize;
          break;
        case '<':
          pointer = (pointer - 1 + tapeSize) % tapeSize;
          break;
        case '+':
          tape[pointer] = (tape[pointer] + 1) % cellSize;
          break;
        case '-':
          tape[pointer] = (tape[pointer] - 1 + cellSize) % cellSize;
          break;
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
        case '.': {
          output += String.fromCharCode(tape[pointer]);
          this.output = output;
          this.sendStateUpdate(false); // Don't send tape data for output updates
          break;
        }
        case ',': {
          this.log(`Turbo (resume): Input requested at pointer ${pointer}`);
          // Update instance variables before pausing
          this.pointer = pointer;
          this.output = output;
          this.currentChar = op.position; // IMPORTANT: Update position so provideInput knows where we are
          this.isWaitingForInput = true;
          this.isPaused = true;
          // Keep isRunning = true so we know to resume after input
          this.sendStateUpdate(true);
          return;
        }
        case '$': {
          this.log(`Turbo: Hit in-code breakpoint $ at operation ${pc}`);
          const nextPc = pc + 1;
          if (nextPc < ops.length) {
            this.currentChar = ops[nextPc].position;
          }
          this.isPaused = true;
          // Update instance variables before returning
          this.pointer = pointer;
          this.output = output;
          this.sendStateUpdate(true);
          return;
        }
      }

      // Check VM output flag in turbo mode
      // if (this.vmOutputConfig) {
      //   const flagValue = this.tape[this.vmOutputConfig.outFlagCellIndex];
      //   if (flagValue === 1) {
      //     this.sendVMOutput();
      //     // Don't clear the flag here - let the callback do it to avoid race condition
      //     // The callback will clear it after reading the output character
      //   }
      // }

      pc++;
      opsExecuted++;

      // Check for regular breakpoints
      if (pc < ops.length) {
        const nextOp = ops[pc];
        if (this.shouldPauseAtBreakpoint(nextOp.position)) {
          this.log(`Turbo: Hit breakpoint at operation ${pc}`);
          this.currentChar = nextOp.position;
          this.lastPausedBreakpoint = { ...nextOp.position };
          this.isPaused = true;
          // Update instance variables before returning
          this.pointer = pointer;
          this.output = output;
          // Always send tape data when hitting a breakpoint
          this.sendStateUpdate(true);
          return;
        }
      }

      // Periodic updates
      if (opsExecuted % UPDATE_INTERVAL === 0) {
        const elapsed = (performance.now() - startTime) / 1000;
        this.log(`Turbo progress: ${opsExecuted} ops in ${elapsed}s`);
        // Update instance variables before sending periodic update
        this.pointer = pointer;
        this.output = output;
        // Don't send tape data during execution for performance
        this.sendStateUpdate(false);

        // Yield to allow message processing
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }

    // Update instance variables with final state
    this.pointer = pointer;
    this.output = output;

    this.isRunning = false;
    this.isStopped = true; // Mark as finished/stopped
    // Send final state with tape data
    this.sendStateUpdate(true);

    const totalTime = (performance.now() - startTime) / 1000;
    this.lastExecutionTime = totalTime;
    this.lastOperationCount = opsExecuted;
    this.log(
      `Turbo execution completed: ${opsExecuted} operations in ${totalTime}s from position ${startPc}`,
    );
  }

  pause() {
    this.isPaused = true;
    // Send tape data when pausing so UI shows correct state
    this.sendStateUpdate(true);
  }

  stop() {
    this.isRunning = false;
    this.isPaused = false;
    this.isStopped = true;
    this.wasInTurboMode = false; // Clear turbo mode flag
    // Send tape data when stopping
    this.sendStateUpdate(true);
  }

  setBreakpoints(breakpoints: Position[], sourceBreakpoints: Position[]) {
    this.breakpoints = breakpoints;
    this.sourceBreakpoints = sourceBreakpoints;
  }

  setPosition(position: Position) {
    this.currentChar = position;
    if (this.sourceMapLookup) {
      this.updateSourcePosition();
    }
    this.sendStateUpdate();
  }

  setVMOutputConfig(config: {
    outCellIndex: number;
    outFlagCellIndex: number;
    clearOnRead?: boolean;
    sparseCellPattern?: {
      start: number;
      step: number;
      count?: number;
    };
  }) {
    this.vmOutputConfig = config;
    this.lastVMFlagValue = 0;
  }

  private updateSourcePosition() {
    if (!this.sourceMapLookup) {
      this.currentSourcePosition = undefined;
      this.macroContext = undefined;
      return;
    }

    const currentPos = this.currentChar;
    const entry = this.sourceMapLookup.getSourcePosition(
      currentPos.line + 1,
      currentPos.column + 1,
    );

    if (entry) {
      this.currentSourcePosition = {
        line: entry.sourceRange.start.line - 1,
        column: entry.sourceRange.start.column - 1,
      };

      const context = this.sourceMapLookup.getMacroContext(
        currentPos.line + 1,
        currentPos.column + 1,
      );

      this.macroContext = context
        .map((e) => ({
          macroName: e.macroName || '',
          parameters: e.parameterValues,
        }))
        .filter((c) => c.macroName);
    } else {
      this.currentSourcePosition = undefined;
      this.macroContext = undefined;
    }
  }

  private sendStateUpdate(includeTapeData = false) {
    // Debug log when finishing
    if (this.isStopped && this.lastExecutionTime !== undefined) {
      this.log(
        `Sending finished state with metrics: time=${this.lastExecutionTime}s, ops=${this.lastOperationCount}`,
      );
    }

    const message: StateUpdateMessage = {
      type: 'stateUpdate',
      pointer: this.pointer,
      isRunning: this.isRunning,
      isPaused: this.isPaused,
      isStopped: this.isStopped,
      isWaitingForInput: this.isWaitingForInput,
      output: this.output,
      currentChar: this.currentChar,
      currentSourcePosition: this.currentSourcePosition,
      macroContext: this.macroContext,
      lastExecutionTime: this.lastExecutionTime,
      lastOperationCount: this.lastOperationCount,
    };

    // Include tape data when requested or when not using SharedArrayBuffer
    // Check if SharedArrayBuffer exists before using instanceof
    const isSharedArrayBuffer =
      typeof SharedArrayBuffer !== 'undefined' &&
      this.tape.buffer instanceof SharedArrayBuffer;

    // Always send tape data when paused, stopped, or explicitly requested
    const shouldSendTape =
      !isSharedArrayBuffer &&
      (includeTapeData || this.isPaused || this.isStopped || !this.isRunning);

    if (shouldSendTape) {
      // Send a copy of the tape buffer
      const bufferCopy = this.tape.buffer.slice(0);
      message.tapeData = bufferCopy;

      // Use transferable for efficiency
      self.postMessage(message, [bufferCopy]);
    } else {
      self.postMessage(message);
    }
  }

  private sendVMOutput() {
    const isSharedArrayBuffer =
      typeof SharedArrayBuffer !== 'undefined' &&
      this.tape.buffer instanceof SharedArrayBuffer;

    // If using SharedArrayBuffer, we can send a minimal message
    if (isSharedArrayBuffer) {
      self.postMessage({
        type: 'vmOutput',
        pointer: this.pointer,
      });
      return;
    }

    // Only send sparse data if not using SharedArrayBuffer
    if (this.vmOutputConfig) {
      const values: number[] = [];
      const indices: number[] = [];

      // Use configured pattern or default to 4, 12, 20, 28...
      const pattern = this.vmOutputConfig.sparseCellPattern || {
        start: 4,
        step: 8,
        count: 1024,
      };

      const maxCells = pattern.count || 1024;

      for (let i = 0; i < maxCells; i++) {
        const index = pattern.start + i * pattern.step;
        if (index >= this.tape.length) break;

        values.push(this.tape[index]);
        indices.push(index);
      }

      self.postMessage({
        type: 'vmOutput',
        pointer: this.pointer,
        sparseTapeData: { values, indices },
      });
    }
  }

  private log(message: string) {
    const logMessage: LogMessage = {
      type: 'log',
      message,
    };
    self.postMessage(logMessage);
  }

  provideInput(char: string) {
    if (!this.isWaitingForInput) {
      this.log('Input provided but interpreter is not waiting for input');
      return;
    }

    // Get ASCII value of the input character
    const asciiValue = char.charCodeAt(0);

    // Place the value in the current cell - this completes the ',' instruction
    this.tape[this.pointer] = asciiValue % this.cellSize;

    // Clear waiting state
    this.isWaitingForInput = false;

    this.log(
      `Input received: '${char}' (ASCII ${asciiValue}) placed at position ${this.pointer}`,
    );

    // Now move to the next instruction since ',' is complete
    const hasMore = this.moveToNextChar();
    if (!hasMore) {
      this.stop();
      return;
    }

    // Resume execution if we were running
    if (this.isRunning) {
      this.isPaused = false; // Clear pause state
      this.sendStateUpdate(true); // Send state update to reflect the change

      // Check if we were in turbo mode
      if (this.wasInTurboMode) {
        // Resume turbo execution from the new position
        this.log('Resuming turbo execution after input...');
        setTimeout(() => {
          this.resumeTurbo();
        }, 10);
      } else {
        // Just send state update for step mode
        this.log('Ready for next step after input');
      }
    } else {
      // Not running, just update state
      this.sendStateUpdate(true);
    }
  }
}

// Create interpreter instance
console.log('Creating WorkerInterpreter instance...');
const interpreter = new WorkerInterpreter();
console.log('WorkerInterpreter instance created');

// Handle messages from main thread
self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  console.log('Worker received message:', message.type);

  try {
    switch (message.type) {
      case 'init':
        interpreter.init(message);
        break;
      case 'reset':
        interpreter.reset();
        break;
      case 'step':
        interpreter.step();
        break;
      case 'runTurbo':
        interpreter.runTurbo();
        break;
      case 'resumeTurbo':
        interpreter.resumeTurbo();
        break;
      case 'pause':
        interpreter.pause();
        break;
      case 'stop':
        interpreter.stop();
        break;
      case 'setBreakpoints':
        interpreter.setBreakpoints(
          message.breakpoints,
          message.sourceBreakpoints,
        );
        break;
      case 'setPosition':
        interpreter.setPosition(message.position);
        break;
      case 'setVMOutputConfig':
        interpreter.setVMOutputConfig(message.config);
        break;
      case 'provideInput':
        interpreter.provideInput(message.char);
        break;
    }
  } catch (error) {
    const errorMessage: ErrorMessage = {
      type: 'error',
      error: error instanceof Error ? error.message : 'Unknown error',
    };
    self.postMessage(errorMessage);
  }
};

console.log('Interpreter worker ready');
