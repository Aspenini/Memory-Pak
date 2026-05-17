import { chmodSync, cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { basename, dirname, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

const binary = resolve(process.argv[2] || 'target/release/memory-pak');
const out = resolve(process.argv[3] || 'Memory-Pak-linux-x86_64-portable.tar.gz');
const root = resolve(process.argv[4] || 'Memory-Pak-linux-x86_64');

if (!existsSync(binary)) {
  throw new Error(`Linux binary not found: ${binary}`);
}

rmSync(root, { recursive: true, force: true });
rmSync(out, { force: true });
mkdirSync(root, { recursive: true });

cpSync(binary, `${root}/memory-pak`);
chmodSync(`${root}/memory-pak`, 0o755);
cpSync('README.md', `${root}/README.md`);
cpSync('LICENSE', `${root}/LICENSE`);
cpSync('icons/linux/AppIcon.png', `${root}/memory-pak.png`);
writeFileSync(
  `${root}/README-linux-portable.txt`,
  `Memory Pak Linux portable build

This archive runs the Memory Pak binary directly and uses your system WebKitGTK.
It is the recommended manual download for Linux users who do not use a distro
package, and it is also the source artifact for the memory-pak-bin AUR package.

Arch dependencies:
  sudo pacman -S webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg

Run:
  ./memory-pak
`
);

const tar = spawnSync('tar', ['-czf', out, basename(root)], {
  cwd: dirname(root),
  stdio: 'inherit'
});
if (tar.error) throw tar.error;
if (tar.status !== 0) {
  throw new Error(`tar failed with exit code ${tar.status}`);
}

console.log(`Created ${out}`);
