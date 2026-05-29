#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$script_dir/.." && pwd)
image_name=outdir-tempdir-consumer-test

cd "$repo_root"

docker build \
  -f tests/docker/consumer.Dockerfile \
  -t "$image_name" \
  .

docker run --rm "$image_name" cargo test -- --test-threads=1

