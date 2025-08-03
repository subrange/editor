import { MacroExpanderWorkerClient } from './macro-expander-worker-client';

// Factory function to create async macro expander using web worker
export function createAsyncMacroExpander(): MacroExpanderWorkerClient {
  return new MacroExpanderWorkerClient();
}