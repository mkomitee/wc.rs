#!/usr/bin/env zsh
set -x

for bin in wc ./target/debug/wc; do
    time echo -n "HELLO WORLD\nHELLO WORLD" | $bin -cmlLw - *(.) missing
done
