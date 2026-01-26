#!/usr/bin/env node
const path = require('path');
const { spawnSync } = require('child_process');

const platform = process.platform;
const arch = process.arch;

let binaryName;
if (platform === 'win32') {
  binaryName = 'luban-gen.exe';
} else if (platform === 'darwin') {
  binaryName = arch === 'arm64' ? 'luban-gen-darwin-arm64' : 'luban-gen-darwin-x64';
} else {
  binaryName = 'luban-gen-linux';
}

const binaryPath = path.join(__dirname, binaryName);

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: platform === 'win32',
});

if (result.error) {
  console.error('Failed to run luban-gen:', result.error.message);
  process.exit(1);
}

process.exit(result.status || 0);
