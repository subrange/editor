#!/bin/bash

# Script to copy Electron builds to public/downloads for web distribution

echo "Publishing desktop app downloads..."

# Create downloads directory if it doesn't exist
mkdir -p public/downloads

# Copy macOS build
if [ -f "electron-dist/Braintease IDE-0.0.0-arm64.dmg" ]; then
    cp "electron-dist/Braintease IDE-0.0.0-arm64.dmg" "public/downloads/Braintease-IDE-mac.dmg"
    echo "✓ Published macOS build"
elif [ -f "electron-dist/Braintease IDE-0.0.0.dmg" ]; then
    cp "electron-dist/Braintease IDE-0.0.0.dmg" "public/downloads/Braintease-IDE-mac.dmg"
    echo "✓ Published macOS build"
else
    echo "⚠ macOS build not found"
fi

# Copy Windows build
if [ -f "electron-dist/Braintease IDE Setup 0.0.0.exe" ]; then
    cp "electron-dist/Braintease IDE Setup 0.0.0.exe" "public/downloads/Braintease-IDE-win.exe"
    echo "✓ Published Windows build"
else
    echo "⚠ Windows build not found"
fi

# Copy Linux AppImage
if [ -f "electron-dist/Braintease IDE-0.0.0.AppImage" ]; then
    cp "electron-dist/Braintease IDE-0.0.0.AppImage" "public/downloads/Braintease-IDE-linux.AppImage"
    echo "✓ Published Linux build"
else
    echo "⚠ Linux build not found"
fi

# Generate a simple downloads page
cat > public/downloads/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Braintease IDE - Downloads</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #0f172a;
            color: #cbd5e1;
            padding: 2rem;
            max-width: 800px;
            margin: 0 auto;
        }
        h1 { color: #60a5fa; }
        .download-section {
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 1.5rem;
            margin: 1rem 0;
        }
        .download-btn {
            display: inline-block;
            background: #2563eb;
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 6px;
            text-decoration: none;
            margin: 0.5rem 0;
        }
        .download-btn:hover {
            background: #1d4ed8;
        }
        .platform { 
            font-size: 1.25rem;
            font-weight: 600;
            margin-bottom: 0.5rem;
        }
        .requirements {
            font-size: 0.875rem;
            color: #94a3b8;
            margin-top: 0.5rem;
        }
    </style>
</head>
<body>
    <h1>Braintease IDE Desktop Downloads</h1>
    
    <div class="download-section">
        <div class="platform">🍎 macOS</div>
        <a href="Braintease-IDE-mac.dmg" class="download-btn" download>Download for macOS (.dmg)</a>
        <div class="requirements">Requires macOS 10.12 or later</div>
    </div>
    
    <div class="download-section">
        <div class="platform">🪟 Windows</div>
        <a href="Braintease-IDE-win.exe" class="download-btn" download>Download for Windows (.exe)</a>
        <div class="requirements">Requires Windows 10 or later</div>
    </div>
    
    <div class="download-section">
        <div class="platform">🐧 Linux</div>
        <a href="Braintease-IDE-linux.AppImage" class="download-btn" download>Download for Linux (.AppImage)</a>
        <div class="requirements">AppImage - works on most Linux distributions</div>
    </div>
    
    <p style="margin-top: 2rem; text-align: center;">
        <a href="/" style="color: #60a5fa;">← Back to Web IDE</a>
    </p>
</body>
</html>
EOF

echo "✓ Generated downloads page"
echo ""
echo "Downloads published to public/downloads/"
echo "These will be available at /downloads/ when the web app is deployed"