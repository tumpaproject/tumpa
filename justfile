_default:
  @just --list --unsorted

# To create a development environment
dev:
  python3 -m venv .venv
  .venv/bin/python3 -m pip install -r requirements.txt

# To run from source
run:
  .venv/bin/python3 src/tumpa/app.py

# Updates the runtime requirements file
update-runtime:
  pip-compile --generate-hashes --output-file=requirements.txt requirements.in

# Updates the dev requirments file
update-dev:
  pip-compile --generate-hashes --output-file=dev-requirements.txt dev-requirements.in

# Creates the source tarball
sdist:
  ./scripts/create-sourcetarball

