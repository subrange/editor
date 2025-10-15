import { BehaviorSubject } from 'rxjs';
import { Position, Line } from '../components/editor/stores/editor.store.ts';

interface WorkerMessage {
  type: string;
  data?: any;
}

interface InterpreterState {
  tape: Uint8Array | Uint16Array | Uint32Array;
  pointer: number;
  isRunning: boolean;
  isPaused: boolean;
  isStopped: boolean;
  breakpoints: Position[];
  output: string;
  laneCount: number;
}

export class WasmInterpreterService {
  private worker: Worker | null = null;
  private pendingRequests = new Map<string, (data: any) => void>();
  private requestId = 0;

  public state = new BehaviorSubject<InterpreterState>({
    tape: new Uint8Array(1024 * 1024),
    pointer: 0,
    isRunning: false,
    isPaused: false,
    isStopped: false,
    breakpoints: [],
    output: '',
    laneCount: 1,
  });

  public currentChar = new BehaviorSubject<Position>({
    line: 0,
    column: 0,
  });

  private breakpoints: Position[] = [];
  private tapeCache: Uint8Array | Uint16Array | Uint32Array;

  constructor(
    private tapeSize: number,
    private cellSize: number,
  ) {
    this.tapeCache = this.createTapeArray(tapeSize, cellSize);
    this.initWorker();
  }

  private createTapeArray(
    size: number,
    cellSize: number,
  ): Uint8Array | Uint16Array | Uint32Array {
    switch (cellSize) {
      case 256:
        return new Uint8Array(size);
      case 65536:
        return new Uint16Array(size);
      case 4294967296:
        return new Uint32Array(size);
      default:
        throw new Error(`Unsupported cell size: ${cellSize}`);
    }
  }

  private async initWorker() {
    // Create worker from the worker file
    this.worker = new Worker(
      new URL('../workers/brainfuck.worker.ts', import.meta.url),
      { type: 'module' },
    );

    this.worker.onmessage = (event: MessageEvent<WorkerMessage>) => {
      this.handleWorkerMessage(event.data);
    };

    this.worker.onerror = (error) => {
      console.error('Worker error:', error);
    };

    // Initialize the interpreter in the worker
    await this.sendMessage('init', {
      tapeSize: this.tapeSize,
      cellSize: this.cellSize,
    });
  }

  private handleWorkerMessage(message: WorkerMessage) {
    const { type, data } = message;

    switch (type) {
      case 'stepped':
      case 'turboProgress':
      case 'turboComplete':
        if (data?.state) {
          const state = JSON.parse(data.state);
          this.updateState(state);
        }
        break;

      case 'state':
        if (data) {
          this.updateState(data);
        }
        break;

      case 'tapeSlice':
        if (data) {
          // Update tape cache with the slice
          const slice = new Uint8Array(data);
          // This is a simplified version - in practice you'd need to track which slice this is
          this.tapeCache.set(slice, 0);
        }
        break;

      case 'error':
        console.error('Worker error:', message);
        break;
    }

    // Resolve any pending request
    const resolver = this.pendingRequests.get(type);
    if (resolver) {
      resolver(data);
      this.pendingRequests.delete(type);
    }
  }

  private updateState(wasmState: any) {
    const currentState = this.state.getValue();

    // Create appropriate tape array if needed
    let tape = currentState.tape;
    if (currentState.tape.length !== this.tapeSize) {
      tape = this.createTapeArray(this.tapeSize, this.cellSize);
    }

    this.state.next({
      tape,
      pointer: wasmState.pointer,
      isRunning: wasmState.is_running,
      isPaused: wasmState.is_paused,
      isStopped: wasmState.is_stopped,
      breakpoints: this.breakpoints,
      output: wasmState.output,
      laneCount: wasmState.lane_count,
    });
  }

  private sendMessage(type: string, data?: any): Promise<any> {
    return new Promise((resolve) => {
      if (!this.worker) {
        throw new Error('Worker not initialized');
      }

      const id = `${type}_${this.requestId++}`;
      this.pendingRequests.set(id, resolve);

      this.worker.postMessage({ type, data });

      // Auto-resolve after timeout
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          resolve(undefined);
        }
      }, 5000);
    });
  }

  public async setCode(lines: Line[]) {
    await this.sendMessage('setCode', { code: lines });
  }

  public async step(): Promise<boolean> {
    const result = await this.sendMessage('step');
    return result?.hasMore ?? false;
  }

  public async runTurbo() {
    await this.sendMessage('runTurbo');
  }

  public async pause() {
    await this.sendMessage('pause');
  }

  public async resume() {
    await this.sendMessage('resume');
  }

  public async stop() {
    await this.sendMessage('stop');
  }

  public async reset() {
    await this.sendMessage('reset');
  }

  public async toggleBreakpoint(position: Position) {
    this.breakpoints = this.breakpoints.filter(
      (bp) => bp.line !== position.line || bp.column !== position.column,
    );

    const exists = this.breakpoints.some(
      (bp) => bp.line === position.line && bp.column === position.column,
    );

    if (!exists) {
      this.breakpoints.push(position);
    }

    await this.sendMessage('toggleBreakpoint', position);
  }

  public async clearBreakpoints() {
    this.breakpoints = [];
    await this.sendMessage('clearBreakpoints');
  }

  public async setTapeSize(size: number) {
    this.tapeSize = size;
    this.tapeCache = this.createTapeArray(size, this.cellSize);
    await this.sendMessage('setTapeSize', { size });
  }

  public async setCellSize(size: number) {
    this.cellSize = size;
    this.tapeCache = this.createTapeArray(this.tapeSize, size);
    await this.sendMessage('setCellSize', { size });
  }

  public async setLaneCount(count: number) {
    await this.sendMessage('setLaneCount', { count });
  }

  public hasBreakpointAt(position: Position): boolean {
    return this.breakpoints.some(
      (bp) => bp.line === position.line && bp.column === position.column,
    );
  }

  public destroy() {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
  }
}
