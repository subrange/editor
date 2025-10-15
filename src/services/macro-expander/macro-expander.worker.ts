import {
  createMacroExpanderV3,
  type MacroExpander,
  type MacroExpanderOptions,
  type MacroExpanderResult,
} from './macro-expander';

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

// Create a single instance of the macro expander using V3 with source map support
const macroExpander: MacroExpander = createMacroExpanderV3();

// Log that we're using the updated expander with source map fixes
console.log('Macro expander worker started with source map generation fixes');

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
          result,
        };
        self.postMessage(response);
      } catch (error) {
        console.error('Macro expansion error in worker:', error);
        const errorMessage =
          error instanceof Error
            ? `${error.message}\n${error.stack}`
            : 'Unknown error';
        const response: ErrorMessage = {
          type: 'error',
          id: message.id,
          error: errorMessage,
        };
        self.postMessage(response);
      }
      break;
  }
};
