import { BehaviorSubject, Subscription } from 'rxjs';
import type { Line, Position } from '../../components/editor/stores/editor.store';
import type { TapeSnapshot } from '../../components/debugger/interpreter.store';
import type { SourceMap } from '../macro-expander/source-map';
import { editorManager } from '../editor-manager.service';

type InterpreterState = {
  tape: Uint8Array | Uint16Array | Uint32Array;
  pointer: number;
  isRunning: boolean;
  isPaused: boolean;
  isStopped: boolean;
  breakpoints: Position[];
  sourceBreakpoints?: Position[];
  output: string;
  laneCount: number;
  sourceMap?: SourceMap;
  currentSourcePosition?: Position;
  macroContext?: Array<{
    macroName: string;
    parameters?: Record<string, string>;
  }>;
  lastExecutionMode?: 'normal' | 'turbo';
};

export class InterpreterWorkerStore {
  private worker: Worker;
  private code: Line[] = [];
  private vmOutputCallback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null = null;
  private editorSubscription: Subscription | null = null;
  private sharedTapeBuffer: SharedArrayBuffer | null = null;
  private sharedTape: Uint8Array | Uint16Array | Uint32Array | null = null;

  // Observable state - will be initialized in constructor after SharedArrayBuffer
  public state: BehaviorSubject<InterpreterState>;
  public currentChar = new BehaviorSubject<Position>({ line: 0, column: 0 });
  public currentSourceChar = new BehaviorSubject<Position | null>(null);
  public tapeSize = new BehaviorSubject<number>(1024 * 1024);
  public cellSize = new BehaviorSubject<number>(256);
  public laneCount = new BehaviorSubject<number>(1);

  constructor() {
    // Initialize state early to avoid undefined errors
    this.state = new BehaviorSubject<InterpreterState>({
      tape: new Uint8Array(1024 * 1024),
      pointer: 0,
      isRunning: false,
      isPaused: false,
      isStopped: false,
      breakpoints: [],
      sourceBreakpoints: [],
      output: '',
      laneCount: 1,
      lastExecutionMode: 'turbo'
    });

    // Create worker
    try {
      console.log('Creating interpreter worker...');
      this.worker = new Worker(
        new URL('./interpreter.worker.ts', import.meta.url),
        { type: 'module' }
      );
      console.log('Interpreter worker created successfully');
    } catch (error) {
      console.error('Failed to create interpreter worker:', error);
      throw error;
    }

    // Handle worker messages
    this.worker.onmessage = (event) => {
      const message = event.data;
      
      switch (message.type) {
        case 'stateUpdate':
          this.handleStateUpdate(message);
          break;
        case 'vmOutput':
          this.handleVMOutput(message);
          break;
        case 'log':
          console.log('[Worker]', message.message);
          break;
        case 'error':
          console.error('[Worker Error]', message.error);
          break;
      }
    };

    this.worker.onerror = (error) => {
      console.error('Worker error:', error);
    };

    // Load settings from localStorage first
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

    // Initialize SharedArrayBuffer for the tape after loading settings
    this.initializeSharedTape();

    // Update state with the shared tape
    if (this.sharedTape) {
      const currentState = this.state.getValue();
      this.state.next({
        ...currentState,
        tape: this.sharedTape,
        laneCount: this.laneCount.getValue()
      });
    }

    // Subscribe to editor updates
    this.subscribeToEditor();
  }

  private initializeSharedTape() {
    const tapeSize = this.tapeSize.getValue();
    const cellSize = this.cellSize.getValue();
    
    let bytesPerCell: number;
    if (cellSize === 256) {
      bytesPerCell = 1;
    } else if (cellSize === 65536) {
      bytesPerCell = 2;
    } else {
      bytesPerCell = 4;
    }
    
    const bufferSize = tapeSize * bytesPerCell;
    
    try {
      // Create SharedArrayBuffer
      this.sharedTapeBuffer = new SharedArrayBuffer(bufferSize);
      
      // Create typed array view
      if (cellSize === 256) {
        this.sharedTape = new Uint8Array(this.sharedTapeBuffer);
      } else if (cellSize === 65536) {
        this.sharedTape = new Uint16Array(this.sharedTapeBuffer);
      } else {
        this.sharedTape = new Uint32Array(this.sharedTapeBuffer);
      }
      
      console.log('SharedArrayBuffer initialized:', bufferSize, 'bytes');
    } catch (error) {
      console.warn('SharedArrayBuffer not available. Using regular ArrayBuffer.');
      console.log('To enable SharedArrayBuffer, configure proper CORS headers.');
      
      // Fall back to regular typed array
      if (cellSize === 256) {
        this.sharedTape = new Uint8Array(tapeSize);
      } else if (cellSize === 65536) {
        this.sharedTape = new Uint16Array(tapeSize);
      } else {
        this.sharedTape = new Uint32Array(tapeSize);
      }
      this.sharedTapeBuffer = null; // Make sure it's null for fallback mode
    }
  }

  private subscribeToEditor() {
    const checkMainEditor = () => {
      const mainEditor = editorManager.getEditor('main');
      if (mainEditor) {
        // Unsubscribe from any previous subscription
        if (this.editorSubscription) {
          this.editorSubscription.unsubscribe();
        }
        
        // Subscribe to main editor only
        this.editorSubscription = mainEditor.editorState.subscribe(s => {
          if (JSON.stringify(s.lines) !== JSON.stringify(this.code)) {
            this.setCode(s.lines);
          }
        });
      } else {
        // Main editor not created yet, check again later
        setTimeout(checkMainEditor, 100);
      }
    };
    
    checkMainEditor();
  }

  private handleStateUpdate(message: any) {
    const currentState = this.state.getValue();
    let tape = this.sharedTape || currentState.tape;
    
    // If tape data is included in the message, update our local tape
    if (message.tapeData) {
      console.log('Received tape data from worker, size:', message.tapeData.byteLength);
      const cellSize = this.cellSize.getValue();
      if (cellSize === 256) {
        tape = new Uint8Array(message.tapeData);
      } else if (cellSize === 65536) {
        tape = new Uint16Array(message.tapeData);
      } else {
        tape = new Uint32Array(message.tapeData);
      }
      
      // Update our local reference if not using SharedArrayBuffer
      if (!this.sharedTapeBuffer) {
        this.sharedTape = tape;
      }
      
      // Log some tape values for debugging
      const nonZeroCount = Array.from(tape.slice(0, 100)).filter(v => v !== 0).length;
      console.log('Tape values (first 100 cells):', nonZeroCount, 'non-zero cells');
    } else {
      console.log('No tape data in state update');
    }
    
    // Update state
    this.state.next({
      ...currentState,
      tape: tape,
      pointer: message.pointer,
      isRunning: message.isRunning,
      isPaused: message.isPaused,
      isStopped: message.isStopped,
      output: message.output,
      currentSourcePosition: message.currentSourcePosition,
      macroContext: message.macroContext
    });

    this.currentChar.next(message.currentChar);
    
    if (message.currentSourcePosition) {
      this.currentSourceChar.next(message.currentSourcePosition);
    } else {
      this.currentSourceChar.next(null);
    }
  }

  private handleVMOutput(message: any) {
    if (this.vmOutputCallback) {
      const currentState = this.state.getValue();
      const tape = currentState.tape;
      
      // If sparse tape data is included, update only those cells
      if (message.sparseTapeData && !this.sharedTapeBuffer) {
        // Update the specific cells directly in the existing tape
        // No need to clone the entire tape
        const { values, indices } = message.sparseTapeData;
        for (let i = 0; i < indices.length; i++) {
          tape[indices[i]] = values[i];
        }
        
        // Trigger a state update to notify observers
        // We're passing the same tape reference, but React/RxJS will still update
        this.state.next({
          ...currentState,
          tape: tape
        });
      }
      
      this.vmOutputCallback(tape, message.pointer);
    }
  }

  public setCode(code: Line[]) {
    this.code = code;
    this.sendInit();
  }

  private sendInit() {
    const currentState = this.state.getValue();
    const message: any = {
      type: 'init',
      code: this.code,
      tapeSize: this.tapeSize.getValue(),
      cellSize: this.cellSize.getValue(),
      laneCount: this.laneCount.getValue(),
      sourceMap: currentState.sourceMap
    };
    
    // Only include sharedTapeBuffer if it exists
    if (this.sharedTapeBuffer) {
      message.sharedTapeBuffer = this.sharedTapeBuffer;
    }
    
    this.worker.postMessage(message);
  }

  public reset() {
    this.worker.postMessage({ type: 'reset' });
  }

  public step(): boolean {
    this.worker.postMessage({ type: 'step' });
    return true; // Worker will handle completion
  }

  public run(delay?: number) {
    // Worker-based interpreter only supports turbo mode
    console.warn('Worker interpreter only supports turbo mode. Use runTurbo() instead.');
    this.runTurbo();
  }

  public runSmooth() {
    // Worker-based interpreter only supports turbo mode
    console.warn('Worker interpreter only supports turbo mode. Use runTurbo() instead.');
    this.runTurbo();
  }

  public runFromPosition(position: Position) {
    this.worker.postMessage({ type: 'setPosition', position });
    this.runTurbo();
  }

  public stepToPosition(position: Position) {
    this.worker.postMessage({ type: 'setPosition', position });
    this.step();
  }

  public async runImmediately() {
    return this.runTurbo();
  }

  public async runTurbo() {
    this.worker.postMessage({ type: 'runTurbo' });
  }

  public async resumeTurbo() {
    this.worker.postMessage({ type: 'resumeTurbo' });
  }

  public pause() {
    this.worker.postMessage({ type: 'pause' });
  }

  public resume() {
    // Clear last paused breakpoint and resume turbo
    this.resumeTurbo();
  }

  public stop() {
    this.worker.postMessage({ type: 'stop' });
  }

  public toggleBreakpoint(position: Position) {
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
    
    this.worker.postMessage({
      type: 'setBreakpoints',
      breakpoints,
      sourceBreakpoints: currentState.sourceBreakpoints || []
    });
  }

  public toggleSourceBreakpoint(position: Position) {
    const currentState = this.state.getValue();
    const sourceBreakpoints = [...(currentState.sourceBreakpoints || [])];
    
    const index = sourceBreakpoints.findIndex(bp => bp.line === position.line && bp.column === position.column);
    if (index !== -1) {
      sourceBreakpoints.splice(index, 1);
    } else {
      sourceBreakpoints.push(position);
    }
    
    this.state.next({
      ...currentState,
      sourceBreakpoints
    });
    
    // For now, also add regular breakpoint
    // TODO: Implement source map lookup for breakpoints
    this.toggleBreakpoint(position);
  }

  public clearBreakpoints() {
    const currentState = this.state.getValue();
    this.state.next({
      ...currentState,
      breakpoints: [],
      sourceBreakpoints: []
    });
    
    this.worker.postMessage({
      type: 'setBreakpoints',
      breakpoints: [],
      sourceBreakpoints: []
    });
  }

  public hasBreakpointAt(position: Position): boolean {
    const currentState = this.state.getValue();
    return currentState.breakpoints.some(
      bp => bp.line === position.line && bp.column === position.column
    );
  }

  public hasSourceBreakpointAt(position: Position): boolean {
    const currentState = this.state.getValue();
    const sourceBreakpoints = currentState.sourceBreakpoints || [];
    return sourceBreakpoints.some(
      bp => bp.line === position.line && bp.column === position.column
    );
  }

  public setTapeSize(size: number) {
    if (size <= 0) {
      throw new Error("Tape size must be a positive integer");
    }
    this.tapeSize.next(size);
    localStorage.setItem('tapeSize', size.toString());
    this.initializeSharedTape(); // Recreate SharedArrayBuffer with new size
    this.reset();
    this.sendInit();
  }

  public setCellSize(size: number) {
    if (![256, 65536, 4294967296].includes(size)) {
      throw new Error("Unsupported cell size. Use 256, 65536, or 4294967296.");
    }
    this.cellSize.next(size);
    localStorage.setItem('cellSize', size.toString());
    this.initializeSharedTape(); // Recreate SharedArrayBuffer with new cell size
    this.reset();
    this.sendInit();
  }

  public setLaneCount(count: number) {
    if (count < 1 || count > 10) {
      throw new Error("Lane count must be between 1 and 10");
    }
    this.laneCount.next(count);
    localStorage.setItem('brainfuck-ide-lane-count', count.toString());
    const currentState = this.state.getValue();
    this.state.next({
      ...currentState,
      laneCount: count
    });
  }

  public loadSnapshot(snapshot: TapeSnapshot) {
    // For worker-based interpreter, we need to reset and apply snapshot
    if (snapshot.tapeSize !== this.tapeSize.getValue()) {
      this.setTapeSize(snapshot.tapeSize);
    }
    if (snapshot.cellSize !== this.cellSize.getValue()) {
      this.setCellSize(snapshot.cellSize);
    }
    
    // TODO: Send snapshot data to worker
    console.warn('Snapshot loading not fully implemented for worker interpreter');
  }

  public setSourceMap(sourceMap: SourceMap | undefined) {
    const currentState = this.state.getValue();
    this.state.next({
      ...currentState,
      sourceMap
    });
    this.sendInit();
  }

  public setVMOutputCallback(callback: ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void) | null) {
    this.vmOutputCallback = callback;
  }

  public setVMOutputConfig(config: { 
    outCellIndex: number; 
    outFlagCellIndex: number;
    sparseCellPattern?: {
      start: number;
      step: number;
      count?: number;
    };
  }) {
    this.worker.postMessage({
      type: 'setVMOutputConfig',
      config
    });
  }

  public destroy() {
    if (this.editorSubscription) {
      this.editorSubscription.unsubscribe();
    }
    this.worker.terminate();
  }
}