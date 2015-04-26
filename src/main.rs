extern crate rustc_serialize;
extern crate docopt;

use std::cmp::max;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufReader, stderr, stdin};
use std::io::{Write, BufRead, Read};
use std::process::exit;
use std::str::{Utf8Error, from_utf8};
use docopt::Docopt;

static LF: char = '\n';
static NULL: char = '\0';
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
enum WCError {
    IO(IOError),
    Utf8(Utf8Error),
}

type WCResult<T> = Result<T, WCError>;

impl Display for WCError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            WCError::IO(ref e) => write!(f, "{}", e),
            WCError::Utf8(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<IOError> for WCError {
    fn from(e: IOError) -> WCError {
        WCError::IO(e)
    }
}
impl From<Utf8Error> for WCError {
    fn from(e: Utf8Error) -> WCError {
        WCError::Utf8(e)
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

impl FileInfo {
    fn new() -> FileInfo {
        FileInfo{bytes: 0,
                 chars: 0,
                 lines: 0,
                 words: 0,
                 max_line_length: 0,
        }
    }
}

fn open_file(filename: &str) -> WCResult<Box<BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(try!(File::open(filename))))),
    }
}

fn process_reader<T: BufRead>(mut file: T) -> WCResult<FileInfo> {
    let mut info = FileInfo::new();
    // TODO: Only to as much processing as is absolutely necessary to
    // provide the data we will end up printing.
    let mut lbuf = Vec::new();
    loop {
        let size = try!(file.read_until(LF as u8, &mut lbuf));
        info.bytes += size;
        if size == 0 {
            break;
        }
        // Create a scope because we're going to borrow lbuf and
        // the borrow must end before we can clear it.
        {
            // TODO: Handle files which are not utf8-encoded. Right
            // now we get an error here.
            let line = try!(from_utf8(&lbuf));
            let size = line.chars().count();
            let last = line.chars().last().unwrap_or(NULL);
            info.max_line_length = if last == LF {
                info.lines += 1;
                max(info.max_line_length, size - 1)
            } else {
                max(info.max_line_length, size)
            };
            info.chars += size;
            let mut words: Vec<&str> = line
                .split(|c: char| c.is_whitespace())
                .collect();
            words.retain(|s: &&str| s.len() > 0);
            info.words += words.len();
        }
        lbuf.clear()
    }
    Ok(info)
}

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        )
        );

fn split_file_on_nulls<T: BufRead>(mut file: T) -> WCResult<Vec<String>> {
    let mut result = Vec::new();
    let mut lbuf = Vec::new();
    loop {
        let size = try!(file.read_until(NULL as u8, &mut lbuf));
        if size == 0 {
            break;
        }
        {
            let line = try!(from_utf8(&lbuf));
            result.push(line.trim_right_matches(NULL).to_string());
        }
        lbuf.clear()
    }
    Ok(result)
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
        println_stderr!("wc: file operands cannot be combined with --files0-from");
        println_stderr!("Try 'wc --help' for more information");
        exit(1);
    }

    let mut files: Vec<String> = Vec::new();
    if args.flag_files0_from.len() != 0 {
        let split_results = match open_file(args.flag_files0_from.as_ref()) {
            Ok(f) => split_file_on_nulls(f),
            Err(e) => {
                println_stderr!("wc: cannot open {} for reading: {}",
                                args.flag_files0_from, e);
                exit(1);
            },
        };
        match split_results {
            Ok(parts) => files.extend(parts),
            Err(e) => {
                println_stderr!("wc: error reading {}: {}",
                                args.flag_files0_from, e);
                exit(1);
            },
        };
        if &args.flag_files0_from == "-" && files.contains(&"-".to_string()) {
            println_stderr!("wc: when reading file names from stdin, no file name of '-' allowed");
            exit(1);
        }
    } else {
        files.extend(args.arg_FILE);
    };

    let mut results: Vec<WCResult<FileInfo>> = files.iter()
        .map(|f| f.as_ref())
        .map(|f|
             match open_file(f) {
                 Ok(f) => process_reader(f),
                 Err(e) => Err(e),
             })
        .collect();

    let total: FileInfo = results.iter()
        .filter(|r| r.as_ref().is_ok())
        .map(|r| r.as_ref().unwrap())
        .fold(FileInfo::new(),
              |acc, item|
              FileInfo{
                  bytes: acc.bytes + item.bytes,
                  chars: acc.chars + item.chars,
                  lines: acc.lines + item.lines,
                  words: acc.words + item.words,
                  max_line_length: max(acc.max_line_length, item.max_line_length),
              });

    // This is used for formatting. The number in the byte count will
    // be the largest, and so will be the widest string, so it's
    // suitable for a field width.
    let field_size = total.bytes.to_string().len();

    if files.len() > 1 {
        files.push("total".to_string());
        results.push(Ok(total));
    }

    let mut errors_encountered = false;

    // Present the results
    for data in files.iter().zip(results.iter()) {
        let (filename, result) = data;
        match *result {
            Ok(ref r) => {
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
                println_stderr!("wc: {}: {}", filename, e);
            },
        }
    }
    if errors_encountered {
        exit(1);
    }
}
