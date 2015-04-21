extern crate rustc_serialize;
extern crate docopt;

use std::io::{Write, BufRead};
use docopt::Docopt;

static LF: char = '\n';
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
enum ProcessingError {
    IO(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl From<std::io::Error> for ProcessingError {
    fn from(e: std::io::Error) -> ProcessingError {
        ProcessingError::IO(e)
    }
}
impl From<std::str::Utf8Error> for ProcessingError {
    fn from(e: std::str::Utf8Error) -> ProcessingError {
        ProcessingError::Utf8(e)
    }
}

#[derive(Debug)]
struct FileInfo{
    bytes: usize,
    chars: usize,
    lines: usize,
    words: usize,
    max_line_length: usize,
}

type FileInfoResult = Result<FileInfo, ProcessingError>;

fn process_reader<T: std::io::Read>(reader: T) -> FileInfoResult {
    let mut info = FileInfo{
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
    };
    let mut rbuf = std::io::BufReader::new(reader);
    let mut lbuf: Vec<u8> = Vec::new();
    loop {
        let size = try!(rbuf.read_until(LF as u8, &mut lbuf));
        info.bytes += size;
        if size == 0 {
            break;
        }
        // Create a scope because we're going to borrow lbuf and
        // the borrow must end before we can clear it.
        {
            let line = try!(std::str::from_utf8(&lbuf));
            let size = line.chars().count();
            info.max_line_length = match line.chars().last() {
                Some(c) => {
                    if c == LF {
                        info.lines += 1;
                        std::cmp::max(info.max_line_length, size - 1)
                    } else {
                        std::cmp::max(info.max_line_length, size)
                    }
                },
                None => std::cmp::max(info.max_line_length, size),
            };
            info.chars += size;
            let mut words: Vec<&str> = line
                .split(|c: char| c.is_whitespace())
                .collect();
            words.retain(|s: &&str| s.len() > 0);
            // words.retain(|s: &&str| s.len() > 0);
            info.words += words.len();
        }
        lbuf.clear()
    }
    Ok(info)
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("wc v{}", VERSION);
        std::process::exit(0);
    }

    if args.flag_files0_from.len() != 0  && args.arg_FILE.len() != 0 {
        match writeln!(&mut std::io::stderr(),
                       "wc: file operands cannot be combined with --files0-from") {
            Ok(_) => {},
            Err(e) => panic!("Unable to write to stderr: {}", e)
        }
        match writeln!(&mut std::io::stderr(),
                       "Try 'wc --help' for more information") {
            Ok(_) => {},
            Err(e) => panic!("Unable to write to stderr: {}", e)
        }
        std::process::exit(1);
    }

    // TODO: Process --files0-from

    let mut results = Vec::new();
    let mut totals = FileInfo{
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
    };
    for file_arg in &args.arg_FILE {
        let filename = file_arg.as_ref();
        let result = match filename {
            "-" => process_reader(std::io::stdin()),
            _ => {
                let file = std::fs::File::open(filename.to_string());
                match file {
                    Ok(f) => process_reader(f),
                    Err(e) => Err(ProcessingError::IO(e)),
                }
            }
        };
        match result {
            Ok(ref r) => {
                totals.chars += r.chars;
                totals.lines += r.lines;
                totals.bytes += r.bytes;
                totals.words += r.words;
                totals.max_line_length = std::cmp::max(totals.max_line_length,
                                                       r.max_line_length);
            },
            Err(_) => {},
        }
        results.push((filename, result));
    }

    // This is used for formatting. The number in the byte count will
    // be the largest, and so will be the widest string, so it's
    // suitable for a field width.
    let field_size = totals.bytes.to_string().len();

    if results.len() > 1 {
        results.push(("total", Ok(totals)));
    }

    let mut errors_encountered = false;

    // Present the results
    for data in results.iter() {
        let (filename, ref result) = *data;
        match *result {
            Ok(ref r) => {
                // let mut parts = Vec::new();
                let mut requested_field = false;
                if args.flag_lines {
                    print!("{:1$} ", r.lines, field_size);
                    requested_field = true;
                }
                if args.flag_words {
                    print!("{:1$} ", r.words, field_size);
                    requested_field = true;
                }
                if args.flag_bytes {
                    print!("{:1$} ", r.bytes, field_size);
                    requested_field = true;
                }
                if args.flag_chars {
                    print!("{:1$} ", r.chars, field_size);
                    requested_field = true;
                }
                if args.flag_max_line_length {
                    print!("{:1$} ", r.max_line_length, field_size);
                    requested_field = true;
                }
                if !requested_field {
                    print!("{:1$} ", r.lines, field_size);
                    print!("{:1$} ", r.words, field_size);
                    print!("{:1$} ", r.bytes, field_size);
                }
                println!("{}", filename);
            },
            Err(ref e) => {

                errors_encountered = true;
                match *e {
                    ProcessingError::IO(ref e) => {
                        println!("wc: {}: {}", filename, e)
                    },
                    ProcessingError::Utf8(ref e) => {
                        println!("wc: {}: {}", filename, e)
                    },
                }
            },
        }
    }

    if errors_encountered {
        std::process::exit(1);
    }
}
