#!/bin/bash

# Build all Ripple assembly examples

echo "Building Ripple assembly examples..."

# Create output directory
mkdir -p build/examples

# Build each example
for asm_file in src/services/ripple-assembler/examples/*.asm; do
    if [ -f "$asm_file" ]; then
        basename=$(basename "$asm_file" .asm)
        echo "Building $basename..."
        npx tsx src/services/ripple-assembler/cli.ts "$asm_file" -o "build/examples/$basename.bfm" -h "RippleVM Example"
    fi
done

echo "Build complete! Output files are in build/examples/"
echo ""
echo "To view the output:"
echo "  cat build/examples/hello-world.bfm"