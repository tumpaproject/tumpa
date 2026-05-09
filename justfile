# Tumpa - The Usability Minded PGP Application
# Run `just --list` to see available targets

# Default target
default:
    @just --list

# Run development server
dev:
    pnpm tauri dev

# Build production app (all formats)
build:
    pnpm tauri build

# Build without bundling (faster for testing)
build-no-bundle:
    pnpm tauri build --no-bundle

# Build only the frontend
build-frontend:
    pnpm build

# Install frontend dependencies
install:
    pnpm install

# Run all tests (Rust + frontend)
test:
    cd src-tauri && cargo test
    pnpm test

# Run Rust tests only
test-rust:
    cd src-tauri && cargo test

# Run frontend tests only
test-frontend:
    pnpm test

# Run Rust tests including smartcard tests (requires physical card)
test-card:
    cd src-tauri && cargo test -- --include-ignored --test-threads=1

# Clean build artifacts
clean:
    rm -rf dist
    rm -rf src-tauri/target
    rm -rf node_modules

# Clean only dist directory (package outputs)
clean-dist:
    rm -rf dist

# Clean only Rust build artifacts
clean-rust:
    rm -rf src-tauri/target

# Update the marketing version in package.json, src-tauri/tauri.conf.json,
# src-tauri/Cargo.toml, and the iOS Info.plist (CFBundleShortVersionString).
# Resets the Apple build number (CFBundleVersion / bundle.macOS.bundleVersion)
# to 1 in both the iOS Info.plist and the macOS bundle config.
# Usage: just set-version 0.10.90
set-version VERSION:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION="{{VERSION}}"
    if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+].*)?$ ]]; then
        echo "Error: '$VERSION' does not look like a semver version" >&2
        exit 1
    fi

    # JSON files: replace the first "version": "..." line.
    for f in package.json src-tauri/tauri.conf.json; do
        awk -v ver="$VERSION" '
            !done && /"version"[[:space:]]*:[[:space:]]*"/ {
                sub(/"version"[[:space:]]*:[[:space:]]*"[^"]*"/, "\"version\": \"" ver "\"")
                done = 1
            }
            { print }
        ' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    done

    # tauri.conf.json: reset bundle.macOS.bundleVersion -> "1" so the
    # macOS .app bundle gets CFBundleVersion=1 on the next build.
    awk '
        !done && /"bundleVersion"[[:space:]]*:[[:space:]]*"/ {
            sub(/"bundleVersion"[[:space:]]*:[[:space:]]*"[^"]*"/, "\"bundleVersion\": \"1\"")
            done = 1
        }
        { print }
    ' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json

    # Cargo.toml: only the first `version = "..."` (the [package] entry).
    awk -v ver="$VERSION" '
        !done && /^version[[:space:]]*=[[:space:]]*"/ {
            sub(/^version[[:space:]]*=[[:space:]]*"[^"]*"/, "version = \"" ver "\"")
            done = 1
        }
        { print }
    ' src-tauri/Cargo.toml > src-tauri/Cargo.toml.tmp && mv src-tauri/Cargo.toml.tmp src-tauri/Cargo.toml

    # iOS Info.plist: marketing version <- VERSION, build number <- 1.
    PLIST="src-tauri/gen/apple/tumpa_iOS/Info.plist"
    if [ -f "$PLIST" ]; then
        awk -v ver="$VERSION" '
            /<key>CFBundleShortVersionString<\/key>/ { print; getline; sub(/<string>[^<]*<\/string>/, "<string>" ver "</string>"); print; next }
            /<key>CFBundleVersion<\/key>/ { print; getline; sub(/<string>[^<]*<\/string>/, "<string>1</string>"); print; next }
            { print }
        ' "$PLIST" > "$PLIST.tmp" && mv "$PLIST.tmp" "$PLIST"
    fi

    # Refresh Cargo.lock so the tumpa entry stays in sync.
    (cd src-tauri && cargo update -p tumpa >/dev/null 2>&1) || true

    echo "Version set to ${VERSION} (iOS + macOS build number reset to 1)"

# Increment the Apple build number by 1, in both the iOS Info.plist
# (CFBundleVersion) and the macOS bundle config
# (tauri.conf.json -> bundle.macOS.bundleVersion). Apple requires every
# uploaded build — DMG or IPA — to carry a unique, monotonically
# increasing CFBundleVersion, so re-signing for App Store / notarization
# needs a bump even when the marketing version has not changed.
# Usage: just bump-build
bump-build:
    #!/usr/bin/env bash
    set -euo pipefail
    PLIST="src-tauri/gen/apple/tumpa_iOS/Info.plist"
    CONF="src-tauri/tauri.conf.json"

    # Read current iOS CFBundleVersion as the source of truth.
    if [ ! -f "$PLIST" ]; then
        echo "Error: $PLIST not found — run \`pnpm tauri ios init\` first" >&2
        exit 1
    fi
    CURRENT=$(awk '
        /<key>CFBundleVersion<\/key>/ {
            getline
            sub(/.*<string>/, "")
            sub(/<\/string>.*/, "")
            print
            exit
        }
    ' "$PLIST")

    if ! [[ "$CURRENT" =~ ^[0-9]+$ ]]; then
        echo "Error: CFBundleVersion is '$CURRENT', not a plain integer." >&2
        echo "Run \`just set-version <ver>\` to reset it to 1, then bump from there." >&2
        exit 1
    fi

    # Cross-check: macOS bundleVersion in tauri.conf.json should agree.
    MAC_CURRENT=$(awk '
        /"bundleVersion"[[:space:]]*:[[:space:]]*"/ {
            sub(/.*"bundleVersion"[[:space:]]*:[[:space:]]*"/, "")
            sub(/".*/, "")
            print
            exit
        }
    ' "$CONF")
    if [ -n "$MAC_CURRENT" ] && [ "$MAC_CURRENT" != "$CURRENT" ]; then
        echo "Warning: macOS bundleVersion ($MAC_CURRENT) differs from iOS CFBundleVersion ($CURRENT)." >&2
        echo "Bumping based on iOS value; both will be set to $((CURRENT + 1))." >&2
    fi

    NEW=$((CURRENT + 1))

    # Update iOS Info.plist.
    awk -v new="$NEW" '
        /<key>CFBundleVersion<\/key>/ { print; getline; sub(/<string>[^<]*<\/string>/, "<string>" new "</string>"); print; next }
        { print }
    ' "$PLIST" > "$PLIST.tmp" && mv "$PLIST.tmp" "$PLIST"

    # Update macOS bundle.macOS.bundleVersion in tauri.conf.json. Insert
    # the field if the user has a `macOS` block but no `bundleVersion`.
    if grep -q '"bundleVersion"' "$CONF"; then
        awk -v new="$NEW" '
            !done && /"bundleVersion"[[:space:]]*:[[:space:]]*"/ {
                sub(/"bundleVersion"[[:space:]]*:[[:space:]]*"[^"]*"/, "\"bundleVersion\": \"" new "\"")
                done = 1
            }
            { print }
        ' "$CONF" > "$CONF.tmp" && mv "$CONF.tmp" "$CONF"
    else
        echo "Warning: bundle.macOS.bundleVersion not found in $CONF — macOS DMG won't get the bumped build number." >&2
        echo "Add \"macOS\": { \"bundleVersion\": \"$NEW\" } under \"bundle\" in $CONF." >&2
    fi

    echo "Apple build number: ${CURRENT} -> ${NEW} (iOS Info.plist + macOS bundleVersion)"

# Generate app icons from SVG source (square owl icon)
icons:
    #!/usr/bin/env bash
    SVG="src/assets/icons/tumpa-icon.svg"
    ICONS="src-tauri/icons"
    BG="#54298B"
    mkdir -p "$ICONS"
    convert -background "$BG" "$SVG" -resize 32x32 -gravity center -extent 32x32 "PNG32:$ICONS/32x32.png"
    convert -background "$BG" "$SVG" -resize 128x128 -gravity center -extent 128x128 "PNG32:$ICONS/128x128.png"
    convert -background "$BG" "$SVG" -resize 256x256 -gravity center -extent 256x256 "PNG32:$ICONS/128x128@2x.png"
    convert -background "$BG" "$SVG" -resize 512x512 -gravity center -extent 512x512 "PNG32:$ICONS/icon.png"
    # Windows ICO (multi-resolution)
    convert "$ICONS/32x32.png" "$ICONS/128x128.png" "$ICONS/icon.png" "$ICONS/icon.ico"
    echo "Icons generated in $ICONS/"

# Convert semver prerelease to RPM-compatible version
# 0.1.0-alpha.1 -> 0.1.0~alpha.1 (sorts before 0.1.0)
_rpm-version:
    #!/usr/bin/env bash
    VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
    echo "${VERSION//-/\~}"

# Convert semver prerelease to DEB-compatible version
_deb-version:
    #!/usr/bin/env bash
    VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
    echo "${VERSION//-/\~}"

# Build RPM package using Docker
# Usage: just build-rpm [base_image]
# Examples:
#   just build-rpm              # Uses fedora:43 (default)
#   just build-rpm fedora:42
build-rpm base_image="fedora:43":
    #!/usr/bin/env bash
    set -e
    BASE_IMAGE="{{base_image}}"
    DISTRO_NAME=$(echo "$BASE_IMAGE" | tr ':' '-')
    OUTPUT_DIR="dist/$DISTRO_NAME"

    SEMVER=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
    RPM_VERSION="${SEMVER//-/\~}"

    echo "Building RPM for $BASE_IMAGE..."
    echo "Semver: $SEMVER -> RPM version: $RPM_VERSION"
    mkdir -p "$OUTPUT_DIR"

    docker build \
        --build-arg BASE_IMAGE="$BASE_IMAGE" \
        --build-arg PKG_VERSION="$RPM_VERSION" \
        -f Dockerfile.rpm \
        -t "tumpa-rpm-$DISTRO_NAME" \
        .

    CONTAINER_ID=$(docker create "tumpa-rpm-$DISTRO_NAME")
    docker cp "$CONTAINER_ID:/app/src-tauri/target/release/bundle/rpm/." "$OUTPUT_DIR/"
    docker rm "$CONTAINER_ID"

    # Rename RPM files to include distro tag
    distro=$(echo "$BASE_IMAGE" | cut -d: -f1)
    ver=$(echo "$BASE_IMAGE" | cut -d: -f2)
    case "$distro" in
        fedora) distro_tag="fc${ver}" ;;
        centos|rocky|alma) distro_tag="el${ver}" ;;
        *) distro_tag="${distro}${ver}" ;;
    esac
    for f in "$OUTPUT_DIR"/*.rpm; do
        [ -f "$f" ] || continue
        basename=$(basename "$f")
        newname=$(echo "$basename" | sed "s/\.\(x86_64\|aarch64\)\.rpm/.${distro_tag}.\1.rpm/")
        if [ "$basename" != "$newname" ]; then
            mv "$f" "$OUTPUT_DIR/$newname"
        fi
    done

    echo ""
    echo "RPM package(s) for $BASE_IMAGE available in $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/"*.rpm 2>/dev/null || echo "No RPM files found"

# Build RPM using local (unpublished) wecanencrypt
# Usage: just build-rpm-local [wecanencrypt_path] [base_image]
# Examples:
#   just build-rpm-local                                           # defaults
#   just build-rpm-local /home/kdas/code/learning/wecanencrypt
#   just build-rpm-local /home/kdas/code/learning/wecanencrypt fedora:42
build-rpm-local wecanencrypt_path="/home/kdas/code/learning/wecanencrypt" base_image="fedora:43":
    #!/usr/bin/env bash
    set -e
    BASE_IMAGE="{{base_image}}"
    WCE_PATH="{{wecanencrypt_path}}"
    DISTRO_NAME=$(echo "$BASE_IMAGE" | tr ':' '-')
    OUTPUT_DIR="dist/$DISTRO_NAME"

    if [ ! -d "$WCE_PATH/src" ]; then
        echo "Error: wecanencrypt not found at $WCE_PATH"
        exit 1
    fi

    SEMVER=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
    RPM_VERSION="${SEMVER//-/\~}"

    echo "Building RPM for $BASE_IMAGE (with local wecanencrypt from $WCE_PATH)..."
    echo "Semver: $SEMVER -> RPM version: $RPM_VERSION"
    mkdir -p "$OUTPUT_DIR"

    # Create a temp build context with both tumpa and wecanencrypt
    BUILD_CTX=$(mktemp -d)
    trap "rm -rf $BUILD_CTX" EXIT

    # Copy sources into build context
    rsync -a --exclude='target' --exclude='node_modules' --exclude='dist' --exclude='.git' . "$BUILD_CTX/tumpa/"
    rsync -a --exclude='target' --exclude='.git' "$WCE_PATH/" "$BUILD_CTX/wecanencrypt/"

    docker build \
        --build-arg BASE_IMAGE="$BASE_IMAGE" \
        --build-arg PKG_VERSION="$RPM_VERSION" \
        -f Dockerfile.rpm.local \
        -t "tumpa-rpm-local-$DISTRO_NAME" \
        "$BUILD_CTX"

    CONTAINER_ID=$(docker create "tumpa-rpm-local-$DISTRO_NAME")
    docker cp "$CONTAINER_ID:/app/src-tauri/target/release/bundle/rpm/." "$OUTPUT_DIR/"
    docker rm "$CONTAINER_ID"

    # Rename RPM files to include distro tag
    distro=$(echo "$BASE_IMAGE" | cut -d: -f1)
    ver=$(echo "$BASE_IMAGE" | cut -d: -f2)
    case "$distro" in
        fedora) distro_tag="fc${ver}" ;;
        centos|rocky|alma) distro_tag="el${ver}" ;;
        *) distro_tag="${distro}${ver}" ;;
    esac
    for f in "$OUTPUT_DIR"/*.rpm; do
        [ -f "$f" ] || continue
        basename=$(basename "$f")
        newname=$(echo "$basename" | sed "s/\.\(x86_64\|aarch64\)\.rpm/.${distro_tag}.\1.rpm/")
        if [ "$basename" != "$newname" ]; then
            mv "$f" "$OUTPUT_DIR/$newname"
        fi
    done

    echo ""
    echo "RPM package(s) for $BASE_IMAGE available in $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/"*.rpm 2>/dev/null || echo "No RPM files found"

# Build DEB package using Docker
# Usage: just build-deb [base_image]
# Examples:
#   just build-deb                  # Uses debian:13 (default)
#   just build-deb debian:12
#   just build-deb ubuntu:24.04
build-deb base_image="debian:13":
    #!/usr/bin/env bash
    set -e
    BASE_IMAGE="{{base_image}}"
    DISTRO_NAME=$(echo "$BASE_IMAGE" | tr ':' '-')
    OUTPUT_DIR="dist/$DISTRO_NAME"

    SEMVER=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')
    DEB_VERSION="${SEMVER//-/\~}"

    echo "Building DEB for $BASE_IMAGE..."
    echo "Semver: $SEMVER -> DEB version: $DEB_VERSION"
    mkdir -p "$OUTPUT_DIR"

    docker build \
        --build-arg BASE_IMAGE="$BASE_IMAGE" \
        --build-arg PKG_VERSION="$DEB_VERSION" \
        -f Dockerfile.deb \
        -t "tumpa-deb-$DISTRO_NAME" \
        .

    CONTAINER_ID=$(docker create "tumpa-deb-$DISTRO_NAME")
    docker cp "$CONTAINER_ID:/app/src-tauri/target/release/bundle/deb/." "$OUTPUT_DIR/"
    docker rm "$CONTAINER_ID"

    # Rename DEB files to include distro name
    distro_tag=$(echo "$DISTRO_NAME" | tr '-' '')
    for f in "$OUTPUT_DIR"/*.deb; do
        [ -f "$f" ] || continue
        basename=$(basename "$f")
        newname=$(echo "$basename" | sed "s/_\(amd64\|arm64\|armhf\)\.deb/_${distro_tag}_\1.deb/")
        if [ "$basename" != "$newname" ]; then
            mv "$f" "$OUTPUT_DIR/$newname"
        fi
    done

    echo ""
    echo "DEB package(s) for $BASE_IMAGE available in $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/"*.deb 2>/dev/null || echo "No DEB files found"

# Build Arch Linux package using Docker
build-arch:
    #!/usr/bin/env bash
    set -e
    OUTPUT_DIR="dist/archlinux"

    SEMVER=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')

    echo "Building Arch Linux package..."
    echo "Version: $SEMVER"
    mkdir -p "$OUTPUT_DIR"

    docker build \
        --build-arg PKG_VERSION="$SEMVER" \
        -f Dockerfile.arch \
        -t "tumpa-arch" \
        .

    CONTAINER_ID=$(docker create "tumpa-arch")
    docker cp "$CONTAINER_ID:/build/tumpa-${SEMVER}-1-x86_64.pkg.tar.zst" "$OUTPUT_DIR/" 2>/dev/null || true
    # Fallback: copy any pkg.tar.zst
    docker cp "$CONTAINER_ID:/build/." "$OUTPUT_DIR/_tmp" 2>/dev/null || true
    find "$OUTPUT_DIR/_tmp" -name '*.pkg.tar.zst' -exec cp {} "$OUTPUT_DIR/" \; 2>/dev/null || true
    rm -rf "$OUTPUT_DIR/_tmp" 2>/dev/null || true
    docker rm "$CONTAINER_ID"

    echo ""
    echo "Arch Linux package available in $OUTPUT_DIR/"
    ls -la "$OUTPUT_DIR/"*.pkg.tar.zst 2>/dev/null || echo "No package files found"

# Build AppImage
build-appimage:
    pnpm tauri build --bundles appimage

# Build all RPM packages for supported distributions
build-all-rpm:
    just build-rpm fedora:43
    just build-rpm fedora:42

# Build all DEB packages for supported distributions
build-all-deb:
    just build-deb debian:13
    just build-deb debian:12
    just build-deb ubuntu:24.04
    just build-deb ubuntu:22.04

# Collect all built packages into dist/release/ for upload
collect-release:
    #!/usr/bin/env bash
    set -e
    mkdir -p dist/release
    find dist -maxdepth 2 -name '*.rpm' -o -name '*.deb' -o -name '*.pkg.tar.zst' | while read f; do
        cp "$f" dist/release/
    done
    echo "Release packages collected in dist/release/:"
    ls -la dist/release/

# Sign all packages in dist/release/ with GPG detached signatures
sign:
    #!/usr/bin/env bash
    set -e
    for f in dist/release/*.rpm dist/release/*.deb dist/release/*.pkg.tar.zst; do
        [ -f "$f" ] || continue
        echo "Signing $f ..."
        gpg --armor --detach-sign "$f"
    done
    echo ""
    echo "Signed packages:"
    ls -la dist/release/*.asc 2>/dev/null || echo "No signatures found"

# Check Rust code
check-rust:
    cd src-tauri && cargo check

# Format Rust code
format-rust:
    cd src-tauri && cargo fmt

# Lint Rust code
lint-rust:
    cd src-tauri && cargo clippy

# Run all checks
check: check-rust

# Format all code
format-all: format-rust

# Build macOS DMG (unsigned)
build-dmg:
    pnpm tauri build --bundles dmg

# Build macOS DMG with signing and notarization
# Uses keychain profile "tugpgp" for notarization credentials
# Set up with: xcrun notarytool store-credentials tugpgp --apple-id EMAIL --team-id TEAM_ID
build-dmg-signed:
    #!/usr/bin/env bash
    set -e

    export APPLE_SIGNING_IDENTITY="Developer ID Application: Kushal Das (A7WGUTKMK6)"

    echo "Building signed macOS DMG..."
    echo "Signing identity: $APPLE_SIGNING_IDENTITY"

    pnpm tauri build --bundles dmg

    DMG_FILE=$(ls src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)
    if [ -z "$DMG_FILE" ]; then
        echo "Error: No DMG file found"
        exit 1
    fi

    echo ""
    echo "Submitting for notarization..."
    xcrun notarytool submit "$DMG_FILE" --keychain-profile tugpgp --wait

    echo "Stapling notarization ticket..."
    xcrun stapler staple "$DMG_FILE"

    echo ""
    echo "Done! Signed and notarized DMG:"
    ls -la "$DMG_FILE"

# Build and install an Android debug APK to the attached device.
# Debug APKs are signed with Android's auto-generated debug key, so they
# install directly. If multiple devices are attached, set ANDROID_SERIAL.
android-debug:
    #!/usr/bin/env bash
    set -euo pipefail
    pnpm tauri android build --debug --apk
    APK=$(ls -t src-tauri/gen/android/app/build/outputs/apk/universal/debug/*.apk 2>/dev/null | head -n1)
    if [ -z "${APK:-}" ]; then
        echo "Debug APK not found after build" >&2
        exit 1
    fi
    serial_arg=""
    if [ -n "${ANDROID_SERIAL:-}" ]; then
        serial_arg="-s ${ANDROID_SERIAL}"
    fi
    adb $serial_arg install -r "$APK"
    echo "Installed $APK"

# Build and install an Android release APK to the attached device.
# Release APKs need a signingConfig in src-tauri/gen/android/app/build.gradle.kts
# — `adb install` will reject an unsigned release APK.
android-release:
    #!/usr/bin/env bash
    set -euo pipefail
    pnpm tauri android build --apk
    APK=$(ls -t src-tauri/gen/android/app/build/outputs/apk/universal/release/*.apk 2>/dev/null | head -n1)
    if [ -z "${APK:-}" ]; then
        echo "Release APK not found after build" >&2
        exit 1
    fi
    serial_arg=""
    if [ -n "${ANDROID_SERIAL:-}" ]; then
        serial_arg="-s ${ANDROID_SERIAL}"
    fi
    adb $serial_arg install -r "$APK"
    echo "Installed $APK"

# Capture a screenshot from the connected Android device into ./screenshots/
# Uses `adb exec-out screencap -p` so nothing is written to device storage.
# If multiple devices are attached, set ANDROID_SERIAL to pick one.
android-screenshot:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p screenshots
    ts=$(date +%Y%m%d-%H%M%S)
    out="screenshots/tumpa-${ts}.png"
    serial_arg=""
    if [ -n "${ANDROID_SERIAL:-}" ]; then
        serial_arg="-s ${ANDROID_SERIAL}"
    fi
    adb $serial_arg exec-out screencap -p > "$out"
    if [ ! -s "$out" ]; then
        rm -f "$out"
        echo "Screenshot failed — is a device connected? (adb devices)" >&2
        exit 1
    fi
    echo "Saved $out"

# Show project info
info:
    @echo "Tumpa - The Usability Minded PGP Application"
    @echo ""
    @echo "Frontend: Vue 3 + Vite"
    @echo "Backend: Tauri 2 (Rust) + wecanencrypt"
    @echo ""
    @echo "Node version: $(node --version)"
    @echo "pnpm version: $(pnpm --version)"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"

# List available build targets
list-targets:
    @echo "RPM targets (Fedora):"
    @echo "  just build-rpm fedora:43  (default)"
    @echo "  just build-rpm fedora:42"
    @echo ""
    @echo "DEB targets (Debian/Ubuntu):"
    @echo "  just build-deb debian:13  (default)"
    @echo "  just build-deb debian:12"
    @echo "  just build-deb ubuntu:24.04"
    @echo "  just build-deb ubuntu:22.04"
    @echo ""
    @echo "Arch Linux:"
    @echo "  just build-arch"
    @echo ""
    @echo "macOS targets:"
    @echo "  just build-dmg           (unsigned)"
    @echo "  just build-dmg-signed    (signed + notarized)"
