# Memory Pak AUR Maintenance

Memory Pak's Arch package is `memory-pak-bin`. It uses the prebuilt
`Memory-Pak-linux-x86_64-portable.tar.gz` release artifact and lets Arch users
update through their normal AUR helper or package-manager workflow. GitHub
Actions only stages the AUR files; it must not push to AUR automatically.

The `-bin` suffix is intentional because the package uses a prebuilt upstream
artifact while the project source is public.

## User Install

```bash
yay -S memory-pak-bin
yay -Syu
```

`paru`, `makepkg`, or any other AUR flow works too. There is no in-app updater on
Linux.

## First Submission

1. Create an account at <https://aur.archlinux.org/> and add your SSH public key.
2. Run the manual GitHub **Package Artifacts** workflow for the release tag.
3. Download and extract the `memory-pak-release-metadata` workflow artifact.
4. Clone the AUR package repository:

```bash
git clone ssh://aur@aur.archlinux.org/memory-pak-bin.git
```

5. Copy the staged files into the AUR checkout:

```bash
cp path/to/release-metadata/aur/PKGBUILD memory-pak-bin/
cp path/to/release-metadata/aur/.SRCINFO memory-pak-bin/
cp path/to/release-metadata/aur/memory-pak-bin.install memory-pak-bin/
```

6. Test locally on Arch:

```bash
cd memory-pak-bin
makepkg -si
```

7. Commit and push when you are ready:

```bash
git add PKGBUILD .SRCINFO memory-pak-bin.install
git commit -m "Add memory-pak-bin 0.3.0"
git push
```

## Local Script

The repo also has a local helper for the normal post-release flow. After you
publish a GitHub release and attach `latest.json` plus
`Memory-Pak-linux-x86_64-portable.tar.gz`, run:

```bash
bun run aur:stage -- --aur-dir ../memory-pak-bin
```

`aur:stage` fetches the latest GitHub release, downloads that release's
`latest.json`, reads its `version`, finds the Linux tarball in the same release,
checksums it, writes `PKGBUILD`, `.SRCINFO`, and `memory-pak-bin.install`, then
runs `git add`. It does not commit or push.

When you are ready:

```bash
bun run aur:publish -- --aur-dir ../memory-pak-bin
```

`aur:publish` commits the staged AUR files and pushes them.

Useful options:

```bash
# Stage from a specific release instead of GitHub's latest release
bun run aur:stage -- --tag v0.4.0

# Verify or install-test with makepkg after staging
bun run aur:stage -- --verify
bun run aur:stage -- --install

# Use a local tarball for checksum calculation while still writing the release URL
bun run aur:stage -- --tarball ./Memory-Pak-linux-x86_64-portable.tar.gz

# Override the commit message used by aur:publish
bun run aur:publish -- --message "Update to 0.4.0"
```

The helper publishes AUR metadata only. AUR does not host the Memory Pak binary;
the generated `PKGBUILD` downloads the public tarball from the GitHub release.

## Updating For A New Release

Run the manual package workflow with the new `version` and `tag`, then repeat
the copy/test/commit/push steps. Use a commit like:

```bash
git commit -m "Update to 0.4.0"
```

The staged `PKGBUILD` and `.SRCINFO` already contain the release tarball URL and
SHA-256 checksum. If you edit `PKGBUILD` by hand after staging, regenerate
`.SRCINFO` on Arch before pushing:

```bash
makepkg --printsrcinfo > .SRCINFO
```
