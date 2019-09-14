// NOTE: Code partially borrowed from: https://github.com/assert-rs/dir-diff/blob/master/src/lib.rs

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
    SELinux(&'static str),
}

// These trait implementations are necessary for the `?` operator. Because it calls the `from`
// method behind the scene. This way I can use `Error` as the only error type in this crate, because
// all error can be converted automatically.

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(e: std::path::StripPrefixError) -> Error {
        Error::StripPrefix(e)
    }
}

impl From<walkdir::Error> for Error {
    fn from(e: walkdir::Error) -> Error {
        Error::WalkDir(e)
    }
}