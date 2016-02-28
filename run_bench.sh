#!/bin/bash

# remove old files:
rm plot/*.txt

for i in $(seq 1 24)
do
    cargo run --release -- --no_ppm --num_threads $i --num_of_runs 10
done
