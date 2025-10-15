import { MacroExpanderWorkerClient } from './macro-expander-worker-client';
import { MacroExpanderWasmWorkerClient } from './macro-expander-wasm-worker-client';

export type { MacroExpanderWorkerClient, MacroExpanderWasmWorkerClient };

// Factory function to create async macro expander using web worker
export function createAsyncMacroExpander(): MacroExpanderWorkerClient {
  return new MacroExpanderWorkerClient();
}

// Factory function to create async WASM macro expander using web worker
export function createAsyncMacroExpanderWasm(): MacroExpanderWasmWorkerClient {
  return new MacroExpanderWasmWorkerClient();
}
