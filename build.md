# Building Tumpa

## Prerequisites

- Node.js 22+
- pnpm
- Rust toolchain (rustup)
- System dependencies (see below)

## Quick Start

```bash
pnpm install
pnpm tauri dev          # Development with hot reload
pnpm tauri build        # Production build
```

Or using just:

```bash
just install
just dev
just build
```

## System Dependencies

### Fedora

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel gtk3-devel \
    pango-devel gdk-pixbuf2-devel librsvg2-devel \
    libappindicator-gtk3-devel pcsc-lite-devel \
    gcc gcc-c++ make pkgconf-pkg-config
```

### Debian/Ubuntu

```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
    libpango1.0-dev libgdk-pixbuf-2.0-dev librsvg2-dev \
    libayatana-appindicator3-dev libpcsclite-dev \
    build-essential pkg-config
```

## Building Packages

### RPM (Fedora/RHEL) via Docker

```bash
just build-rpm              # Fedora 43 (default)
just build-rpm fedora:42    # Fedora 42
just build-all-rpm          # All supported Fedora versions
```

Output: `dist/fedora-43/tumpa-*.rpm`

### DEB (Debian/Ubuntu) via Docker

```bash
just build-deb              # Debian 13 (default)
just build-deb debian:12    # Debian 12
just build-deb ubuntu:24.04 # Ubuntu 24.04
just build-all-deb          # All supported Debian/Ubuntu versions
```

Output: `dist/debian-13/tumpa_*.deb`

### Arch Linux via Docker

```bash
just build-arch
```

Output: `dist/archlinux/tumpa-*.pkg.tar.zst`

Install: `sudo pacman -U dist/archlinux/tumpa-*.pkg.tar.zst`

### AppImage

```bash
just build-appimage
```

### macOS DMG

```bash
just build-dmg              # Unsigned
just build-dmg-signed       # Signed + notarized (requires Apple Developer credentials)
```

## Verifying Packages

### RPM

```bash
rpm -qip dist/fedora-43/tumpa-*.rpm    # Package info
rpm -qpR dist/fedora-43/tumpa-*.rpm    # Dependencies
rpm -qlp dist/fedora-43/tumpa-*.rpm    # File list
```

### DEB

```bash
dpkg-deb -I dist/debian-13/tumpa_*.deb  # Package info
dpkg-deb -c dist/debian-13/tumpa_*.deb  # File list
```

## Release Process

```bash
just build-all-rpm
just build-all-deb
just collect-release    # Gather into dist/release/
just sign               # GPG sign all packages
```

## Generating Icons

Requires ImageMagick:

```bash
just icons
```

Generates RGBA PNGs from `src/assets/icons/logo.svg` into `src-tauri/icons/`.

## Running Tests

```bash
just test               # All tests (Rust + frontend)
just test-rust          # Rust tests only
just test-frontend      # Frontend tests only
just test-card          # Include smartcard tests (requires physical card)
```

## Troubleshooting

### pnpm install fails
Ensure esbuild is in the approved builds list in package.json under `pnpm.onlyBuiltDependencies`.

### WebKitGTK headers not found
Install `webkit2gtk4.1-devel` (Fedora) or `libwebkit2gtk-4.1-dev` (Debian/Ubuntu).

### Icon format error during build
Icons must be RGBA PNG format. Regenerate with `just icons`.

### Smart card not detected
Ensure `pcscd` service is running: `systemctl start pcscd.socket`

### Wayland display protocol error on Fedora 43
If you see `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`, run with:
```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 tumpa
```
Or force X11 backend:
```bash
GDK_BACKEND=x11 tumpa
```
This is a known WebKitGTK + Wayland compatibility issue.
