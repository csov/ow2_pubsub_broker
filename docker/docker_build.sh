#!/bin/bash
script_dir="$(dirname "$(readlink -f "$0")")"
cd "${script_dir}/.."

mkdir -p .tmp
set -e

if [[ $OSTYPE == 'darwin'* ]]; then
  export TARGET_CC=x86_64-linux-musl-gcc
  export RUSTFLAGS="-C linker=x86_64-linux-musl-gcc"
fi

cargo update
cargo build --release --target=x86_64-unknown-linux-musl
mv target/x86_64-unknown-linux-musl/release/ow2_pubsub_broker .tmp/ow2_pubsub_broker

docker build \
  -t europe-docker.pkg.dev/orion-web2/eu.gcr.io/ow2_pubsub_broker:$(git describe) \
  -f docker/from_scratch.Dockerfile .tmp/
