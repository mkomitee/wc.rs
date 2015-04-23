use std::fmt::Result as FmtResult;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Error as IOError;
use std::io::{BufReader, stdin};
use std::io::Read;

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
type OpenResult<T> = WCResult<BufReader<T>>;

fn open_arg<T: Read>(filename: &str) -> OpenResult<T> {
    match filename {
        "-" => Ok(BufReader::new(stdin())),
        _ => Ok(BufReader::new(try!(File::open(filename)))),
    }
}

fn main() {
    for arg in &["-", "filename"] {
        let file = open_arg(arg);
        println!("-: {:?}", file);
    }
}

// src/repro.rs:34:34: 34:41 error: mismatched types:
// expected `T`,
// found `std::io::stdio::Stdin`
//     (expected type parameter,
//       found struct `std::io::stdio::Stdin`) [E0308]
//     src/repro.rs:34         "-" => Ok(BufReader::new(stdin())),
// ^~~~~~~
//     <std macros>:3:43: 3:46 error: mismatched types:
// expected `T`,
// found `std::fs::File`
//     (expected type parameter,
//       found struct `std::fs::File`) [E0308]
//     <std macros>:3 $ crate:: result:: Result:: Ok ( val ) => val , $ crate:: result:: Result::
// ^~~
//     <std macros>:1:1: 6:48 note: in expansion of try!
//     src/repro.rs:35:32: 35:58 note: expansion site
//     error: aborting due to 2 previous errors
