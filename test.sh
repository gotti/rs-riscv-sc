#!/bin/bash
files="./riscv-tests/*"
for file in $files; do
  ./riscv64-unknown-linux-gnu-objcopy --output-format=binary file ./out
  ./target/debug/rs-riscv-sc -t out
done
