#!/usr/bin/bash

# Set standard path of selection
if [[ -n ${1} ]]; then
    DIR="${1}*";
else
    DIR="./inputs/output/*";
fi

echo "Directory: ${DIR}";
echo "Evaluation Mode: ${2}";
sleep 5

for f in $DIR; do
    # run each input and evaluate it
    echo "Executing $f";
    cargo run --release -- $f ${2};
done
