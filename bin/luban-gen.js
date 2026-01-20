#!/usr/bin/env node
const path = require('path');
const { spawnSync } = require('child_process');

const isWindows = process.platform === 'win32';
const binaryName = isWindows ? 'luban-gen.exe' : 'luban-gen';
const binaryPath = path.join(__dirname, binaryName);

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: isWindows,
});

if (result.error) {
  console.error('Failed to run luban-gen:', result.error.message);
  process.exit(1);
}

process.exit(result.status || 0);
