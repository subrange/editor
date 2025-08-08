#!/bin/bash

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM module
echo "Building WASM module..."
wasm-pack build --target web --out-dir pkg

echo "WASM build complete! Output in ./pkg/"
echo ""
echo "To use in JavaScript:"
echo "  import init, { BrainfuckInterpreter } from './pkg/bf_wasm.js';"
echo "  await init();"
echo "  const interpreter = new BrainfuckInterpreter();"
echo "  const result = interpreter.run_program('++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.', new Uint8Array());"
echo "  console.log(result);"