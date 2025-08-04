export interface Token {
    type: 'incdec' | 'brackets' | 'move' | 'dot' | 'comma' | 'whitespace' | 'comment' | 'unknown'
    value: string;
    start: number;
    end: number;
}

interface PendingRequest {
  resolve: (result: Token[] | Token[][]) => void;
  reject: (error: Error) => void;
}

export class TokenizerWorkerClient {
  private worker: Worker;
  private pendingRequests = new Map<string, PendingRequest>();
  private requestId = 0;

  constructor() {
    // Import worker with Vite's special syntax
    this.worker = new Worker(
      new URL('./tokenizer.worker.ts', import.meta.url),
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
      console.error('Tokenizer worker error:', error);
      // Reject all pending requests
      for (const [id, pending] of this.pendingRequests) {
        pending.reject(new Error('Worker error'));
        this.pendingRequests.delete(id);
      }
    };
  }

  async tokenizeLine(
    text: string, 
    lineIndex: number, 
    isLastLine: boolean = false
  ): Promise<Token[]> {
    const id = String(this.requestId++);
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { 
        resolve: (result) => resolve(result as Token[]), 
        reject 
      });
      
      this.worker.postMessage({
        type: 'tokenizeLine',
        id,
        text,
        lineIndex,
        isLastLine
      });
    });
  }

  async tokenizeAllLines(lines: string[]): Promise<Token[][]> {
    const id = String(this.requestId++);
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { 
        resolve: (result) => resolve(result as Token[][]), 
        reject 
      });
      
      this.worker.postMessage({
        type: 'tokenizeAll',
        id,
        lines
      });
    });
  }

  destroy() {
    this.worker.terminate();
    // Reject all pending requests
    for (const [id, pending] of this.pendingRequests) {
      pending.reject(new Error('Worker terminated'));
      this.pendingRequests.delete(id);
    }
  }
}

// Export token styles from the original tokenizer
export { tokenStyles } from '../../components/editor/tokenizer.fast';