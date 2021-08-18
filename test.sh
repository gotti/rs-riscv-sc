#!/bin/bash
files="./riscv-tests/*"
success=0
all=0
for file in $files; do
  all=$(($all+1))
  ./riscv64-unknown-linux-gnu-objcopy --output-format=binary $file ./out
  ./target/debug/rs-riscv-sc -t out &> /dev/null; output=$?
  if [ $output -eq 0 ]; then
    echo "$file Success!!"
    success=$(($success+1))
  else
    echo "$file Failed... $output"
  fi
done

echo "test passed: $success"
echo "test failed: $(($all-$success))"
echo "$success/$all"
echo "pass rate  : $(bc <<< "scale=1; 100*$success/$all")%"
