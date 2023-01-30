mod error;

use crate::error::{Error, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

pub struct TempDir {
    root: PathBuf,
    target: PathBuf,
    full: PathBuf,
    autorm: bool,
}

impl TempDir {
    pub fn new() -> Result<Self> {
        TempDir::with_path(format!("test-{}", Uuid::new_v4()))
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let target = cleansing_path(path)?;

        let target_root = target_root().ok_or(Error::RootDirNotFound)?;
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

    pub fn autorm(mut self) -> Self {
        self.autorm = true;
        self
    }

    pub fn path(&self) -> &Path {
        self.full.as_path()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if self.autorm {
            if let Some(topdir) = self.target.iter().next() {
                let rmdir = self.root.join(topdir);
                fs::remove_dir_all(rmdir).unwrap();
            }
        }
    }
}

fn target_root() -> Option<PathBuf> {
    Some(PathBuf::from(std::env!("OUT_DIR")))
}

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

        let expected = PathBuf::from(format!("foo{sep}bar{sep}baz"));
        let actual = cleansing_path("foo\\bar\\baz").unwrap();
        assert_eq!(actual, expected);

        // Current directory check
        let expected = PathBuf::from(format!("tmp{sep}path"));
        let actual = cleansing_path("./tmp/path").unwrap();
        assert_eq!(actual, expected);

        let expected = PathBuf::from(format!("tmp{sep}path"));
        let actual = cleansing_path(".\\tmp\\path").unwrap();
        assert_eq!(actual, expected);

        // Root check
        let name = "/tmp/path";
        match cleansing_path(name) {
            Err(Error::RootDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
        }

        let name = "C:\\tmp\\path";
        match cleansing_path(name) {
            Err(Error::RootDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
        }

        // Parent directory check
        let name = "../tmp/path";
        match cleansing_path(name) {
            Err(Error::ParentDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
        }

        let name = "..\\tmp\\path";
        match cleansing_path(name) {
            Err(Error::ParentDirContains(s)) => assert_eq!(s, PathBuf::from(name)),
            _ => panic!(),
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
