// Web Worker for running Brainfuck code using WASM interpreter
let wasmModule = null;
let BrainfuckInterpreter = null;

// Initialize WASM module
async function initWasm() {
  try {
    const wasmUrl = new URL('./wasm/bf_wasm_bg.wasm', self.location).href;
    const initModule = await import('./wasm/bf_wasm.js');

    wasmModule = await initModule.default(wasmUrl);
    BrainfuckInterpreter = initModule.BrainfuckInterpreter;

    self.postMessage({ type: 'ready' });
  } catch (error) {
    self.postMessage({
      type: 'error',
      error: `Failed to initialize WASM: ${error.message}`,
    });
  }
}

// Handle messages from main thread
self.onmessage = async function (e) {
  const { type, id, code, input, options } = e.data;

  if (type === 'init') {
    await initWasm();
    return;
  }

  if (type === 'run') {
    if (!BrainfuckInterpreter) {
      self.postMessage({
        type: 'error',
        id,
        error: 'WASM module not initialized',
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

      // Create interpreter
      const interpreter = BrainfuckInterpreter.with_options(
        tapeSize,
        cellSize,
        wrap,
        wrapTape,
        optimize,
      );

      // Convert input string to Uint8Array
      const inputBytes = input
        ? new TextEncoder().encode(input)
        : new Uint8Array();

      // Create output callback that sends messages to main thread
      const outputCallback = (char, charCode) => {
        self.postMessage({
          type: 'output',
          id,
          char,
          charCode,
        });
      };

      // Run the program with callback
      const result = interpreter.run_program_with_callback(
        code,
        inputBytes,
        outputCallback,
      );

      // Send completion message with final state
      // The tape is already truncated by the Rust code if needed
      self.postMessage({
        type: 'complete',
        id,
        result: {
          tape: result.tape,
          pointer: result.pointer,
          output: result.output,
          tapeTruncated: result.tape_truncated || false,
          originalTapeSize: result.original_tape_size || result.tape.length,
        },
      });
    } catch (error) {
      self.postMessage({
        type: 'error',
        id,
        error: error.message || 'Unknown error occurred',
      });
    }
  }

  if (type === 'optimize') {
    if (!BrainfuckInterpreter) {
      self.postMessage({
        type: 'error',
        id,
        error: 'WASM module not initialized',
      });
      return;
    }

    try {
      const interpreter = new BrainfuckInterpreter();
      const optimized = interpreter.optimize_brainfuck(code);

      self.postMessage({
        type: 'optimized',
        id,
        code: optimized,
      });
    } catch (error) {
      self.postMessage({
        type: 'error',
        id,
        error: error.message || 'Optimization failed',
      });
    }
  }
};

// Auto-initialize on worker creation
initWasm();
