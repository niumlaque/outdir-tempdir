//! # OUTDIR-TEMPDIR
//! A crate for cargo-test to create temporary directories.  
//! The temporary directories are always created in the `OUT_DIR`.
//!
//! # Usage
//! Add dependency to your `Cargo.toml`.
//! ```toml
//! [dev-dependencies]
//! outdir-tempdir = "0.1"
//! ```
//!
//! # Examples
//! Create a temporary directory with automatic removal.
//! ```no_run
//! # use crate::*;
//! #[test]
//! fn test_something() {
//!     // Create a random named temporary directory
//!     // and automatically remove it when it is dropped.
//!     let dir = TempDir::new().unwrap().autorm();
//!
//!     // Get temporary directory
//!     // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/test-<random>)
//!     let tempdir = dir.path();
//!
//!     // Test your code using tempdir
//!     // ...
//!
//!     // Remove the temporary directory when the dir variable is dropped
//! }
//! ```
//!
//! Create a temporary directory without automatic removal.
//! ```no_run
//! # use crate::*;
//! #[test]
//! fn test_something() {
//!     // Create a random named temporary directory
//!     let dir = TempDir::new().unwrap();
//!
//!     // Get temporary directory
//!     // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/test-<random>)
//!     let tempdir = dir.path();
//!
//!     // Test your code using tempdir
//!     // ...
//!
//!     // The temporary directory will not be deleted even when dir is dropped.
//! }
//! ```
//!
//! Create a temporary directory with the desired path.
//! ```no_run
//! # use crate::*;
//! #[test]
//! fn test_something() {
//!     // Create a temporary directory with a specified path of 'foo/bar/baz'
//!     // and automatically remove it when it is dropped.
//!     let dir = TempDir::with_path("foo/bar/baz").unwrap().autorm();
//!
//!     // Get temporary directory
//!     // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/foo/bar/baz)
//!     let tempdir = dir.path();
//!
//!     // Test your code using tempdir
//!     // ...
//!
//!     // Remove the temporary directory when the dir variable is dropped
//! }
//! ```
mod error;
pub use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

/// Provides a function to create a temporary directory that will be automatically removed when dropped.
pub struct TempDir {
    root: PathBuf,
    target: PathBuf,
    full: PathBuf,
    autorm: bool,
}

impl TempDir {
    /// Create a random named temporary directory.
    ///
    /// # Errors
    ///
    /// If the temporary directory cannot be created, it will result `Io` error.
    pub fn new() -> Result<Self> {
        TempDir::with_path(format!("test-{}", Uuid::new_v4()))
    }

    /// Create a temporary directory with a specified path.
    ///
    /// # Errors
    ///
    /// Access to parent directory will result in `ParentDirContains` error as it may escape from `OUT_DIR`.
    /// Access to root directory will also result in `RootDirContains` error for the same reason.
    /// If the current directory is specified, it will delete `OUT_DIR`, so it will result `InvalidPath` error.
    /// If the temporary directory cannot be created, it will result `Io` error.
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let target = cleansing_path(path)?;

        let target_root = target_root().ok_or(Error::OutDirNotFound)?;
        let target_full_path = target_root.join(&target);

        if target_root == target_full_path {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }

        fs::create_dir_all(target_full_path.as_path())?;

        Ok(Self {
            root: target_root,
            target,
            full: target_full_path,
            autorm: false,
        })
    }

    /// Set automatically removal.
    pub fn autorm(mut self) -> Self {
        self.autorm = true;
        self
    }

    /// Get path to the temporary directory.
    pub fn path(&self) -> &Path {
        self.full.as_path()
    }
}

impl Drop for TempDir {
    /// Remove the temporary directory if autorm is true.
    fn drop(&mut self) {
        if self.autorm {
            if let Some(topdir) = self.target.iter().next() {
                let rmdir = self.root.join(topdir);
                fs::remove_dir_all(rmdir).unwrap();
            }
        }
    }
}

/// Get `OUT_DIR` as temporary directory root.
fn target_root() -> Option<PathBuf> {
    Some(PathBuf::from(std::env!("OUT_DIR")))
}

/// Clean up the specified path.
///
/// # Errors
///
/// Access to parent directory will result in `ParentDirContains` error as it may escape from `OUT_DIR`.
/// Access to root directory will also result in `RootDirContains` error for the same reason.
fn cleansing_path<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = path.as_ref();
    let mut ret = PathBuf::new();
    for item in path.components() {
        match item {
            Component::Normal(x) => ret.push(x),
            Component::CurDir => (), // ignore
            Component::ParentDir => return Err(Error::ParentDirContains(path.to_path_buf())),
            Component::Prefix(_) | Component::RootDir => {
                return Err(Error::RootDirContains(path.to_path_buf()))
            }
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::MAIN_SEPARATOR;

    #[test]
    fn test_cleansing_path() {
        let sep = MAIN_SEPARATOR;

        // Normal check
        let expected = PathBuf::from(format!("foo{sep}bar{sep}baz"));
        let actual = cleansing_path("foo/bar/baz").unwrap();
        assert_eq!(actual, expected);

        #[cfg(target_os = "windows")]
        {
            let expected = PathBuf::from(format!("foo{sep}bar{sep}baz"));
            let actual = cleansing_path("foo\\bar\\baz").unwrap();
            assert_eq!(actual, expected);
        }

        // Current directory check
        let expected = PathBuf::from(format!("tmp{sep}path"));
        let actual = cleansing_path("./tmp/path").unwrap();
        assert_eq!(actual, expected);

        #[cfg(target_os = "windows")]
        {
            let expected = PathBuf::from(format!("tmp{sep}path"));
            let actual = cleansing_path(".\\tmp\\path").unwrap();
            assert_eq!(actual, expected);
        }

        // Root check
        let name = "/tmp/path";
        match cleansing_path(name) {
            Err(Error::RootDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
        }

        #[cfg(target_os = "windows")]
        {
            let name = "C:\\tmp\\path";
            match cleansing_path(name) {
                Err(Error::RootDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
                _ => panic!(),
            }
        }

        // Parent directory check
        let name = "../tmp/path";
        match cleansing_path(name) {
            Err(Error::ParentDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
        }

        #[cfg(target_os = "windows")]
        {
            let name = "..\\tmp\\path";
            match cleansing_path(name) {
                Err(Error::ParentDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_dir() {
        // no auto remove dir
        let mut rmdir = {
            let temp = TempDir::with_path("foo/bar/baz").unwrap();
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());
            temp.path().to_path_buf()
        };
        assert!(rmdir.try_exists().unwrap());
        assert!(rmdir.is_dir());
        rmdir.pop();
        rmdir.pop();
        fs::remove_dir_all(&rmdir).unwrap();
        assert!(!rmdir.try_exists().unwrap());

        // auto remove dir
        let rmdir = {
            let temp = TempDir::with_path("foo/bar/baz").unwrap().autorm();
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());
            temp.path().to_path_buf()
        };
        assert!(!rmdir.try_exists().unwrap());
    }
}
