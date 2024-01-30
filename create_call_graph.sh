#!/bin/sh

cargo build --release
rm callgrind.out.*
valgrind --tool=callgrind ./target/release/nombrilo $1
source ./venv/bin/activate
gprof2dot -f callgrind -o callgrind.out.dot callgrind.out.*
dot -Tsvg callgrind.out.dot -o callgrind.svg
