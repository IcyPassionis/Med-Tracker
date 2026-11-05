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

## PLATFORM:

- Linux X11 
- Linux Wayland

also it could work on windows but I didnt test it.
