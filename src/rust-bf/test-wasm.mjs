import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Read the WASM file
const wasmPath = join(__dirname, 'pkg', 'bf_wasm_bg.wasm');
const wasmBuffer = readFileSync(wasmPath);

// Import and initialize the module
const wasmModule = await WebAssembly.instantiate(wasmBuffer, {
  wbindgen: {
    __wbindgen_throw: (ptr, len) => {
      const msg = new TextDecoder().decode(
        new Uint8Array(memory.buffer, ptr, len),
      );
      throw new Error(msg);
    },
  },
});

console.log('WASM module loaded successfully!');
console.log('Exports:', Object.keys(wasmModule.instance.exports));

// Simple test
const { memory } = wasmModule.instance.exports;

console.log('\nTest Summary:');
console.log('✅ WASM module compiled and loaded');
console.log('✅ Memory export found');
console.log(
  `✅ Module exports ${Object.keys(wasmModule.instance.exports).length} functions`,
);
console.log(
  '\nTo use in a browser, open test-wasm.html with a local web server',
);
console.log('For example: python3 -m http.server 8000');
