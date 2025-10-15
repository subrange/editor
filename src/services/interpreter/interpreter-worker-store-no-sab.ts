// Alternative implementation without SharedArrayBuffer
// This version uses efficient chunked transfers for large tapes

import { BehaviorSubject, Subscription } from 'rxjs';
import type {
  Line,
  Position,
} from '../../components/editor/stores/editor.store';
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

// Message to request only modified regions
interface RequestTapeRegionMessage {
  type: 'requestTapeRegion';
  start: number;
  length: number;
}

interface TapeRegionMessage {
  type: 'tapeRegion';
  start: number;
  data: ArrayBuffer;
}

export class InterpreterWorkerStoreNoSAB {
  private worker: Worker;
  private code: Line[] = [];
  private vmOutputCallback:
    | ((tape: Uint8Array | Uint16Array | Uint32Array, pointer: number) => void)
    | null = null;
  private editorSubscription: Subscription | null = null;

  // Local tape copy - only update modified regions
  private localTape: Uint8Array | Uint16Array | Uint32Array;
  private modifiedRegions = new Set<number>(); // Track modified regions

  // Observable state
  public state: BehaviorSubject<InterpreterState>;
  public currentChar = new BehaviorSubject<Position>({ line: 0, column: 0 });
  public currentSourceChar = new BehaviorSubject<Position | null>(null);
  public tapeSize = new BehaviorSubject<number>(1024 * 1024);
  public cellSize = new BehaviorSubject<number>(256);
  public laneCount = new BehaviorSubject<number>(1);

  constructor() {
    // Initialize local tape
    this.localTape = new Uint8Array(this.tapeSize.getValue());

    // Initialize state
    this.state = new BehaviorSubject<InterpreterState>({
      tape: this.localTape,
      pointer: 0,
      isRunning: false,
      isPaused: false,
      isStopped: false,
      breakpoints: [],
      sourceBreakpoints: [],
      output: '',
      laneCount: 1,
      lastExecutionMode: 'turbo',
    });

    // Create worker without special requirements
    this.worker = new Worker(
      new URL('./interpreter-worker-no-sab.ts', import.meta.url),
      { type: 'module' },
    );

    // Handle worker messages
    this.worker.onmessage = (event) => {
      const message = event.data;

      switch (message.type) {
        case 'stateUpdate':
          this.handleStateUpdate(message);
          break;
        case 'tapeRegion':
          this.handleTapeRegion(message);
          break;
        case 'requestTapeRegion':
          this.sendTapeRegion(message.start, message.length);
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

    // Load settings from localStorage
    this.loadSettings();

    // Subscribe to editor updates
    this.subscribeToEditor();
  }

  private loadSettings() {
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

    // Recreate tape with correct size
    this.recreateTape();
  }

  private recreateTape() {
    const tapeSize = this.tapeSize.getValue();
    const cellSize = this.cellSize.getValue();

    if (cellSize === 256) {
      this.localTape = new Uint8Array(tapeSize);
    } else if (cellSize === 65536) {
      this.localTape = new Uint16Array(tapeSize);
    } else {
      this.localTape = new Uint32Array(tapeSize);
    }

    const currentState = this.state.getValue();
    this.state.next({
      ...currentState,
      tape: this.localTape,
    });
  }

  private handleStateUpdate(message: any) {
    const currentState = this.state.getValue();

    // Update state without tape (tape updates come separately)
    this.state.next({
      ...currentState,
      pointer: message.pointer,
      isRunning: message.isRunning,
      isPaused: message.isPaused,
      isStopped: message.isStopped,
      output: message.output,
      currentSourcePosition: message.currentSourcePosition,
      macroContext: message.macroContext,
    });

    this.currentChar.next(message.currentChar);

    if (message.currentSourcePosition) {
      this.currentSourceChar.next(message.currentSourcePosition);
    } else {
      this.currentSourceChar.next(null);
    }

    // Track modified regions if provided
    if (message.modifiedRegions) {
      message.modifiedRegions.forEach((region: number) => {
        this.modifiedRegions.add(region);
      });
    }
  }

  private handleTapeRegion(message: TapeRegionMessage) {
    // Update local tape with the received region
    const view = new Uint8Array(message.data);
    const targetArray = new Uint8Array(this.localTape.buffer, message.start);
    targetArray.set(view);

    // Trigger state update
    const currentState = this.state.getValue();
    this.state.next({
      ...currentState,
      tape: this.localTape,
    });
  }

  private sendTapeRegion(start: number, length: number) {
    const region = this.localTape.slice(start, start + length);
    this.worker.postMessage(
      {
        type: 'tapeRegion',
        start,
        data: region.buffer,
      },
      [region.buffer],
    );
  }

  private handleVMOutput(message: any) {
    if (this.vmOutputCallback) {
      this.vmOutputCallback(this.localTape, message.pointer);
    }
  }

  // ... rest of the methods similar to the original but adapted for no-SAB approach
}
