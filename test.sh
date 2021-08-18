#!/bin/bash
files="./riscv-tests/*"
for file in $files; do
  ./riscv64-unknown-elf-objcopy --output-format=binary file ./out
  ./target/debug/rs-riscv-sc -t out
done
