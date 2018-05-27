#!/bin/sh

make clean

rm -r rkm || true
mkdir rkm
cd rkm
cp ../../target/x86_64-unknown-kernel/release/libcervus.a ./
ar x libcervus.a
rm libcervus.a
cd ..

OBJ_LIST=$(echo rkm/cervus-*.o) make
