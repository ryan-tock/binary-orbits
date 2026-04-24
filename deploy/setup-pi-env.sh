#!/bin/bash
# Initialize a binary-orbits checkout for running on the pi: create a venv
# with the system's apt-installed scipy/numpy visible, install maturin, and
# do an initial Rust extension build. Idempotent.
#
# Run from the checkout root (e.g. ~/binary-orbits-main or ~/binary-orbits-staging).
set -e
cd "$(dirname "$0")/.."

if [ ! -x .venv/bin/python ]; then
    python3 -m venv --system-site-packages .venv
fi

if [ ! -x .venv/bin/maturin ]; then
    .venv/bin/pip install --quiet maturin
fi

if [ -d rust ]; then
    .venv/bin/maturin develop --release --manifest-path rust/Cargo.toml
fi

echo "setup-pi-env: ok (python=$(.venv/bin/python -V), maturin=$(.venv/bin/maturin --version))"
