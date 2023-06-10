#!/usr/bin/env bash
tag_name=$1
GOOS_GOARCH='linux-amd64'

mkdir dist
cargo build --release --target x86_64-unknown-linux-gnu
# cp target/x86_64-unknown-linux-gnu/release/gh-ac gh-ac

cp target/x86_64-unknown-linux-gnu/release/gh-ac dist/gh-ac_${tag_name}_$GOOS_GOARCH
# echo "TODO implement this script."
# echo "It should build binaries in dist/<platform>-<arch>[.exe] as needed."
# exit 1
