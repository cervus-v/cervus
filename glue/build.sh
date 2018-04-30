#!/bin/sh

make clean

rm -r rkm || true
mkdir rkm
cd rkm
cp ../../target/x86_64-unknown-none-gnu/release/libcervus.a ./
ar x libcervus.a
rm libcervus.a
cd ..

OBJ_LIST=$(echo rkm/core-*.o rkm/alloc-*.o rkm/cervus-*.o rkm/hexagon_e-*.o) make
