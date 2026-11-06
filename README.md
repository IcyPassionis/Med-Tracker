# Med Tracker

A basic Medication Tracking application for desktops written in rust/iced library.
You can add your medications, edit schedules, set taken time, reschedule your medications.
Application is currently on working state, but dont expect much.

## REQUIREMENTS

- Rust toolchain

## KNOWN ISSUES

**Stock feature:** It also doesnt implemented yet you can set it but it doesnt used on anything.

## BUILD & RUN

```bash
cargo run
# or for release build
cargo build --release
cargo build --release-lto
```

## DEBIAN PACKAGE (.deb)

Building the package needs the GTK 3 and ALSA development headers. On
Debian/Ubuntu/Mint:

```bash
sudo apt install build-essential pkg-config libgtk-3-dev libasound2-dev
```

Then build and install it:

```bash
./packaging/build-deb.sh
sudo apt install ./dist/med-tracker_<version>_<arch>.deb
```

The script runs `cargo build --release` itself. Set `MED_TRACKER_SKIP_BUILD=1`
to package an existing `target/release/Med-Tracker`.

Packages are built against the host glibc. A package built on Ubuntu 24.04 or
Mint 22 runs on Ubuntu 24.04+ / Debian 13+, but not on older releases. Build
inside an older container to target them.

## APPIMAGE

The AppImage is built inside an Ubuntu 22.04 container, so it needs glibc
2.35 or newer (Linux Mint 21+, Ubuntu 22.04+, Debian 12+). Install Docker
once:

```bash
sudo apt install docker.io
sudo usermod -aG docker "$USER"
# log out and back in, then verify:
docker run --rm hello-world
```

Build it:

```bash
./packaging/build-appimage-docker.sh
```

Output: `dist/med-tracker-<version>-x86_64.AppImage` plus `.sha256`.

```bash
chmod +x dist/med-tracker-0.8.5-x86_64.AppImage
./dist/med-tracker-0.8.5-x86_64.AppImage
```

If FUSE 2 is not available on your system:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./dist/med-tracker-0.8.5-x86_64.AppImage
```

The AppImage uses the system GTK3, ALSA and graphics drivers. On
Debian/Ubuntu/Mint install them if missing:

```bash
# Ubuntu 24.04+ / Mint 22+
sudo apt install libgtk-3-0t64 libasound2t64 libvulkan1
# older releases
sudo apt install libgtk-3-0 libasound2 libvulkan1
```

"Start with system" is disabled in AppImage builds: AppImages are mounted
under a temporary path on every launch, so a login autostart entry would
point at a stale directory. Use your desktop's own autostart settings
instead.

## PLATFORM:

- Linux X11 
- Linux Wayland

also it could work on windows but I didnt test it.
