import type { MacroExpanderOptions, MacroExpanderResult } from './macro-expander';

interface PendingRequest {
  resolve: (result: MacroExpanderResult) => void;
  reject: (error: Error) => void;
}

export class MacroExpanderWasmWorkerClient {
  private worker: Worker;
  private pendingRequests = new Map<string, PendingRequest>();
  private requestId = 0;

  constructor() {
    // Import WASM worker with Vite's special syntax
    this.worker = new Worker(
      new URL('./macro-expander-wasm.worker.ts', import.meta.url),
      { type: 'module' }
    );

    this.worker.onmessage = (event) => {
      const message = event.data;
      const pending = this.pendingRequests.get(message.id);
      
      if (!pending) {
        console.warn('Received response for unknown request:', message.id);
        return;
      }

      this.pendingRequests.delete(message.id);

      if (message.type === 'result') {
        pending.resolve(message.result);
      } else if (message.type === 'error') {
        pending.reject(new Error(message.error));
      }
    };

    this.worker.onerror = (error) => {
      console.error('WASM Worker error:', error);
      // Reject all pending requests
      for (const [id, pending] of this.pendingRequests) {
        pending.reject(new Error('WASM Worker error'));
        this.pendingRequests.delete(id);
      }
    };
  }

  async expand(input: string, options?: MacroExpanderOptions): Promise<MacroExpanderResult> {
    const id = String(this.requestId++);
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      
      this.worker.postMessage({
        type: 'expand',
        id,
        input,
        options
      });
    });
  }

  // For backward compatibility - synchronous version that throws
  expandSync(_input: string, _options?: MacroExpanderOptions): MacroExpanderResult {
    throw new Error('Synchronous expansion not supported in WASM worker mode. Use expand() instead.');
  }

  destroy() {
    this.worker.terminate();
    // Reject all pending requests
    for (const [id, pending] of this.pendingRequests) {
      pending.reject(new Error('WASM Worker terminated'));
      this.pendingRequests.delete(id);
    }
  }
}