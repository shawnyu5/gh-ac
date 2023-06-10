#!/usr/bin/env bash
tag_name=$1
cargo build --release --target x86_64-unknown-linux-gnu
cp target/release/x86_64-unknown-linux-gnu/gh-ac gh-ac
cp target/x86_64-unknown-linux-gnu/release/gh-ac dist/gh-ac_${tag_name}_x86_64-unknown-linux-gnu
# echo "TODO implement this script."
# echo "It should build binaries in dist/<platform>-<arch>[.exe] as needed."
# exit 1
