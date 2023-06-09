#!/usr/bin/env bash
cargo --version
cargo build --release
cp target/release/gh-ac gh-ac
# echo "TODO implement this script."
# echo "It should build binaries in dist/<platform>-<arch>[.exe] as needed."
# exit 1
