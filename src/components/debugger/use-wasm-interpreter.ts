// Toggle for using WebAssembly interpreter
import { BehaviorSubject } from 'rxjs';

const USE_WASM_KEY = 'brainfuck-ide-use-wasm';

// Check if WASM is supported and user preference
const isWasmSupported = typeof WebAssembly !== 'undefined';
const storedPreference = localStorage.getItem(USE_WASM_KEY);
const defaultUseWasm = isWasmSupported && storedPreference !== 'false';

export const useWasmInterpreter = new BehaviorSubject<boolean>(defaultUseWasm);

// Save preference when changed
useWasmInterpreter.subscribe(value => {
    localStorage.setItem(USE_WASM_KEY, value.toString());
});

// Export helper to get the appropriate interpreter store
export async function getInterpreterStore() {
    if (useWasmInterpreter.getValue()) {
        try {
            const { wasmInterpreterStore } = await import('./interpreter-wasm.store');
            return wasmInterpreterStore;
        } catch (error) {
            console.error('Failed to load WASM interpreter, falling back to JS:', error);
            useWasmInterpreter.next(false);
        }
    }
    
    const { interpreterStore } = await import('./interpreter.store');
    return interpreterStore;
}