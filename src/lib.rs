//! # OUTDIR-TEMPDIR
//!
//! `outdir-tempdir` is a small testing utility crate for creating temporary
//! directories under Cargo-provided directories.
//!
//! By default, temporary directories are created under `OUT_DIR`.
//! This is intentional: this crate is not a general-purpose temporary directory
//! crate and does not use `$TMPDIR`, `%TEMP%`, or [`std::env::temp_dir`] by default.
//!
//! For integration tests and benchmarks where `OUT_DIR` is not writable at test
//! runtime, this crate also provides APIs that create temporary directories under
//! `CARGO_TARGET_TMPDIR`.
//!
//! # Usage
//!
//! Add this crate to your `Cargo.toml`.
//!
//! ```toml
//! [dev-dependencies]
//! outdir-tempdir = "0.3"
//! ```
//!
//! # Examples
//!
//! Create a temporary directory under `OUT_DIR` with automatic removal.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::new().autorm();
//!
//!     // Example:
//!     // /path/to/crate/target/debug/build/<package>/out/test-<uuid>
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//!
//!     // The temporary directory is removed when `dir` is dropped.
//! }
//! ```
//!
//! Create a temporary directory under `OUT_DIR` without automatic removal.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::new();
//!
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//!
//!     // The temporary directory is not removed when `dir` is dropped.
//! }
//! ```
//!
//! Create a temporary directory under `OUT_DIR` using a specified relative path.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::with_path("foo/bar/baz").autorm();
//!
//!     // Example:
//!     // /path/to/crate/target/debug/build/<package>/out/foo/bar/baz
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//! }
//! ```
//!
//! Create a temporary directory under `CARGO_TARGET_TMPDIR`.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::new_in_target_tmp().autorm();
//!
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//! }
//! ```
//!
//! Use `CARGO_TARGET_TMPDIR` with a specified relative path.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::with_path_in_target_tmp("foo/bar/baz").autorm();
//!
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//! }
//! ```
//!
//! # Path safety
//!
//! Specified paths must be relative paths inside the selected root directory.
//! Parent-directory components such as `..` and absolute paths are rejected to
//! avoid escaping from `OUT_DIR` or `CARGO_TARGET_TMPDIR`.

mod error;
pub use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

/// Root directory kind used to create temporary directories.
enum TempDirRoot {
    /// Use Cargo's OUT_DIR.
    OutDir,

    /// Use Cargo's CARGO_TARGET_TMPDIR.
    CargoTargetTmpDir,
}

/// Represents a temporary directory created under a Cargo-provided root directory.
///
/// The directory is removed when this value is dropped only if automatic removal
/// has been enabled by calling [`TempDir::autorm`].
pub struct TempDir {
    root: PathBuf,
    target: PathBuf,
    full: PathBuf,
    autorm: bool,
}

impl TempDir {
    /// Create a randomly named temporary directory.
    ///
    /// # Panics
    ///
    /// This function panics if the temporary directory cannot be created.  
    /// (because testing cannot proceed)
    pub fn new() -> Self {
        TempDir::with_path(format!("test-{}", Uuid::new_v4()))
    }

    /// Create a temporary directory with a specified path.
    ///
    /// # Panics
    ///
    /// This function triggers a panic under the following conditions.  
    /// (because testing cannot proceed)
    ///
    /// * Attempting to access the parent directory (which may result in escaping from `OUT_DIR`).
    /// * Attempting to access the root directory (for the same reason).
    /// * Specifying the current directory (which may lead to the deletion of `OUT_DIR`).
    /// * Failing to create the temporary directory.
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self::with_path_safe(path).unwrap()
    }

    /// Create a temporary directory with a specified path under `OUT_DIR`.
    ///
    /// # Errors
    ///
    /// Attempting to access the parent directory will result in a `ParentDirContains` error, as it could lead to escaping from `OUT_DIR`.
    /// Similarly, attempting to access the root directory will result in a `RootDirContains` error for the same reason.
    /// If the current directory is specified, there is a potential risk of deleting `OUT_DIR`, resulting in an `InvalidPath` error.
    /// If the temporary directory cannot be created, it will lead to an `Io` error.
    pub fn with_path_safe<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_path_safe_in(path, TempDirRoot::OutDir)
    }

    /// Create a randomly named temporary directory under `CARGO_TARGET_TMPDIR`.
    ///
    /// # Panics
    ///
    /// This function panics if the temporary directory cannot be created.  
    /// (because testing cannot proceed)
    pub fn new_in_target_tmp() -> Self {
        Self::with_path_in_target_tmp(format!("test-{}", Uuid::new_v4()))
    }

    /// Create a temporary directory with a specified path under `CARGO_TARGET_TMPDIR`.
    ///
    /// # Panics
    ///
    /// This function triggers a panic under the following conditions.  
    /// (because testing cannot proceed)
    ///
    /// * `CARGO_TARGET_TMPDIR` is not available.
    /// * Attempting to access the parent directory (which may result in escaping from `CARGO_TARGET_TMPDIR`).
    /// * Attempting to access the root directory (for the same reason).
    /// * Specifying the current directory (which may lead to the deletion of `CARGO_TARGET_TMPDIR`).
    /// * Failing to create the temporary directory.
    pub fn with_path_in_target_tmp<P: AsRef<Path>>(path: P) -> Self {
        Self::with_path_safe_in_target_tmp(path).unwrap()
    }

    /// Create a temporary directory with a specified path under `CARGO_TARGET_TMPDIR`.
    ///
    /// # Errors
    ///
    /// If `CARGO_TARGET_TMPDIR` is not available, it will lead to a `CargoTargetTmpDirNotFound` error.
    /// Attempting to access the parent directory will result in a `ParentDirContains` error, as it could lead to escaping from `CARGO_TARGET_TMPDIR`.
    /// Similarly, attempting to access the root directory will result in a `RootDirContains` error for the same reason.
    /// If the current directory is specified, there is a potential risk of deleting `CARGO_TARGET_TMPDIR`, resulting in an `InvalidPath` error.
    /// If the temporary directory cannot be created, it will lead to an `Io` error.
    pub fn with_path_safe_in_target_tmp<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::with_path_safe_in(path, TempDirRoot::CargoTargetTmpDir)
    }

    /// Create a temporary directory with a specified path under the selected root directory.
    ///
    /// # Errors
    ///
    /// Attempting to access the parent directory will result in a `ParentDirContains` error, as it could lead to escaping from the selected root directory.
    /// Similarly, attempting to access the root directory will result in a `RootDirContains` error for the same reason.
    /// If the current directory is specified, there is a potential risk of deleting the selected root directory, resulting in an `InvalidPath` error.
    /// If the selected root directory is not available, it will lead to an `OutDirNotFound` or `CargoTargetTmpDirNotFound` error.
    /// If the temporary directory cannot be created, it will lead to an `Io` error.
    fn with_path_safe_in<P: AsRef<Path>>(path: P, root: TempDirRoot) -> Result<Self> {
        let path = path.as_ref();
        let target = cleansing_path(path)?;

        let target_root = target_root(root)?;
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

    /// Enable automatic removal when this value is dropped.
    pub fn autorm(mut self) -> Self {
        self.autorm = true;
        self
    }

    /// Get the path to the temporary directory.
    pub fn path(&self) -> &Path {
        self.full.as_path()
    }
}

impl Drop for TempDir {
    /// Remove the temporary directory if automatic removal is enabled.
    fn drop(&mut self) {
        if self.autorm {
            if let Some(topdir) = self.target.iter().next() {
                let rmdir = self.root.join(topdir);
                fs::remove_dir_all(rmdir).unwrap();
            }
        }
    }
}

impl Default for TempDir {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the selected root directory from Cargo-provided environment variables.
fn target_root(root: TempDirRoot) -> Result<PathBuf> {
    match root {
        TempDirRoot::OutDir => Ok(PathBuf::from(std::env!("OUT_DIR"))),
        TempDirRoot::CargoTargetTmpDir => std::env::var_os("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .ok_or(Error::CargoTargetTmpDirNotFound),
    }
}

/// Clean up the specified path.
///
/// # Errors
///
/// Attempting to access the parent directory will result in a `ParentDirContains` error,
/// as it could lead to escaping from the selected root directory.
/// Similarly, attempting to access the root directory will result in a `RootDirContains` error
/// for the same reason.
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
            let temp = TempDir::with_path("foo/bar/baz");
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
            let temp = TempDir::with_path("foo/bar/baz").autorm();
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());
            temp.path().to_path_buf()
        };
        assert!(!rmdir.try_exists().unwrap());
    }

    #[test]
    fn test_dir_in_target_tmp() {
        let Some(target_tmp) = std::env::var_os("CARGO_TARGET_TMPDIR").map(PathBuf::from) else {
            println!("skipped: CARGO_TARGET_TMPDIR is not set");
            return;
        };

        let rmdir = {
            let temp = TempDir::with_path_safe_in_target_tmp("foo/bar/baz")
                .expect("failed to create temporary directory under CARGO_TARGET_TMPDIR")
                .autorm();

            assert!(temp.path().starts_with(&target_tmp));
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());

            temp.path().to_path_buf()
        };

        assert!(!rmdir.try_exists().unwrap());
    }
}
