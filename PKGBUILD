# Local working-tree package: includes uncommitted application changes.
# Keep makepkg's src/ separate from the application's src/ directory.
BUILDDIR="${startdir}/.makepkg"

pkgname=med-tracker
pkgver=0.8.5
pkgrel=1
pkgdesc='Desktop medication tracker with schedules, reminders and tray support'
arch=('x86_64')
url='https://github.com/IcyPassionis/Med-Tracker'
license=('GPL-3.0-only')
depends=('alsa-lib' 'gcc-libs' 'glibc' 'gtk3' 'libx11' 'libxcb' 'libxkbcommon' 'libxkbcommon-x11' 'wayland')
makedepends=('cargo' 'pkgconf' 'desktop-file-utils')
optdepends=(
  'vulkan-driver: Vulkan graphics backend'
  'mesa: OpenGL graphics backend'
  'xdg-desktop-portal: native sound file chooser'
  'xdg-desktop-portal-gtk: file chooser portal for GTK and lightweight desktops'
  'dex: XDG login autostart on i3'
)
options=('!lto')

prepare() {
  # This PKGBUILD intentionally packages this checkout, not the remote Git HEAD.
  mkdir -p "${srcdir}/med-tracker"
  cp -a "${startdir}/Cargo.toml" "${startdir}/Cargo.lock" \
    "${startdir}/build.rs" "${startdir}/native" \
    "${startdir}/LICENSE" "${startdir}/src" "${startdir}/icons" \
    "${startdir}/audio" "${startdir}/.cargo" "${startdir}/packaging" \
    "${srcdir}/med-tracker/"
  cd "${srcdir}/med-tracker"
  cargo fetch --locked
}

build() {
  cd "${srcdir}/med-tracker"
  cargo build --release --frozen
}

check() {
  cd "${srcdir}/med-tracker"
  desktop-file-validate packaging/med-tracker.desktop
  cargo test --release --frozen
}

package() {
  cd "${srcdir}/med-tracker"
  install -Dm755 target/release/Med-Tracker "${pkgdir}/usr/lib/med-tracker/Med-Tracker"
  install -d "${pkgdir}/usr/bin" "${pkgdir}/usr/lib/med-tracker/icons" \
    "${pkgdir}/usr/lib/med-tracker/audio"
  ln -s /usr/lib/med-tracker/Med-Tracker "${pkgdir}/usr/bin/med-tracker"
  install -m644 icons/*.png icons/*.svg "${pkgdir}/usr/lib/med-tracker/icons/"
  install -m644 audio/*.wav "${pkgdir}/usr/lib/med-tracker/audio/"
  install -Dm644 icons/med-tracker.svg "${pkgdir}/usr/share/icons/hicolor/scalable/apps/med-tracker.svg"
  install -Dm644 packaging/med-tracker.desktop "${pkgdir}/usr/share/applications/med-tracker.desktop"
  install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
