#!/bin/bash

COUNT=0;

while [[ ${ITERATIONS} -gt ${COUNT} || ${ITERATIONS} -eq -1 ]]; do

    # Increment count on each iteration
    if [[ ${ITERATIONS} -gt -1 ]]; then
        ((COUNT++))
	#echo ${COUNT}
    fi

    cargo run --release --features=evaluation && wait

done