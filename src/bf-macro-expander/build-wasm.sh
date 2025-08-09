#!/bin/bash
set -e

echo "Building WASM module for bf-macro-expander..."

# Build with wasm-pack
wasm-pack build --target web --features wasm --no-default-features

# Optional: Optimize the WASM file with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -O3 pkg/bf_macro_expander_bg.wasm -o pkg/bf_macro_expander_bg.wasm
fi

echo "WASM build complete! Output in pkg/"