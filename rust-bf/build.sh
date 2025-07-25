#!/bin/bash

# Build the WebAssembly module
wasm-pack build --target web --out-dir ../src/wasm

# Optional: Optimize the wasm file with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    wasm-opt -O3 ../src/wasm/rust_bf_bg.wasm -o ../src/wasm/rust_bf_bg.wasm
fi

echo "WebAssembly build complete!"