#!/bin/bash

set -x

VERSION="v1.2.0" # todo read this from cargo.toml
BINARY="./better-git-branch"
OUTPUT_FILE="better-git-branch-${VERSION}.tar.gz"
HASH_FILE="better-git-branch-${VERSION}.tar.gz.sha256.txt"

# make the binary
cargo build --release

cd ./target/release

# make the tarball
tar -czf ${OUTPUT_FILE} ${BINARY}

# hash
HASH=`shasum -a 256 ${OUTPUT_FILE}`
echo "${HASH}" > $HASH_FILE

