// Web Worker for running Brainfuck code using WASM interpreter
let wasmModule = null;
let BrainfuckInterpreter = null;
let StatefulBrainfuckInterpreter = null;
let currentInterpreter = null;

// Initialize WASM module
async function initWasm() {
    try {
        const wasmUrl = new URL('./wasm/bf_wasm_bg.wasm', self.location).href;
        const initModule = await import('./wasm/bf_wasm.js');
        
        wasmModule = await initModule.default(wasmUrl);
        BrainfuckInterpreter = initModule.BrainfuckInterpreter;
        StatefulBrainfuckInterpreter = initModule.StatefulBrainfuckInterpreter;
        
        self.postMessage({ type: 'ready' });
    } catch (error) {
        self.postMessage({ 
            type: 'error', 
            error: `Failed to initialize WASM: ${error.message}` 
        });
    }
}

// Handle messages from main thread
self.onmessage = async function(e) {
    const { type, id, code, input, options } = e.data;
    
    if (type === 'init') {
        await initWasm();
        return;
    }
    
    if (type === 'run') {
        if (!StatefulBrainfuckInterpreter) {
            self.postMessage({ 
                type: 'error', 
                id,
                error: 'WASM module not initialized' 
            });
            return;
        }
        
        try {
            // Default options
            const tapeSize = options?.tapeSize || 30000;
            const cellSize = options?.cellSize || 8;
            const wrap = options?.wrap !== false;
            const wrapTape = options?.wrapTape !== false;
            const optimize = options?.optimize !== false;
            
            // Create stateful interpreter that can pause for input
            currentInterpreter = new StatefulBrainfuckInterpreter(
                code,
                tapeSize, 
                cellSize, 
                wrap, 
                wrapTape, 
                optimize
            );
            
            // Create output callback that sends messages to main thread
            const outputCallback = (char, charCode) => {
                self.postMessage({
                    type: 'output',
                    id,
                    char,
                    charCode
                });
            };
            
            // Run until input is needed or program finishes
            const needsInput = currentInterpreter.run_until_input(outputCallback);
            
            if (needsInput && currentInterpreter.is_waiting_for_input()) {
                // Program is paused waiting for input
                self.postMessage({
                    type: 'waiting_for_input',
                    id
                });
            } else {
                // Program completed
                const result = currentInterpreter.get_state();
                
                self.postMessage({
                    type: 'complete',
                    id,
                    result: {
                        tape: result.tape,
                        pointer: result.pointer,
                        output: result.output,
                        tapeTruncated: result.tape_truncated || false,
                        originalTapeSize: result.original_tape_size || result.tape.length
                    }
                });
                
                currentInterpreter = null;
            }
            
        } catch (error) {
            self.postMessage({
                type: 'error',
                id,
                error: error.message || 'Unknown error occurred'
            });
            currentInterpreter = null;
        }
    }
    
    if (type === 'provide_input') {
        if (!currentInterpreter) {
            self.postMessage({
                type: 'error',
                id,
                error: 'No interpreter running'
            });
            return;
        }
        
        try {
            // Provide the input character
            const charCode = e.data.charCode;
            currentInterpreter.provide_input(charCode);
            
            // Create output callback
            const outputCallback = (char, charCode) => {
                self.postMessage({
                    type: 'output',
                    id,
                    char,
                    charCode
                });
            };
            
            // Continue execution
            const needsInput = currentInterpreter.run_until_input(outputCallback);
            
            if (needsInput && currentInterpreter.is_waiting_for_input()) {
                // Still needs more input
                self.postMessage({
                    type: 'waiting_for_input',
                    id
                });
            } else {
                // Program completed
                const result = currentInterpreter.get_state();
                
                self.postMessage({
                    type: 'complete',
                    id,
                    result: {
                        tape: result.tape,
                        pointer: result.pointer,
                        output: result.output,
                        tapeTruncated: result.tape_truncated || false,
                        originalTapeSize: result.original_tape_size || result.tape.length
                    }
                });
                
                currentInterpreter = null;
            }
        } catch (error) {
            self.postMessage({
                type: 'error',
                id,
                error: error.message || 'Failed to provide input'
            });
        }
    }
    
    if (type === 'optimize') {
        if (!BrainfuckInterpreter) {
            self.postMessage({ 
                type: 'error', 
                id,
                error: 'WASM module not initialized' 
            });
            return;
        }
        
        try {
            const interpreter = new BrainfuckInterpreter();
            const optimized = interpreter.optimize_brainfuck(code);
            
            self.postMessage({
                type: 'optimized',
                id,
                code: optimized
            });
        } catch (error) {
            self.postMessage({
                type: 'error',
                id,
                error: error.message || 'Optimization failed'
            });
        }
    }
};

// Auto-initialize on worker creation
initWasm();