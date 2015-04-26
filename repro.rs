use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufRead, BufReader, stdin};

#[derive(Debug)]
enum WCError {
    IO(IOError),
    // There's more in the real program.
}

impl Display for WCError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match *self {
            WCError::IO(ref e) => write!(f, "{}", e),
            // There's more in the real program.
        }
    }
}

impl From<IOError> for WCError {
    fn from(e: IOError) -> WCError {
        WCError::IO(e)
    }
}

type WCResult<T> = Result<T, WCError>;

fn open_arg(filename: &str) -> WCResult<Box<BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(stdin()))),
        _ => Ok(Box::new(BufReader::new(try!(File::open(filename))))),
    }
}

fn main() {
    for arg in &["-", "Cargo.toml", "MISSING"] {
        let file = open_arg(arg);
        match file {
            Ok(mut reader) => {
                let mut lbuf = Vec::new();
                let _ = reader.read_until('\n' as u8, &mut lbuf);
                // I'll do the error checking in the real program
                println!("{}: {:?}", arg, lbuf);
            },
            Err(e) => {
                println!("{}: {:?}", arg, e);
            }
        }
    }
}
