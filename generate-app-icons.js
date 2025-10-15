import { favicons } from 'favicons';
import fs from 'fs';
import path from 'path';

const source = 'src/favicon_huge.png'; // Source image

// Ensure build-resources directory exists
if (!fs.existsSync('build-resources')) {
  fs.mkdirSync('build-resources', { recursive: true });
}

// Configuration for app icons
const configuration = {
  path: '/',
  appName: 'Braintease IDE',
  appShortName: 'BF IDE',
  appDescription: 'A powerful IDE for Brainfuck and Ripple VM development',
  developerName: null,
  developerURL: null,
  background: '#2d3748', // Dark background to match the icon
  theme_color: '#4299e1', // Blue theme color
  appleStatusBarStyle: 'black-translucent',
  display: 'standalone',
  orientation: 'any',
  scope: '/',
  start_url: '/',
  version: '1.0',
  pixel_art: false,
  icons: {
    // Generate all icon types for Electron
    favicons: true,
    android: false,
    appleIcon: true, // Generate Apple icons for macOS
    appleStartup: false,
    windows: true, // Generate Windows icons
    yandex: false,
  },
};

async function generateIcons() {
  try {
    console.log('Generating app icons...');
    const response = await favicons(source, configuration);

    // Save favicon.ico to build-resources for Windows
    const faviconImage = response.images.find(
      (img) => img.name === 'favicon.ico',
    );
    if (faviconImage) {
      fs.writeFileSync('build-resources/icon.ico', faviconImage.contents);
      console.log('Created: build-resources/icon.ico');
    }

    // Generate icon.icns for macOS
    // For now, we'll use the largest PNG and let electron-builder convert it
    const largestAppleIcon = response.images
      .filter((img) => img.name.includes('apple-touch-icon'))
      .sort((a, b) => {
        const sizeA = parseInt(a.name.match(/\d+/)?.[0] || '0');
        const sizeB = parseInt(b.name.match(/\d+/)?.[0] || '0');
        return sizeB - sizeA;
      })[0];

    if (largestAppleIcon) {
      // Save as PNG for electron-builder to convert
      fs.writeFileSync('build-resources/icon.png', largestAppleIcon.contents);
      console.log(
        'Created: build-resources/icon.png (will be converted to .icns)',
      );
    }

    // Generate Linux icons (various PNG sizes)
    const sizes = [16, 32, 48, 64, 128, 256, 512, 1024];
    for (const size of sizes) {
      const iconName = `${size}x${size}.png`;
      // Find closest size from generated images
      const icon = response.images.find(
        (img) =>
          img.name.includes(`${size}x${size}`) ||
          img.name.includes(`icon-${size}`),
      );

      if (icon) {
        fs.writeFileSync(`build-resources/${iconName}`, icon.contents);
        console.log(`Created: build-resources/${iconName}`);
      }
    }

    // Also copy the favicon.ico to dist for the built app
    if (faviconImage && fs.existsSync('dist')) {
      fs.writeFileSync('dist/favicon.ico', faviconImage.contents);
      console.log('Created: dist/favicon.ico');
    }

    console.log('\nApp icons generated successfully!');
    console.log(
      '\nNote: You may need to install icns-creator globally to generate .icns files:',
    );
    console.log('  npm install -g png2icns');
    console.log(
      '  png2icns build-resources/icon.png -o build-resources/icon.icns',
    );
  } catch (error) {
    console.error('Error generating icons:', error.message);
    process.exit(1);
  }
}

generateIcons();
