import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export const AUR_PACKAGE_NAME = 'memory-pak-bin';

export function writeAurPackage({ aurDir, version, sourceUrl, sourceHash, includeReadme = true }) {
  mkdirSync(aurDir, { recursive: true });

  writeFileSync(
    join(aurDir, 'PKGBUILD'),
    `# Maintainer: Aspenini
pkgname=${AUR_PACKAGE_NAME}
pkgver=${version}
pkgrel=1
pkgdesc="A cross-platform retro game collection tracker"
arch=('x86_64')
url="https://github.com/Aspenini/Memory-Pak"
license=('MIT')
depends=('desktop-file-utils' 'gtk3' 'hicolor-icon-theme' 'libayatana-appindicator' 'librsvg' 'webkit2gtk-4.1')
provides=('memory-pak')
conflicts=('memory-pak')
options=('!strip' '!debug')
install=\${pkgname}.install
source_x86_64=("${sourceName(version)}::${sourceUrl}")
sha256sums_x86_64=('${sourceHash}')

package() {
  local appdir="\${srcdir}/Memory-Pak-linux-x86_64"

  install -Dm755 "\${appdir}/memory-pak" "\${pkgdir}/usr/bin/memory-pak"
  install -Dm644 "\${appdir}/memory-pak.png" "\${pkgdir}/usr/share/icons/hicolor/512x512/apps/memory-pak.png"
  install -Dm644 "\${appdir}/LICENSE" "\${pkgdir}/usr/share/licenses/\${pkgname}/LICENSE"
  install -Dm644 "\${appdir}/README.md" "\${pkgdir}/usr/share/doc/\${pkgname}/README.md"
  install -Dm644 "\${appdir}/README-linux-portable.txt" "\${pkgdir}/usr/share/doc/\${pkgname}/README-linux-portable.txt"

  install -d "\${pkgdir}/usr/share/applications"
  cat > "\${pkgdir}/usr/share/applications/memory-pak.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Memory Pak
Comment=A cross-platform retro game collection tracker
Exec=memory-pak
Icon=memory-pak
Terminal=false
Categories=Utility;Game;
StartupWMClass=Memory Pak
DESKTOP
}
`
  );

  writeFileSync(
    join(aurDir, '.SRCINFO'),
    `pkgbase = ${AUR_PACKAGE_NAME}
\tpkgdesc = A cross-platform retro game collection tracker
\tpkgver = ${version}
\tpkgrel = 1
\turl = https://github.com/Aspenini/Memory-Pak
\tinstall = ${AUR_PACKAGE_NAME}.install
\tarch = x86_64
\tlicense = MIT
\tdepends = desktop-file-utils
\tdepends = gtk3
\tdepends = hicolor-icon-theme
\tdepends = libayatana-appindicator
\tdepends = librsvg
\tdepends = webkit2gtk-4.1
\tprovides = memory-pak
\tconflicts = memory-pak
\toptions = !strip
\toptions = !debug
\tsource_x86_64 = ${sourceName(version)}::${sourceUrl}
\tsha256sums_x86_64 = ${sourceHash}

pkgname = ${AUR_PACKAGE_NAME}
`
  );

  writeFileSync(
    join(aurDir, `${AUR_PACKAGE_NAME}.install`),
    `post_install() {
  gtk-update-icon-cache -q -t -f usr/share/icons/hicolor 2>/dev/null || true
  update-desktop-database -q 2>/dev/null || true
}

post_upgrade() {
  post_install
}

post_remove() {
  post_install
}
`
  );

  if (includeReadme) {
    writeFileSync(
      join(aurDir, 'README.md'),
      `# Memory Pak AUR Staging

Copy these files into the \`${AUR_PACKAGE_NAME}\` AUR repository after creating or cloning it:

\`\`\`bash
git clone ssh://aur@aur.archlinux.org/${AUR_PACKAGE_NAME}.git
cp PKGBUILD .SRCINFO ${AUR_PACKAGE_NAME}.install ${AUR_PACKAGE_NAME}/
cd ${AUR_PACKAGE_NAME}
makepkg -si
git add PKGBUILD .SRCINFO ${AUR_PACKAGE_NAME}.install
git commit -m "Update to ${version}"
git push
\`\`\`

The PKGBUILD packages the GitHub release portable tarball and uses Arch's native
\`webkit2gtk-4.1\` dependency. It intentionally does not use or reference AppImage.
`
    );
  }
}

export function sourceName(version) {
  return `${AUR_PACKAGE_NAME}-${version}-linux-x86_64.tar.gz`;
}
