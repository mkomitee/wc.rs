# wc.rs
wc implemented in rust

## TODO
* Handle --files0-from
* To improve performance, only process data if it needs to be printed.
  This means handing off the set of required fields to process_reader,
  and short-circuiting some of the processing used to extract `bytes`,
  `lines`, `chars`, `max_line_length`, or `words` if they're not needed.
* Figure out how to detect/handle non-utf8 encoded files. Right now
  wc.rs yields an error in such cases:

```
% ./target/debug/wc non-utf8-file
wc: non-utf8-file: invalid utf-8: invalid byte near index 1
```
