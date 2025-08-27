import { BehaviorSubject } from 'rxjs';
import { interpreterStore } from '../components/debugger/interpreter-facade.store';

export interface RustWasmOptions {
  tapeSize?: number;
  cellSize?: 8 | 16 | 32;
  wrap?: boolean;
  wrapTape?: boolean;
  optimize?: boolean;
}

export interface RustWasmResult {
  tape: number[];
  pointer: number;
  output: string;
  tapeTruncated?: boolean;
  originalTapeSize?: number;
}

interface RunRequest {
  id: string;
  resolve: (result: RustWasmResult) => void;
  reject: (error: Error) => void;
  outputCallback?: (char: string, charCode: number) => void;
}

class RustWasmInterpreterService {
  private worker: Worker | null = null;
  private isReady = false;
  private pendingRequests = new Map<string, RunRequest>();
  private requestIdCounter = 0;
  private currentRunId: string | null = null;
  
  // Observable for service status
  public status$ = new BehaviorSubject<'initializing' | 'ready' | 'error'>('initializing');
  public isRunning$ = new BehaviorSubject<boolean>(false);
  public isWaitingForInput$ = new BehaviorSubject<boolean>(false);
  
  constructor() {
    this.initializeWorker();
  }
  
  private initializeWorker() {
    try {
      this.worker = new Worker('/wasm-interpreter.worker.js', { type: 'module' });
      
      this.worker.onmessage = (e) => {
        const { type, id, char, charCode, result, error, code } = e.data;
        
        switch (type) {
          case 'waiting_for_input':
            this.isWaitingForInput$.next(true);
            // Update interpreter state to show we're waiting for input
            const currentState = interpreterStore.state.getValue();
            interpreterStore.state.next({
              ...currentState,
              isWaitingForInput: true,
              isPaused: true
            });
            break;
          case 'ready':
            this.isReady = true;
            this.status$.next('ready');
            console.log('Rust WASM interpreter ready');
            break;
            
          case 'output':
            const outputRequest = this.pendingRequests.get(id);
            if (outputRequest?.outputCallback) {
              outputRequest.outputCallback(char, charCode);
            }
            break;
            
          case 'complete':
            const completeRequest = this.pendingRequests.get(id);
            if (completeRequest) {
              completeRequest.resolve(result);
              this.pendingRequests.delete(id);
              if (this.currentRunId === id) {
                this.currentRunId = null;
                this.isRunning$.next(false);
                this.isWaitingForInput$.next(false);
                
                // Clear the waiting state in the interpreter store as well
                const currentState = interpreterStore.state.getValue();
                if (currentState.isWaitingForInput) {
                  interpreterStore.state.next({
                    ...currentState,
                    isWaitingForInput: false,
                    isPaused: false,
                    isStopped: true
                  });
                }
              }
            }
            break;
            
          case 'error':
            if (id) {
              const errorRequest = this.pendingRequests.get(id);
              if (errorRequest) {
                errorRequest.reject(new Error(error));
                this.pendingRequests.delete(id);
                if (this.currentRunId === id) {
                  this.currentRunId = null;
                  this.isRunning$.next(false);
                  this.isWaitingForInput$.next(false);
                  
                  // Clear the waiting state in the interpreter store as well
                  const currentState = interpreterStore.state.getValue();
                  if (currentState.isWaitingForInput) {
                    interpreterStore.state.next({
                      ...currentState,
                      isWaitingForInput: false,
                      isPaused: false,
                      isStopped: true
                    });
                  }
                }
              }
            } else {
              console.error('Rust WASM interpreter error:', error);
              this.status$.next('error');
            }
            break;
            
          case 'optimized':
            const optimizeRequest = this.pendingRequests.get(id);
            if (optimizeRequest) {
              optimizeRequest.resolve({ tape: [], pointer: 0, output: code });
              this.pendingRequests.delete(id);
            }
            break;
        }
      };
      
      this.worker.onerror = (error) => {
        console.error('Worker error:', error);
        this.status$.next('error');
        this.isRunning$.next(false);
        this.currentRunId = null;
        
        // Reject all pending requests
        for (const request of this.pendingRequests.values()) {
          request.reject(new Error('Worker crashed'));
        }
        this.pendingRequests.clear();
      };
      
      // Initialize the WASM module
      this.worker.postMessage({ type: 'init' });
      
    } catch (error) {
      console.error('Failed to create worker:', error);
      this.status$.next('error');
    }
  }
  
  /**
   * Run Brainfuck code with optional real-time output callback
   */
  async runProgram(
    code: string, 
    input: string = '',
    options?: RustWasmOptions,
    outputCallback?: (char: string, charCode: number) => void
  ): Promise<RustWasmResult> {
    if (!this.worker) {
      throw new Error('Worker not initialized');
    }
    
    // Stop any currently running program
    if (this.currentRunId) {
      this.stop();
    }
    
    if (!this.isReady) {
      // Wait for ready state
      await new Promise<void>((resolve, reject) => {
        const checkReady = setInterval(() => {
          if (this.isReady) {
            clearInterval(checkReady);
            resolve();
          }
          if (this.status$.value === 'error') {
            clearInterval(checkReady);
            reject(new Error('WASM initialization failed'));
          }
        }, 100);
        
        // Timeout after 10 seconds
        setTimeout(() => {
          clearInterval(checkReady);
          reject(new Error('WASM initialization timeout'));
        }, 10000);
      });
    }
    
    return new Promise((resolve, reject) => {
      const id = `run_${++this.requestIdCounter}`;
      this.currentRunId = id;
      this.isRunning$.next(true);
      
      this.pendingRequests.set(id, {
        id,
        resolve,
        reject,
        outputCallback
      });
      
      this.worker!.postMessage({
        type: 'run',
        id,
        code,
        input,
        options
      });
    });
  }
  
  /**
   * Stop the currently running program
   */
  stop() {
    if (this.currentRunId) {
      const request = this.pendingRequests.get(this.currentRunId);
      if (request) {
        request.reject(new Error('Program stopped by user'));
        this.pendingRequests.delete(this.currentRunId);
      }
      this.currentRunId = null;
      this.isRunning$.next(false);
      this.isWaitingForInput$.next(false);
      
      // Clear the waiting state in the interpreter store as well
      const currentState = interpreterStore.state.getValue();
      if (currentState.isWaitingForInput) {
        interpreterStore.state.next({
          ...currentState,
          isWaitingForInput: false,
          isPaused: false,
          isStopped: true
        });
      }
      
      // Restart worker to ensure clean state
      this.restart();
    }
  }
  
  /**
   * Optimize Brainfuck code
   */
  async optimizeCode(code: string): Promise<string> {
    if (!this.worker) {
      throw new Error('Worker not initialized');
    }
    
    if (!this.isReady) {
      throw new Error('WASM not ready');
    }
    
    return new Promise((resolve, reject) => {
      const id = `optimize_${++this.requestIdCounter}`;
      
      this.pendingRequests.set(id, {
        id,
        resolve: (result) => resolve(result.output),
        reject,
      });
      
      this.worker!.postMessage({
        type: 'optimize',
        id,
        code
      });
    });
  }
  
  /**
   * Terminate the worker
   */
  terminate() {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
      this.isReady = false;
      this.status$.next('initializing');
      this.isRunning$.next(false);
      this.currentRunId = null;
    }
    
    // Reject all pending requests
    for (const request of this.pendingRequests.values()) {
      request.reject(new Error('Worker terminated'));
    }
    this.pendingRequests.clear();
  }
  
  /**
   * Provide input to the interpreter when it's waiting
   */
  provideInput(char: string) {
    if (!this.worker || !this.currentRunId || !this.isWaitingForInput$.getValue()) {
      console.warn('Cannot provide input: interpreter not waiting for input');
      return;
    }
    
    const charCode = char.charCodeAt(0);
    this.isWaitingForInput$.next(false);
    
    // Clear the waiting state in the interpreter store immediately
    const currentState = interpreterStore.state.getValue();
    if (currentState.isWaitingForInput) {
      interpreterStore.state.next({
        ...currentState,
        isWaitingForInput: false,
        isPaused: false
      });
    }
    
    this.worker.postMessage({
      type: 'provide_input',
      id: this.currentRunId,
      charCode
    });
  }
  
  
  /**
   * Restart the worker
   */
  restart() {
    this.terminate();
    this.initializeWorker();
    this.isWaitingForInput$.next(false);
  }
}

// Export singleton instance
export const rustWasmInterpreter = new RustWasmInterpreterService();

// Make it globally accessible for the IO component
if (typeof window !== 'undefined') {
  (window as any).rustWasmInterpreter = rustWasmInterpreter;
}