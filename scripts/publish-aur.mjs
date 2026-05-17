import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { spawnSync } from 'node:child_process';
import { AUR_PACKAGE_NAME, writeAurPackage } from './lib/aur-package.mjs';

const DEFAULT_OWNER = 'Aspenini';
const DEFAULT_REPO = 'Memory-Pak';
const DEFAULT_ASSET = 'Memory-Pak-linux-x86_64-portable.tar.gz';

const { args, flags } = parseArgs(process.argv.slice(2));
const aurDir = resolve(args.get('--aur-dir') || `../${AUR_PACKAGE_NAME}`);

if (flags.has('--publish')) {
  publishAur(aurDir);
} else {
  await stageAur(aurDir);
}

async function stageAur(targetDir) {
  const owner = args.get('--owner') || DEFAULT_OWNER;
  const repo = args.get('--repo') || DEFAULT_REPO;
  const assetName = args.get('--asset') || DEFAULT_ASSET;
  const release = await resolveRelease(owner, repo);
  const latestJson = await resolveLatestJson(release);
  const version = latestJson.version;

  if (!version || typeof version !== 'string') {
    throw new Error('latest.json did not contain a string "version" field.');
  }

  const tarballAsset = findAsset(release, assetName);
  const sourceUrl = args.get('--url') || tarballAsset?.browser_download_url;
  if (!sourceUrl) {
    throw new Error(`Release ${release.tag_name} does not have ${assetName}.`);
  }

  const sourceHash =
    args.get('--sha256') ||
    (args.has('--tarball') ? sha256File(resolve(args.get('--tarball'))) : await sha256Remote(sourceUrl));

  ensureAurCheckout(targetDir);
  writeAurPackage({ aurDir: targetDir, version, sourceUrl, sourceHash, includeReadme: false });

  if (flags.has('--verify') || flags.has('--install')) {
    run('makepkg', [flags.has('--install') ? '-si' : '--verifysource'], targetDir);
  }

  run('git', ['add', 'PKGBUILD', '.SRCINFO', `${AUR_PACKAGE_NAME}.install`], targetDir);

  console.log(`AUR files staged in ${targetDir}`);
  console.log(`Version came from latest.json in ${release.tag_name}: ${version}`);
  console.log('Review them, then publish when ready:');
  console.log(`  bun run aur:publish -- --aur-dir ${targetDir}`);
}

function publishAur(targetDir) {
  if (!existsSync(resolve(targetDir, '.git'))) {
    throw new Error(`${targetDir} is not an AUR git checkout. Run aur:stage first.`);
  }

  const diff = spawnSync('git', ['diff', '--cached', '--quiet'], { cwd: targetDir });
  if (diff.status === 0) {
    throw new Error('No staged AUR changes found. Run aur:stage first, then aur:publish.');
  }

  const version = readPkgbuildVersion(targetDir);
  const firstSubmission = spawnSync('git', ['rev-parse', '--verify', 'HEAD'], {
    cwd: targetDir,
    stdio: 'ignore'
  }).status !== 0;
  const message =
    args.get('--message') ||
    (firstSubmission ? `Add ${AUR_PACKAGE_NAME} ${version}` : `Update to ${version}`);

  run('git', ['commit', '-m', message], targetDir);
  run('git', ['push', 'origin', 'HEAD:master'], targetDir);
}

async function resolveRelease(owner, repo) {
  if (args.has('--release-file')) {
    return JSON.parse(readFileSync(resolve(args.get('--release-file')), 'utf8'));
  }

  const tag = args.get('--tag');
  const path = tag ? `releases/tags/${encodeURIComponent(tag)}` : 'releases/latest';
  return fetchJson(`https://api.github.com/repos/${owner}/${repo}/${path}`);
}

async function resolveLatestJson(release) {
  if (args.has('--latest-json-file')) {
    return JSON.parse(readFileSync(resolve(args.get('--latest-json-file')), 'utf8'));
  }

  const latestAsset = findAsset(release, 'latest.json');
  if (!latestAsset?.browser_download_url) {
    throw new Error(`Release ${release.tag_name} does not have latest.json.`);
  }
  return fetchJson(latestAsset.browser_download_url);
}

function findAsset(release, name) {
  return release.assets?.find((asset) => asset?.name === name);
}

async function fetchJson(url) {
  const response = await fetch(url, {
    headers: {
      Accept: 'application/vnd.github+json',
      'User-Agent': 'memory-pak-aur-publisher'
    }
  });
  if (!response.ok) {
    throw new Error(`Could not fetch ${url}: HTTP ${response.status}`);
  }
  return response.json();
}

function ensureAurCheckout(targetDir) {
  if (existsSync(resolve(targetDir, '.git'))) return;
  if (existsSync(targetDir)) {
    throw new Error(`${targetDir} exists but is not a git checkout.`);
  }

  mkdirSync(dirname(targetDir), { recursive: true });
  run(
    'git',
    ['clone', `ssh://aur@aur.archlinux.org/${AUR_PACKAGE_NAME}.git`, targetDir],
    dirname(targetDir)
  );
}

function sha256File(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

async function sha256Remote(url) {
  const tempDir = mkdtempSync(resolve(tmpdir(), 'memory-pak-aur-'));
  const tempFile = resolve(tempDir, DEFAULT_ASSET);
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Could not download ${url}: HTTP ${response.status}`);
    }
    const bytes = Buffer.from(await response.arrayBuffer());
    writeFileSync(tempFile, bytes);
    return createHash('sha256').update(bytes).digest('hex');
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

function readPkgbuildVersion(targetDir) {
  const pkgbuild = readFileSync(resolve(targetDir, 'PKGBUILD'), 'utf8');
  const match = pkgbuild.match(/^pkgver=(.+)$/m);
  if (!match) throw new Error('Could not read pkgver from PKGBUILD.');
  return match[1].trim();
}

function parseArgs(argv) {
  const parsedArgs = new Map();
  const parsedFlags = new Set();
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith('--')) {
      throw new Error(`Unexpected argument: ${arg}`);
    }
    const next = argv[i + 1];
    if (next && !next.startsWith('--')) {
      parsedArgs.set(arg, next);
      i += 1;
    } else {
      parsedFlags.add(arg);
    }
  }
  return { args: parsedArgs, flags: parsedFlags };
}

function run(command, commandArgs, cwd) {
  const result = spawnSync(command, commandArgs, {
    cwd,
    stdio: 'inherit'
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} ${commandArgs.join(' ')} failed with exit code ${result.status}`);
  }
}
