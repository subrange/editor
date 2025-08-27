#!/bin/bash

# Script to create .ico file for Windows from PNG with multiple sizes
# Uses ImageMagick's convert command

SOURCE_PNG="src/favicon_huge.png"
OUTPUT_ICO="build-resources/icon.ico"

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "ImageMagick is required but not installed. Please install it:"
    echo "  brew install imagemagick"
    exit 1
fi

# Create build-resources directory if it doesn't exist
mkdir -p build-resources

echo "Creating Windows .ico file with multiple sizes..."

# Convert PNG to ICO with multiple sizes (16, 32, 48, 64, 128, 256)
convert "$SOURCE_PNG" \
    -resize 256x256 \
    -define icon:auto-resize=256,128,64,48,32,16 \
    "$OUTPUT_ICO"

echo "Created $OUTPUT_ICO with sizes: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256"

# Also create individual PNG files for electron-builder
echo "Creating individual PNG files..."
convert "$SOURCE_PNG" -resize 256x256 "build-resources/256x256.png"
convert "$SOURCE_PNG" -resize 512x512 "build-resources/512x512.png"
convert "$SOURCE_PNG" -resize 1024x1024 "build-resources/1024x1024.png"

echo "Icon generation complete!"