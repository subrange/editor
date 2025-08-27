# Building the Electron App

## Prerequisites

- Node.js and npm installed
- Build tools for your platform (Xcode for macOS, Visual Studio for Windows, etc.)

## Setup

1. Install dependencies:
```bash
npm install
```

2. Generate app icons (if not already done):
```bash
npm run generate-app-icons
# On macOS, also run:
./scripts/create-icns.sh
```

## Development

Run the Electron app in development mode:
```bash
# Start Vite dev server in one terminal
npm run dev

# In another terminal, run Electron
npm run electron:dev
```

## Building

Build the web app first, then package with Electron:

### Build for current platform
```bash
npm run dist
```

### Build for specific platforms
```bash
# macOS
npm run dist:mac

# Windows  
npm run dist:win

# Linux
npm run dist:linux

# All platforms (requires appropriate build tools)
npm run dist:all
```

## Output

Built applications will be in the `electron-dist/` directory:
- **macOS**: `.dmg` and `.zip` files
- **Windows**: `.exe` installer and `.zip` 
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

## Configuration

- `electron-builder.yml` - Electron Builder configuration
- `electron/main.cjs` - Main Electron process
- `build-resources/` - Icons and platform-specific resources

## Troubleshooting

### macOS Code Signing
If you encounter code signing issues on macOS, the app is configured to run without signing for development. For distribution, you'll need an Apple Developer certificate.

### Linux Icons
Make sure you have generated all required icon sizes by running `npm run generate-app-icons`.

### Windows Icons
The `.ico` file is automatically generated. If you need to regenerate it, run `npm run generate-app-icons`.