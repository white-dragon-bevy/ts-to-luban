#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

const isWindows = process.platform === 'win32';
const binaryName = isWindows ? 'luban-gen.exe' : 'luban-gen';
const binaryPath = path.join(__dirname, binaryName);

async function ensureBinary() {
  if (fs.existsSync(binaryPath)) {
    return;
  }

  console.error('Binary not found, downloading...');
  const installScript = path.join(__dirname, '..', 'scripts', 'install.js');

  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [installScript], {
      stdio: 'inherit',
      cwd: path.join(__dirname, '..'),
    });

    child.on('close', (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Install failed with code ${code}`));
      }
    });
  });
}

async function main() {
  try {
    await ensureBinary();

    const child = spawn(binaryPath, process.argv.slice(2), {
      stdio: 'inherit',
    });

    child.on('close', (code) => {
      process.exit(code);
    });
  } catch (err) {
    console.error('Failed to run luban-gen:', err.message);
    process.exit(1);
  }
}

main();
