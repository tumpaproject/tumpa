_default:
  @just --list --unsorted

# Builds Mac DMG
buildmac:
  python3 packaging/mac/build_mac_app.py --with-codesign
# Creates the source tarball
sdist:
  ./scripts/create-sourcetarball
