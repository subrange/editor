import type { ITokenizer } from '../editor-manager.service';
import { TokenizerWorkerClient } from './tokenizer-worker-client';
import { FastTokenizer } from '../../components/editor/tokenizer.fast';
import type { Token } from './tokenizer-worker-client';
import { BehaviorSubject } from 'rxjs';

/**
 * Worker-based tokenizer that provides async tokenization with caching
 * Falls back to synchronous tokenization for immediate results while worker processes
 */
export class WorkerTokenizer implements ITokenizer {
  private workerClient: TokenizerWorkerClient;
  private fallbackTokenizer: FastTokenizer;
  private tokenCache = new Map<string, Token[]>();
  private allLinesCache: { lines: string[], result: Token[][] } | null = null;
  private updateSubject = new BehaviorSubject<number>(0);
  
  // Track pending tokenizations to avoid duplicate requests
  private pendingLineTokenizations = new Map<string, Promise<Token[]>>();
  private pendingAllLinesTokenization: Promise<Token[][]> | null = null;

  private onUpdate?: () => void;
  
  constructor(onUpdate?: () => void) {
    this.onUpdate = onUpdate;
    this.workerClient = new TokenizerWorkerClient();
    this.fallbackTokenizer = new FastTokenizer();
  }

  reset(): void {
    this.tokenCache.clear();
    this.allLinesCache = null;
    this.pendingLineTokenizations.clear();
    this.pendingAllLinesTokenization = null;
    this.fallbackTokenizer.reset();
  }

  tokenizeLine(text: string, lineIndex: number, isLastLine?: boolean): Token[] {
    const cacheKey = `${text}:${lineIndex}:${isLastLine}`;
    
    // Return cached result if available
    if (this.tokenCache.has(cacheKey)) {
      return this.tokenCache.get(cacheKey)!;
    }

    // Start async tokenization if not already pending
    if (!this.pendingLineTokenizations.has(cacheKey)) {
      const promise = this.workerClient.tokenizeLine(text, lineIndex, isLastLine ?? false)
        .then(tokens => {
          this.tokenCache.set(cacheKey, tokens);
          this.pendingLineTokenizations.delete(cacheKey);
          
          // Limit cache size
          if (this.tokenCache.size > 1000) {
            const firstKey = this.tokenCache.keys().next().value;
            this.tokenCache.delete(firstKey);
          }
          
          // Trigger update
          if (this.onUpdate) {
            this.onUpdate();
          }
          this.updateSubject.next(this.updateSubject.value + 1);
          
          return tokens;
        })
        .catch(err => {
          console.error('Worker tokenization failed:', err);
          this.pendingLineTokenizations.delete(cacheKey);
          // Fall back to sync tokenizer on error
          const tokens = this.fallbackTokenizer.tokenizeLine(text, lineIndex, isLastLine ?? false);
          this.tokenCache.set(cacheKey, tokens);
          return tokens;
        });
      
      this.pendingLineTokenizations.set(cacheKey, promise);
    }

    // Return fallback tokenization for immediate display
    return this.fallbackTokenizer.tokenizeLine(text, lineIndex, isLastLine);
  }

  tokenizeAllLines(lines: string[]): Token[][] {
    // Check cache first
    if (this.allLinesCache && 
        lines.length === this.allLinesCache.lines.length &&
        lines.every((line, i) => line === this.allLinesCache!.lines[i])) {
      return this.allLinesCache.result;
    }

    // Start async tokenization if not already pending
    if (!this.pendingAllLinesTokenization) {
      this.pendingAllLinesTokenization = this.workerClient.tokenizeAllLines(lines)
        .then(result => {
          this.allLinesCache = { lines: [...lines], result };
          this.pendingAllLinesTokenization = null;
          
          // Trigger update
          if (this.onUpdate) {
            this.onUpdate();
          }
          this.updateSubject.next(this.updateSubject.value + 1);
          
          return result;
        })
        .catch(err => {
          console.error('Worker tokenization failed:', err);
          this.pendingAllLinesTokenization = null;
          // Fall back to sync tokenizer
          const result = this.fallbackTokenizer.tokenizeAllLines(lines);
          this.allLinesCache = { lines: [...lines], result };
          return result;
        });
    }

    // Return fallback tokenization for immediate display
    return this.fallbackTokenizer.tokenizeAllLines(lines);
  }

  // Observable for updates
  get updates$() {
    return this.updateSubject.asObservable();
  }

  destroy() {
    this.workerClient.destroy();
    this.updateSubject.complete();
  }
}