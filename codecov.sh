#!/bin/sh

set -x

export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="rogato_codecov-%p-%m.profraw"

cargo build
cargo test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

rm $(find . | grep -e "\.profraw")
