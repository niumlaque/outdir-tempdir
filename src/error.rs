use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ParentDirContains(PathBuf),
    RootDirContains(PathBuf),
    RootDirNotFound,
    InvalidPath(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Io(e) => e.fmt(formatter),
            ParentDirContains(p) => write!(formatter, "{} contains parent dir", p.display()),
            RootDirContains(p) => write!(formatter, "{} contains root dir", p.display()),
            RootDirNotFound => write!(formatter, "Root dir for test not found"),
            InvalidPath(p) => write!(formatter, "Invalid path {}", p.display()),
        }
    }
}
