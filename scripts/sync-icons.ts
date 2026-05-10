#!/usr/bin/env bun
import { existsSync, mkdirSync, readdirSync, rmSync, copyFileSync } from 'node:fs';
import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, '..');
const src = join(repoRoot, 'icons', 'web');
const dst = join(repoRoot, 'frontend', 'public', 'icons');

const COPY = [
  'apple-touch-icon.png',
  'favicon.ico',
  'icon-192.png',
  'icon-192-maskable.png',
  'icon-512.png',
  'icon-512-maskable.png'
];

if (!existsSync(src)) {
  console.error(`sync-icons: source dir not found: ${src}`);
  process.exit(1);
}

mkdirSync(dst, { recursive: true });

for (const entry of readdirSync(dst)) {
  if (entry.endsWith('.png') || entry.endsWith('.ico')) {
    rmSync(join(dst, entry));
  }
}

for (const file of COPY) {
  const from = join(src, file);
  const to = join(dst, file);
  if (!existsSync(from)) {
    console.error(`sync-icons: missing source file: ${from}`);
    process.exit(1);
  }
  copyFileSync(from, to);
  console.log(`copied ${file}`);
}

console.log(`\nsynced ${COPY.length} icons -> ${dst}`);
