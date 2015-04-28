# wc.rs
wc implemented in rust

## Is it any good?

It works, ... but probably not a good example of rust code. I have no
idea what I'm doing in rust, and this is what I'm doing to teach myself.

## Compiling & Using

This is confirmed to build w/ rust beta2.

```
git clone http://github.com/mkomitee/wc.rs
cd wc.rs
cargo build
target/debug/wc -h
```

## TODO
* Improve performance. It's incredibly inefficient as compared to
  coreutils wc.
* Write tests.
* Figure out how to detect/handle non-utf8 encoded files. Right now
  wc.rs yields an error in such cases:

```
% ./target/debug/wc non-utf8-file
wc: non-utf8-file: invalid utf-8: invalid byte near index 1
```
