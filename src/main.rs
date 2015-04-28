extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::cmp::max;
use std::error::Error;
use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufReader, stderr, stdin};
use std::io::{Write, BufRead, Read};
use std::process::exit;
use std::str::{Utf8Error, from_utf8};

const LF: char = '\n';
const NULL: char = '\0';
const VERSION: &'static str = "0.0.1";
const USAGE: &'static str = "
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

// Macro that acts like print!, but w/ stderr
macro_rules! print_stderr(
    ($($arg:tt)*) => (
        match write!(&mut stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

// Macro that acts like println!, but w/ stderr
macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

// Enum that captures all of the expected error variants we are likely
// to return.
#[derive(Debug)]
enum WCError {
    IO(IOError),
    Utf8(Utf8Error),
}

// Implement the Display trait so that we can print our errors to the
// user. Also a pre-requisite for implementing the Error trait.
impl Display for WCError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            WCError::IO(ref e) => e.fmt(f),
            WCError::Utf8(ref e) => e.fmt(f),
        }
    }
}

// Implement the Error trait so that callers can treat a WCError as
// they would any other error.
impl std::error::Error for WCError {
    fn description(&self) -> &str {
        match *self {
            WCError::IO(ref e) => e.description(),
            WCError::Utf8(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            WCError::IO(ref e) => e.cause(),
            WCError::Utf8(ref e) => e.cause()
        }
    }
}


// Make it possible to wrap an IOError as a variant of our WCError enum.
impl From<IOError> for WCError {
    fn from(e: IOError) -> WCError {
        WCError::IO(e)
    }
}

// Make it possible to wrap a Utf8Error as a variant of our WCError enum.
impl From<Utf8Error> for WCError {
    fn from(e: Utf8Error) -> WCError {
        WCError::Utf8(e)
    }
}

// Define the struct which will wrap all of our counts. Derive the
// Debug trait so we can print it out using the "{:?}" pattern for
// debugging purposes.
#[derive(Debug)]
struct FileInfo{
    bytes: usize,
    chars: usize,
    lines: usize,
    words: usize,
    max_line_length: usize,
}

// Implement a convenience constructor that will return a zeroed
// FileInfo struct. I had considered implementing the Zero and Add
// traits and using sum to acquire our "total" instead of a fold, but
// the behavior of max_line_length just doesn't look like add to me,
// and it may be more confusing that it's worth.
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

// Wrapper around Result that specializes the Error variant to be one
// of our WCErrors.
type WCResult<T> = Result<T, WCError>;

// Print the result of attempting to process a file.
fn display(args: &Args, filename: &str, result: &WCResult<FileInfo>,
           field_size: usize) -> bool {
    match *result {
        Ok(ref r) => {
            if args.flag_lines {
                print!("{:1$} ", r.lines, field_size);
            }
            if args.flag_words {
                print!("{:1$} ", r.words, field_size);
            }
            if args.flag_bytes {
                print!("{:1$} ", r.bytes, field_size);
            }
            if args.flag_chars {
                print!("{:1$} ", r.chars, field_size);
            }
            if args.flag_max_line_length {
                print!("{:1$} ", r.max_line_length, field_size);
            }
            println!("{}", filename);
            true
        },
        Err(ref e) => {
            println_stderr!("wc: {}: {}", filename, e);
            false
        },
    }
}


// Return the named file, but opened, and in a result that's usable as
// a buffered reader. In the case of a filename "-", return an
// appropriately wrapped Stdin. Doing this allows us to treat regular
// files and stdin equivalently.
fn open_file(filename: &str) -> WCResult<Box<BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(try!(File::open(filename))))),
    }
}

fn process_file(filename: &str, args: &Args) -> WCResult<FileInfo> {
    let mut file = try!(open_file(filename));
    let mut info = FileInfo::new();
    let mut lbuf = Vec::new();
    loop {
        let size = try!(file.read_until(LF as u8, &mut lbuf));
        info.bytes += size;
        if size == 0 {
            break;
        }
        // If this if statement wasn't here, we would still need to
        // create a scope because we're going to borrow lbuf and the
        // borrow must end before we can clear it.
        if args.flag_lines || args.flag_words || args.flag_chars ||
            args.flag_max_line_length {
                // TODO: Handle files which are not utf8-encoded. Right
                // now we get an error here.
                let line = try!(from_utf8(&lbuf));

                let size = if args.flag_chars || args.flag_max_line_length {
                    line.chars().count()
                } else { 0 };

                let last = if args.flag_lines || args.flag_max_line_length {
                    line.chars().last().unwrap_or(NULL)
                } else { NULL };

                if last == LF {
                    info.lines += 1;
                }

                if args.flag_max_line_length {
                    info.max_line_length = if last == LF {
                        max(info.max_line_length, size - 1)
                    } else {
                        max(info.max_line_length, size)
                    };
                }

                info.chars += size;

                if args.flag_words {
                    let mut words: Vec<&str> = line
                        .split(|c: char| c.is_whitespace())
                        .collect();
                    words.retain(|s: &&str| s.len() > 0);
                    info.words += words.len();
                }
            }
        lbuf.clear()
    }
    Ok(info)
}

// Open the file (possibly - for stdin) and return an array of strings
// reflecting the contents of the file, split on null characters.
fn process_files0_from(filename: &str) -> WCResult<Vec<String>> {
    let mut file = try!(open_file(filename));
    let mut result = Vec::new();
    let mut lbuf = Vec::new();
    loop {
        let size = try!(file.read_until(NULL as u8, &mut lbuf));
        if size == 0 {
            break;
        }
        // Create a scope because we're going to borrow lbuf and
        // the borrow must end before we can clear it.
        {
            let line = try!(from_utf8(&lbuf));
            result.push(line.trim_right_matches(NULL).to_string());
        }
        lbuf.clear()
    }
    Ok(result)
}

fn main() {
    let mut args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("wc v{}", VERSION);
        exit(0);
    }

    if args.flag_files0_from.len() != 0  && args.arg_FILE.len() != 0 {
        print_stderr!("wc: file operands cannot be combined with ");
        println_stderr!("--files0-from");
        println_stderr!("Try 'wc --help' for more information");
        exit(1);
    }

    // If no data is specifically requested, default to lines/words/bytes
    if !(args.flag_lines || args.flag_words || args.flag_bytes
         || args.flag_chars || args.flag_max_line_length) {
        args.flag_lines = true;
        args.flag_words = true;
        args.flag_bytes = true;
    }

    // Determine what "files" to process
    let mut files: Vec<String> = Vec::new();
    if args.flag_files0_from.len() != 0 {
        match process_files0_from(args.flag_files0_from.as_ref()) {
            Ok(parts) => files.extend(parts),
            Err(e) => {
                println_stderr!("wc: error reading {}: {}",
                                args.flag_files0_from, e);
                exit(1);
            },
        };
        if &args.flag_files0_from == "-" && files.contains(&"-".to_string()) {
            print_stderr!("wc: when reading file names from stdin, no file ");
            println_stderr!("name of '-' allowed");
            exit(1);
        }
    } else {
        files.extend(args.arg_FILE.clone());
    };

    // Process all of our files
    let results: Vec<WCResult<FileInfo>> = files.iter()
        .map(|f| process_file(f, &args))
        .collect();

    // Fold over the FileInfo results which are of the Ok variant to
    // compute the "total".
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
                  max_line_length: max(acc.max_line_length,
                                       item.max_line_length),
              });

    // This is used for formatting. The number in the byte count will
    // be the largest, and so will be the widest string, so it's
    // suitable for a field width.
    let field_size = if args.flag_files0_from == "-" {
        0
    } else {
        total.bytes.to_string().len()
    };

    // For determining eventual exit code
    let mut ok = true;

    // Present the results
    for data in files.iter().zip(results.iter()) {
        let (filename, result) = data;
        ok &= display(&args, filename, result, field_size)
    }
    if results.len() > 1 {
        display(&args, "total", &Ok(total), field_size);
    }
    if !ok {
        exit(1);
    }
}
