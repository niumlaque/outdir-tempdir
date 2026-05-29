//! # OUTDIR-TEMPDIR
//!
//! `outdir-tempdir` is a small testing utility crate for creating temporary
//! directories under Cargo-provided directories.
//!
//! By default, temporary directories are created under `OUT_DIR`.
//! This is intentional: this crate is not a general-purpose temporary directory
//! crate and does not use `$TMPDIR`, `%TEMP%`, or [`std::env::temp_dir`] by default.
//!
//! If you want to use an environment-provided temporary directory such as `$TMPDIR`,
//! use the builder API explicitly, for example [`TempDir::builder`]
//! with [`TempDirBuilder::env`].
//!
//! For integration tests and benchmarks where `OUT_DIR` is not writable at test
//! runtime, this crate also provides APIs that create temporary directories under
//! `CARGO_TARGET_TMPDIR`. If `CARGO_TARGET_TMPDIR` may not be set, prefer
//! [`TempDir::with_path_safe_in_target_tmp`] over the panicking APIs.
//!
//! When you need caller-defined fallback order across multiple writable roots,
//! use [`TempDir::builder`]. This is useful in sandboxed environments where a
//! specific environment variable such as `TMPDIR` may point to the only
//! writable temporary directory.
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
//! With the non-builder `with_path("foo/bar/baz")` APIs, [`TempDir::autorm`]
//! removes the top-level component under the selected Cargo root. In this
//! example, that means `foo`, not only `foo/bar/baz`.
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
//! Create a temporary directory with caller-defined fallback order.
//!
//! ```no_run
//! # use outdir_tempdir::*;
//! #[test]
//! fn test_something() {
//!     let dir = TempDir::builder()
//!         .env("TMPDIR")         // Try TMPDIR first when the environment variable is set.
//!         .cargo_target_tmpdir() // Otherwise try runtime CARGO_TARGET_TMPDIR.
//!         .out_dir()             // Otherwise fall back to the crate's compile-time OUT_DIR.
//!         .build()
//!         .expect("failed to create temporary directory with builder")
//!         .autorm();
//!
//!     let tempdir = dir.path();
//!
//!     // Test your code using `tempdir`.
//! }
//! ```
//!
//! # Builder API
//!
//! Use [`TempDir::builder`] when you want caller-defined fallback order across
//! multiple root candidates.
//!
//! Builder candidates are tried in the exact order you add them.
//!
//! - [`TempDirBuilder::env`] uses the named environment variable only when it is
//!   set and non-empty. For example, `env("TMPDIR")` does not fall back to
//!   `/tmp` by itself.
//! - [`TempDirBuilder::platform_temp_dir`] uses [`std::env::temp_dir`] as an
//!   explicit platform-default fallback. This may resolve to `/tmp` or another
//!   OS default directory.
//! - [`TempDirBuilder::cargo_target_tmpdir`] uses runtime
//!   `CARGO_TARGET_TMPDIR` when it is available.
//! - [`TempDirBuilder::out_dir`] uses the crate's compile-time `OUT_DIR`.
//! - Builder-created directories always live under a random private top-level
//!   directory such as `test-<uuid>`.
//!
//! This is useful in sandboxed environments where `TMPDIR` may point to the
//! only writable temporary directory, while `OUT_DIR` should remain available as
//! a later fallback.
//!
//! For example, `build_with_path("foo/bar/baz")` creates
//! `root/test-<uuid>/foo/bar/baz`, and [`TempDir::autorm`] removes only
//! `root/test-<uuid>`.
//!
//! This differs from the non-builder `with_path(...)` APIs,
//! where [`TempDir::autorm`] removes the top-level component of the specified relative
//! path under the selected Cargo root.
//!
//! # Path safety
//!
//! Specified paths must be relative paths inside the selected root directory.
//! Parent-directory components such as `..` and absolute paths are rejected to
//! avoid escaping from `OUT_DIR` or `CARGO_TARGET_TMPDIR`.

mod builder;
mod error;
pub use crate::builder::TempDirBuilder;
pub use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

/// Root candidate used to create temporary directories.
#[derive(Clone)]
enum TempDirRoot {
    /// Use a root from an environment variable.
    Env(String),

    /// Use the platform temporary directory.
    PlatformTempDir,

    /// Use Cargo's OUT_DIR.
    Out,

    /// Use Cargo's CARGO_TARGET_TMPDIR.
    CargoTargetTmp,
}

/// Represents a temporary directory created under a selected root directory.
///
/// The directory is removed when this value is dropped only if automatic removal
/// has been enabled by calling [`TempDir::autorm`].
pub struct TempDir {
    root: PathBuf,
    remove_target_rel: PathBuf,
    full: PathBuf,
    autorm: bool,
}

impl TempDir {
    /// Create a builder for caller-defined temporary directory root fallback order.
    pub fn builder() -> TempDirBuilder {
        TempDirBuilder::new()
    }

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
        Self::with_path_safe_in(path, TempDirRoot::Out)
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
        Self::with_path_safe_in(path, TempDirRoot::CargoTargetTmp)
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
        let target = Self::cleanse_relative_path(path)?;
        let target_root = target_root(root)?;

        Self::create_in_root(path, &target, target_root)
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
            let rmdir = self.root.join(&self.remove_target_rel);
            match fs::remove_dir_all(rmdir) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => panic!("{error}"),
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
    TempDir::root_path_if_available(&root).ok_or(match root {
        TempDirRoot::Env(_) | TempDirRoot::PlatformTempDir => Error::NoRootCandidatesAvailable,
        TempDirRoot::Out => Error::OutDirNotFound,
        TempDirRoot::CargoTargetTmp => Error::CargoTargetTmpDirNotFound,
    })
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

impl TempDir {
    fn create_in_root(path: &Path, target: &Path, target_root: PathBuf) -> Result<Self> {
        let Some(remove_target_rel) = top_level_component_path(target) else {
            return Err(Error::InvalidPath(path.to_path_buf()));
        };
        Self::create_in_root_with_removal(path, target, target_root, remove_target_rel)
    }

    fn create_in_root_with_removal(
        path: &Path,
        target: &Path,
        target_root: PathBuf,
        remove_target_rel: PathBuf,
    ) -> Result<Self> {
        let target_full_path = target_root.join(target);

        if target_root == target_full_path {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }

        fs::create_dir_all(target_full_path.as_path())?;

        Ok(Self {
            root: target_root,
            remove_target_rel,
            full: target_full_path,
            autorm: false,
        })
    }

    fn cleanse_relative_path(path: &Path) -> Result<PathBuf> {
        cleansing_path(path)
    }

    fn root_path_if_available(root: &TempDirRoot) -> Option<PathBuf> {
        match root {
            TempDirRoot::Env(name) => {
                let value = std::env::var_os(name)?;
                if value.is_empty() {
                    return None;
                }

                Some(PathBuf::from(value))
            }
            TempDirRoot::PlatformTempDir => Some(std::env::temp_dir()),
            TempDirRoot::Out => Some(PathBuf::from(std::env!("OUT_DIR"))),
            TempDirRoot::CargoTargetTmp => {
                std::env::var_os("CARGO_TARGET_TMPDIR").map(PathBuf::from)
            }
        }
    }
}

fn top_level_component_path(path: &Path) -> Option<PathBuf> {
    path.iter().next().map(PathBuf::from)
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

    #[test]
    fn test_builder_prefers_platform_temp_dir_then_out_dir() {
        let temp_root = std::env::temp_dir();
        let rmdir = {
            let temp = TempDir::builder()
                .platform_temp_dir()
                .out_dir()
                .build()
                .expect("failed to create temporary directory with builder")
                .autorm();

            assert!(temp.path().starts_with(&temp_root));
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());

            temp.path().to_path_buf()
        };

        assert!(!rmdir.try_exists().unwrap());
    }

    #[test]
    fn test_builder_build_with_path() {
        let temp_root = std::env::temp_dir();
        let rmdir = {
            let temp = TempDir::builder()
                .platform_temp_dir()
                .build_with_path("foo/bar/baz")
                .expect("failed to create temporary directory with specified path")
                .autorm();

            let relative = temp.path().strip_prefix(&temp_root).unwrap();
            let mut components = relative.iter();
            let private_top = components.next().unwrap().to_string_lossy().into_owned();

            assert!(private_top.starts_with("test-"));
            assert!(temp.path().ends_with(Path::new("foo/bar/baz")));
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());

            temp.path()
                .strip_prefix(&temp_root)
                .unwrap()
                .iter()
                .next()
                .map(|x| temp_root.join(x))
                .unwrap()
        };

        assert!(!rmdir.try_exists().unwrap());
    }

    #[test]
    fn test_builder_falls_back_when_target_tmp_unavailable() {
        let Some(out_dir) = TempDir::root_path_if_available(&TempDirRoot::Out) else {
            panic!("OUT_DIR should always be available");
        };

        if std::env::var_os("CARGO_TARGET_TMPDIR").is_some() {
            println!("skipped: CARGO_TARGET_TMPDIR is set");
            return;
        }

        let rmdir = {
            let temp = TempDir::builder()
                .cargo_target_tmpdir()
                .out_dir()
                .build()
                .expect("failed to fall back from CARGO_TARGET_TMPDIR to OUT_DIR")
                .autorm();

            assert!(temp.path().starts_with(&out_dir));
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());

            temp.path().to_path_buf()
        };

        assert!(!rmdir.try_exists().unwrap());
    }

    #[test]
    fn test_builder_falls_back_from_missing_env_to_out_dir() {
        let Some(out_dir) = TempDir::root_path_if_available(&TempDirRoot::Out) else {
            panic!("OUT_DIR should always be available");
        };

        let rmdir = {
            let temp = TempDir::builder()
                .env("THIS_ENV_SHOULD_NOT_EXIST")
                .out_dir()
                .build()
                .expect("failed to fall back from missing env to OUT_DIR")
                .autorm();

            assert!(temp.path().starts_with(&out_dir));
            assert!(temp.path().try_exists().unwrap());
            assert!(temp.path().is_dir());

            temp.path().to_path_buf()
        };

        assert!(!rmdir.try_exists().unwrap());
    }

    #[test]
    fn test_builder_path_safety() {
        match TempDir::builder()
            .platform_temp_dir()
            .build_with_path("../tmp/path")
        {
            Err(Error::ParentDirContains(path)) => assert_eq!(path, PathBuf::from("../tmp/path")),
            _ => panic!(),
        }

        match TempDir::builder()
            .platform_temp_dir()
            .build_with_path("/tmp/path")
        {
            Err(Error::RootDirContains(path)) => assert_eq!(path, PathBuf::from("/tmp/path")),
            _ => panic!(),
        }

        match TempDir::builder().platform_temp_dir().build_with_path(".") {
            Err(Error::InvalidPath(path)) => assert_eq!(path, PathBuf::from(".")),
            _ => panic!(),
        }
    }
}
