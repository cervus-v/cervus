#!/bin/sh

make clean

rm -r rkm || true
mkdir rkm
cd rkm
cp ../../target/x86_64-unknown-none-gnu/release/libwasm_linux.a ./
ar x libwasm_linux.a
rm libwasm_linux.a
cd ..

OBJ_LIST=$(echo rkm/core-*.o rkm/alloc-*.o rkm/wasm_linux-*.o rkm/hexagon_e-*.o) make
