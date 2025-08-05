import { FastTokenizer, type Token } from '../../components/editor/services/tokenizer.fast.ts';

// Message types for communication with main thread
interface TokenizeLineMessage {
  type: 'tokenizeLine';
  id: string;
  text: string;
  lineIndex: number;
  isLastLine?: boolean;
}

interface TokenizeAllMessage {
  type: 'tokenizeAll';
  id: string;
  lines: string[];
}

interface ResultMessage {
  type: 'result';
  id: string;
  result: Token[] | Token[][];
}

interface ErrorMessage {
  type: 'error';
  id: string;
  error: string;
}

type WorkerMessage = TokenizeLineMessage | TokenizeAllMessage;

// Create a single instance of the tokenizer
const tokenizer = new FastTokenizer();

// Handle messages from main thread
self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  
  try {
    switch (message.type) {
      case 'tokenizeLine': {
        const result = tokenizer.tokenizeLine(
          message.text, 
          message.lineIndex, 
          message.isLastLine
        );
        const response: ResultMessage = {
          type: 'result',
          id: message.id,
          result
        };
        self.postMessage(response);
        break;
      }
      
      case 'tokenizeAll': {
        const result = tokenizer.tokenizeAllLines(message.lines);
        const response: ResultMessage = {
          type: 'result',
          id: message.id,
          result
        };
        self.postMessage(response);
        break;
      }
    }
  } catch (error) {
    const response: ErrorMessage = {
      type: 'error',
      id: message.id,
      error: error instanceof Error ? error.message : 'Unknown error'
    };
    self.postMessage(response);
  }
};