#!/usr/bin/env zsh

for bin in wc ./target/debug/wc; do
    echo "Passing arguments to $bin:"
    time echo -n "HELLO WORLD\nHELLO WORLD" | $bin -cmlLw - *(.) missing
    echo $?
    echo
done

for bin in wc ./target/debug/wc; do
    echo "Using --files0-from with $bin:"
    time ((find *(.) -print0; echo -n '.\0') | $bin --files0-from - -cmlLw)
    echo $?
    echo
done
