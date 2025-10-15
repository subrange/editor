import {
  createMacroExpanderRust,
  initializeWasm,
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

// Initialize WASM and create expander
let macroExpander: MacroExpander | null = null;
let initPromise: Promise<void> | null = null;

async function ensureInitialized(): Promise<MacroExpander> {
  if (!initPromise) {
    initPromise = initializeWasm().then(() => {
      macroExpander = createMacroExpanderRust();
      console.log('WASM macro expander initialized in worker');
    });
  }

  await initPromise;

  if (!macroExpander) {
    throw new Error('Failed to initialize WASM macro expander');
  }

  return macroExpander;
}

// Handle messages from main thread
self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
  const message = event.data;

  switch (message.type) {
    case 'expand':
      try {
        const expander = await ensureInitialized();
        const result = expander.expand(message.input, message.options);
        const response: ResultMessage = {
          type: 'result',
          id: message.id,
          result,
        };
        self.postMessage(response);
      } catch (error) {
        console.error('Macro expansion error in WASM worker:', error);
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
