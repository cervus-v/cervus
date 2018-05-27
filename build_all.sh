#!/bin/sh

RUST_TARGET_PATH=$PWD xargo build --release || exit 1
cd glue || exit 1
./build.sh || exit 
