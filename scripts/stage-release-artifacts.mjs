import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  args.set(process.argv[i], process.argv[i + 1]);
}

const version = args.get('--version');
const tag = args.get('--tag');
const input = resolve(args.get('--input') || 'release-staging');
const output = resolve(args.get('--output') || join(input, 'metadata'));
const baseUrl =
  args.get('--base-url') ||
  (tag ? `https://github.com/Aspenini/Memory-Pak/releases/download/${tag}` : undefined);

if (!version) throw new Error('--version is required.');
if (!baseUrl) throw new Error('--tag or --base-url is required.');
if (!existsSync(input)) throw new Error(`Input directory does not exist: ${input}`);

const files = walk(input).filter((file) => !file.startsWith(output));
const platforms = {};

addUpdaterPlatform(platforms, 'windows-x86_64', findFirst(files, isWindowsUpdaterAsset), baseUrl);
addUpdaterPlatform(platforms, 'linux-x86_64', findFirst(files, isLinuxUpdaterAsset), baseUrl);

const macAsset = findFirst(files, isMacUpdaterAsset);
addUpdaterPlatform(platforms, 'darwin-x86_64', macAsset, baseUrl);
addUpdaterPlatform(platforms, 'darwin-aarch64', macAsset, baseUrl);

if (Object.keys(platforms).length === 0) {
  throw new Error('No signed updater artifacts were found.');
}

mkdirSync(output, { recursive: true });
writeFileSync(
  join(output, 'latest.json'),
  `${JSON.stringify(
    {
      version,
      notes: `Memory Pak ${version}`,
      pub_date: new Date().toISOString(),
      platforms
    },
    null,
    2
  )}\n`
);

writeFileSync(join(output, 'checksums.sha256'), checksums(files, output));

function walk(dir) {
  const entries = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      entries.push(...walk(path));
    } else if (stat.isFile()) {
      entries.push(path);
    }
  }
  return entries;
}

function findFirst(files, predicate) {
  return files.find((file) => predicate(file) && existsSync(`${file}.sig`));
}

function addUpdaterPlatform(target, key, asset, baseUrl) {
  if (!asset) return;
  target[key] = {
    signature: readFileSync(`${asset}.sig`, 'utf8').trim(),
    url: `${baseUrl}/${encodeURIComponent(basename(asset))}`
  };
}

function isWindowsUpdaterAsset(file) {
  const name = basename(file).toLowerCase();
  return name.endsWith('.exe') && name.includes('setup') && !name.includes('portable');
}

function isLinuxUpdaterAsset(file) {
  return basename(file).toLowerCase().endsWith('.appimage');
}

function isMacUpdaterAsset(file) {
  return basename(file).toLowerCase().endsWith('.app.tar.gz');
}

function checksums(files, outputDir) {
  return files
    .filter((file) => !file.startsWith(outputDir))
    .map((file) => {
      const hash = createHash('sha256').update(readFileSync(file)).digest('hex');
      return `${hash}  ${basename(file)}`;
    })
    .sort()
    .join('\n')
    .concat('\n');
}
