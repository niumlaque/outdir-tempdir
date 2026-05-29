#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)

tmpdir="${CARGO_TARGET_TMPDIR:-$repo_root/target/tmp/outdir-tempdir-test}"

mkdir -p "$tmpdir"

cd "$repo_root"
CARGO_TARGET_TMPDIR="$tmpdir" cargo test
