#!/bin/bash

# Script to create .icns file for macOS and .ico for Windows from PNG
# Requires iconutil (comes with macOS)

SOURCE_PNG="src/favicon_huge.png"
ICONSET_DIR="build-resources/icon.iconset"
OUTPUT_ICNS="build-resources/icon.icns"

# Create iconset directory
mkdir -p "$ICONSET_DIR"

# Generate different sizes required for .icns
sips -z 16 16     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16.png"
sips -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png"
sips -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32.png"
sips -z 64 64     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png"
sips -z 128 128   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128.png"
sips -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png"
sips -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256.png"
sips -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png"
sips -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512.png"
sips -z 1024 1024 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512@2x.png"

# Create .icns file
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# Create a 256x256 PNG for Windows (electron-builder will convert to .ico)
echo "Creating icon.png for Windows..."
sips -z 256 256 "$SOURCE_PNG" --out "build-resources/icon.png"

# Also create the various sizes that might be needed
sips -z 512 512 "$SOURCE_PNG" --out "build-resources/512x512.png"
sips -z 256 256 "$SOURCE_PNG" --out "build-resources/256x256.png"
sips -z 128 128 "$SOURCE_PNG" --out "build-resources/128x128.png"
sips -z 64 64   "$SOURCE_PNG" --out "build-resources/64x64.png"
sips -z 48 48   "$SOURCE_PNG" --out "build-resources/48x48.png"
sips -z 32 32   "$SOURCE_PNG" --out "build-resources/32x32.png"
sips -z 24 24   "$SOURCE_PNG" --out "build-resources/24x24.png"
sips -z 16 16   "$SOURCE_PNG" --out "build-resources/16x16.png"

# Note: electron-builder will automatically convert icon.png to icon.ico
echo "Note: Use icon.png for Windows builds - electron-builder will convert it to .ico"

# Clean up
rm -rf "$ICONSET_DIR"

echo "Created $OUTPUT_ICNS and icon.png (for Windows)"