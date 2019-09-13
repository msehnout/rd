// NOTE: Code borrowed from: https://github.com/assert-rs/dir-diff/blob/master/src/lib.rs

/// The various errors that can happen when diffing two directories
#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    StripPrefix(std::path::StripPrefixError),
    WalkDir(walkdir::Error),
    SELinux(&'static str),
}

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