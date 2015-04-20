extern crate rustc_serialize;
extern crate docopt;

use std::io::{Read, stdin, stderr, BufReader, Write, BufRead};
use std::fs::File;
use std::error::Error;
use std::process::exit;
use std::str::from_utf8;
use std::cmp::max;

use docopt::Docopt;

static VERSION: &'static str = "0.0.1";
static USAGE: &'static str = "
Usage: wc [options] FILE...
       wc [options] --files0-from=F
       wc (-h|--help)
       wc (-v|--version)

Print newline, word, and byte counts for each FILE, and a total line if
more than one FILE is specified. With no FILE, or when FILE is -,
read standard input. A word is a non-zero-length sequence of characters
delimited by white space.
The options below may be used to select which counts are printed, always in
the following order: newline, word, character, byte, maximum line length.

Options:
   -c, --bytes            print the byte counts
   -m, --chars            print the character counts
   -l, --lines            print the newline counts
   --files0-from=F        read input from the files specified by NUL-terminated
                          names in file F; If F is - then read names from
                          standard input
   -L, --max-line-length  print the length of the longest line
   -w, --words            print the word counts
   -h, --help             display this help and quit
   -v, --version          output version information and exit
";

#[allow(non_snake_case)]
#[derive(RustcDecodable, Debug)]
struct Args {
    arg_FILE: Vec<String>,
    flag_bytes: bool,
    flag_chars: bool,
    flag_lines: bool,
    flag_max_line_length: bool,
    flag_words: bool,
    flag_help: bool,
    flag_version: bool,
    flag_files0_from: String,
}

#[derive(Debug)]
struct FileInfoData {
    bytes: usize,
    chars: usize,
    lines: usize,
    max_line_length: usize,
    words: usize,
}

impl FileInfoData {
    fn new() -> FileInfoData {
        FileInfoData{ bytes: 0, chars: 0, lines: 0, max_line_length: 0, words: 0 }
    }
}

type FileResult = Result<FileInfoData, String>;

#[derive(Debug)]
struct FileInfo {
    name: String,
    data: FileResult,
}

impl FileInfo {
    fn process<T: Read>(name: String, reader: T) -> FileInfo {
        // TODO: Process reader
        let mut info = FileInfoData::new();
        let mut rbuf = BufReader::new(reader);
        let mut lbuf: Vec<u8> = Vec::new();
        let mut res;

        let delim = '\n';
        loop {
            res = rbuf.read_until(delim as u8, &mut lbuf);
            match(res) {
                Ok(size) => {
                    info.bytes += size;
                    if size == 0 {
                        break;
                    }
                },
                Err(ref e) => {
                    return FileInfo{
                        name: name,
                        data: Err(Error::description(e).to_string())
                    };
                }
            };
            {
                let str_result = from_utf8(&lbuf);
                match str_result {
                    Ok(s) => {
                        let size = s.chars().count();
                        info.max_line_length = match s.chars().last() {
                            Some(x) if x == delim => {
                                info.lines += 1;
                                max(info.max_line_length, size - 1)
                            },
                            Some(_) => max(info.max_line_length, size),
                            None => max(info.max_line_length, size),
                        };
                        info.chars += size;
                        let words: Vec<&str> = s.split(|c: char| c.is_whitespace()).collect();
                        for word in words {
                            if word.len() > 0 {
                                info.words += 1;
                            }

                        }
                    },
                    Err(ref e) => {
                        return FileInfo{
                            name: name,
                            data: Err(Error::description(e).to_string())
                        };
                    }
                }
            }
            lbuf.clear();
        }
        for line in rbuf.lines() {
            match line {
                Ok(ref l) => {
                    println!("{}: {}", name, l);
                },
                Err(ref e) => {
                    return FileInfo{
                        name: name,
                        data: Err(Error::description(e).to_string())
                    };
                },
            }
        }

        FileInfo{ name: name, data: Ok(info) }
    }

    fn error(name: String, error: String) -> FileInfo {
        FileInfo{ name: name, data: Err(error) }
    }
}

    fn main() {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        if args.flag_version {
            println!("wc v{}", VERSION);
            exit(0);
        }

        if args.flag_files0_from.len() != 0  && args.arg_FILE.len() != 0 {
            match writeln!(&mut stderr(), "wc: file operands cannot be combined with --files0-from") {
                Ok(_) => {},
                Err(e) => panic!("Unable to write to stderr: {}", e)
            }
            match writeln!(&mut stderr(), "Try 'wc --help' for more information") {
                Ok(_) => {},
                Err(e) => panic!("Unable to write to stderr: {}", e)
            }
            exit(1);
        }

        // TODO: Process --files0-from

        let mut totals = FileInfoData::new();
        for file_arg in &args.arg_FILE {
            let filename = file_arg.as_ref();
            let result = match filename {
                "-" => FileInfo::process(filename.to_string(), stdin()),
                _ => {
                    let file = File::open(filename.to_string());
                    match file {
                        Ok(f) => FileInfo::process(filename.to_string(), f),
                        Err(ref e) => FileInfo::error(filename.to_string(),
                                                      Error::description(e).to_string()),
                    }
                }
            };

            let FileInfo { name: name, data: data } = result;
            match data {
                Ok(r) => {
                    println!("'{}' Ok: {:?}", name, r);
                    totals.chars += r.chars;
                    totals.lines += r.lines;
                    totals.bytes += r.bytes;
                    totals.words += r.words;
                    totals.max_line_length = max(totals.max_line_length, r.max_line_length);
                },
                Err(e) => {
                    println!("'{}' Error: {:?}", name, e);
                }
            }
        }
        println!("'total' Ok: {:?}", totals);
    }
