# wc.rs
wc implemented in rust

## TODO
* Handle --files0-from
* To improve performance, only process data if it needs to be printed.
  This means handing off the set of required fields to process_reader,
  and short-circuiting some of the processing used to extract `bytes`,
  `lines`, `chars`, `max_line_length`, or `words` if they're not needed.
