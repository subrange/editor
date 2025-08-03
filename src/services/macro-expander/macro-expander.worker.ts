import { createMacroExpander, type MacroExpander, type MacroExpanderOptions, type MacroExpanderResult } from './macro-expander';

// Message types for communication with main thread
interface ExpandMessage {
  type: 'expand';
  id: string;
  input: string;
  options?: MacroExpanderOptions;
}

interface ResultMessage {
  type: 'result';
  id: string;
  result: MacroExpanderResult;
}

interface ErrorMessage {
  type: 'error';
  id: string;
  error: string;
}

type WorkerMessage = ExpandMessage;

// Create a single instance of the macro expander
const macroExpander: MacroExpander = createMacroExpander();

// Handle messages from main thread
self.onmessage = (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;
  
  switch (message.type) {
    case 'expand':
      try {
        const result = macroExpander.expand(message.input, message.options);
        const response: ResultMessage = {
          type: 'result',
          id: message.id,
          result
        };
        self.postMessage(response);
      } catch (error) {
        const response: ErrorMessage = {
          type: 'error',
          id: message.id,
          error: error instanceof Error ? error.message : 'Unknown error'
        };
        self.postMessage(response);
      }
      break;
  }
};