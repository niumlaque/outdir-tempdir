use crate::{Error, Result, TempDir, TempDirRoot};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Builder for selecting temporary directory roots in caller-defined fallback order.
pub struct TempDirBuilder {
    roots: Vec<TempDirRoot>,
}

impl TempDirBuilder {
    pub(crate) fn new() -> Self {
        Self { roots: Vec::new() }
    }

    /// Add the path from the named environment variable as a root candidate.
    pub fn env<S: Into<String>>(mut self, name: S) -> Self {
        self.roots.push(TempDirRoot::Env(name.into()));
        self
    }

    /// Add the platform temporary directory as a root candidate.
    pub fn platform_temp_dir(mut self) -> Self {
        self.roots.push(TempDirRoot::PlatformTempDir);
        self
    }

    /// Add runtime `CARGO_TARGET_TMPDIR` as a root candidate.
    pub fn cargo_target_tmpdir(mut self) -> Self {
        self.roots.push(TempDirRoot::CargoTargetTmp);
        self
    }

    /// Add the crate's compile-time `OUT_DIR` as a root candidate.
    pub fn out_dir(mut self) -> Self {
        self.roots.push(TempDirRoot::Out);
        self
    }

    /// Create a randomly named temporary directory.
    pub fn build(self) -> Result<TempDir> {
        let private_root = PathBuf::from(format!("test-{}", Uuid::new_v4()));
        self.build_in_private_root(private_root, Path::new(""))
    }

    /// Create a temporary directory with a specified relative path.
    pub fn build_with_path<P: AsRef<Path>>(self, path: P) -> Result<TempDir> {
        let path = path.as_ref();
        let target = TempDir::cleanse_relative_path(path)?;
        let private_root = PathBuf::from(format!("test-{}", Uuid::new_v4()));

        if target.as_os_str().is_empty() {
            return Err(Error::InvalidPath(path.to_path_buf()));
        }

        self.build_in_private_root(private_root, &target)
    }

    fn build_in_private_root(self, private_root: PathBuf, target: &Path) -> Result<TempDir> {
        let full_target = if target.as_os_str().is_empty() {
            private_root.clone()
        } else {
            private_root.join(target)
        };

        if self.roots.is_empty() {
            return Err(Error::NoRootCandidatesConfigured);
        }

        let mut last_error = None;

        for root_kind in self.roots {
            let Some(root) = TempDir::root_path_if_available(&root_kind) else {
                continue;
            };

            match TempDir::create_in_root_with_removal(
                target,
                &full_target,
                root,
                private_root.clone(),
            ) {
                Ok(tempdir) => return Ok(tempdir),
                Err(Error::Io(error)) => last_error = Some(error),
                Err(error) => return Err(error),
            }
        }

        match last_error {
            Some(error) => Err(Error::RootCandidatesExhausted(error)),
            None => Err(Error::NoRootCandidatesAvailable),
        }
    }
}
