import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const pubkey = process.env.TAURI_UPDATER_PUBKEY?.trim();
const endpoint =
  process.env.TAURI_UPDATER_ENDPOINT?.trim() ||
  'https://github.com/Aspenini/Memory-Pak/releases/latest/download/latest.json';
const output = resolve(process.argv[2] || 'target/tauri-updater.conf.json');

if (!pubkey) {
  throw new Error('TAURI_UPDATER_PUBKEY is required to build updater-enabled bundles.');
}

const config = {
  bundle: {
    createUpdaterArtifacts: true
  },
  plugins: {
    updater: {
      pubkey,
      endpoints: [endpoint],
      windows: {
        installMode: 'passive'
      }
    }
  }
};

mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, `${JSON.stringify(config, null, 2)}\n`);
console.log(output);
