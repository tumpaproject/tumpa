_default:
  @just --list --unsorted

# Builds Mac DMG
buildmac:
  python3 packaging/mac/build_mac_app.py --with-codesign

# To create a development environment
dev:
  python3 -m venv .venv
  .venv/bin/python3 -m pip install --require-hashes -r requirements.txt

# To install the locally built jce wheel in the current directory
installjce:
   .venv/bin/python3 -m pip install ./johnnycanencrypt*.whl

# To run from source
run:
  .venv/bin/python3 ./run.py

# Updates the runtime requirements file
update-runtime:
  pip-compile --generate-hashes --output-file=requirements.txt requirements.in

# Updates the dev requirments file
update-dev:
  pip-compile --generate-hashes --output-file=dev-requirements.txt dev-requirements.in

# Creates the source tarball
sdist:
  ./scripts/create-sourcetarball

