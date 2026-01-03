#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const PACKAGE_JSON = require('../package.json');
const VERSION = PACKAGE_JSON.version;
const REPO = 'white-dragon-bevy/ts-to-luban';

const PLATFORM_MAP = {
  'linux-x64': { asset: 'luban-gen-linux-x64', binary: 'luban-gen' },
  'darwin-x64': { asset: 'luban-gen-darwin-x64', binary: 'luban-gen' },
  'darwin-arm64': { asset: 'luban-gen-darwin-arm64', binary: 'luban-gen' },
  'win32-x64': { asset: 'luban-gen-win32-x64.exe', binary: 'luban-gen.exe' },
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

function getDownloadUrl(platformKey) {
  const platformInfo = PLATFORM_MAP[platformKey];
  if (!platformInfo) {
    throw new Error(`Unsupported platform: ${platformKey}. Supported: ${Object.keys(PLATFORM_MAP).join(', ')}`);
  }
  return {
    url: `https://github.com/${REPO}/releases/download/v${VERSION}/${platformInfo.asset}`,
    binary: platformInfo.binary,
  };
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    const request = (url) => {
      https.get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Download failed: ${response.statusCode} ${response.statusMessage}`));
          return;
        }

        response.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };

    request(url);
  });
}

async function main() {
  const platformKey = getPlatformKey();
  console.log(`Installing luban-gen for ${platformKey}...`);

  const { url, binary } = getDownloadUrl(platformKey);
  const binDir = path.join(__dirname, '..', 'bin');
  const destPath = path.join(binDir, process.platform === 'win32' ? 'luban-gen.exe' : 'luban-gen');

  // Create bin directory
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  console.log(`Downloading from ${url}...`);

  try {
    await download(url, destPath);

    // Make executable on Unix
    if (process.platform !== 'win32') {
      fs.chmodSync(destPath, 0o755);
    }

    console.log('luban-gen installed successfully!');
  } catch (err) {
    console.error(`Failed to download luban-gen: ${err.message}`);
    console.error('You may need to build from source: cargo build --release');
    process.exit(1);
  }
}

main();
