#!/bin/bash

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module with the wasm feature enabled
echo "Building WASM module..."
RUSTFLAGS='-C target-feature=+bulk-memory' wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    --features wasm \
    --no-default-features

echo "WASM build complete! Output in ./pkg/"
echo ""
echo "To use in JavaScript:"
echo "  import init, { WasmAssembler, WasmLinker, WasmFormatter } from './pkg/ripple_asm.js';"
echo "  await init();"
echo "  const assembler = new WasmAssembler();"
echo "  const result = assembler.assemble('LI R3, 42\\nHALT');"
echo "  console.log(result);"