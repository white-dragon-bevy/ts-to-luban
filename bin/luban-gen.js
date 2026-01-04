#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const isWindows = process.platform === 'win32';
const binaryName = isWindows ? 'luban-gen.exe' : 'luban-gen';
const binaryPath = path.join(__dirname, binaryName);

function ensureBinary() {
  if (fs.existsSync(binaryPath)) {
    return true;
  }

  console.error('Binary not found, downloading...');
  const installScript = path.join(__dirname, '..', 'scripts', 'install.js');

  const result = spawnSync(process.execPath, [installScript], {
    stdio: 'inherit',
    cwd: path.join(__dirname, '..'),
  });

  if (result.status !== 0) {
    console.error('Failed to download binary');
    return false;
  }
  return true;
}

function main() {
  if (!ensureBinary()) {
    process.exit(1);
  }

  const result = spawnSync(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    shell: isWindows,
  });

  if (result.error) {
    console.error('Failed to run luban-gen:', result.error.message);
    process.exit(1);
  }

  process.exit(result.status || 0);
}

main();
