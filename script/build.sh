#!/usr/bin/env bash
tag_name=$1
GOOS_GOARCH='linux-amd64'

rm -f gh-ac
mkdir dist
# cargo build --release --target x86_64-unknown-linux-gnu
RUSTFLAGS="-C target-feature=+crt-static" cargo build --target x86_64-unknown-linux-gnu --release

cp target/x86_64-unknown-linux-gnu/release/gh-ac dist/gh-ac_${tag_name}_$GOOS_GOARCH
cp target/x86_64-unknown-linux-gnu/release/gh-ac .
# echo "TODO implement this script."
# echo "It should build binaries in dist/<platform>-<arch>[.exe] as needed."
# exit 1
