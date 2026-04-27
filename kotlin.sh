#!/bin/bash

cargo run

# if kotlinc -Werror -Wextra argon-data/route.kt -d /dev/null; then
#     echo good
# else
#     echo bad
#     exit
# fi

D=~/projects/temp/gooje-test/android/app/build/generated/source/argon/kotlin/gooje/abi
mkdir -p $D
cp ./argon-data/{gen,route}.kt $D
