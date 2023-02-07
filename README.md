# OUTDIR-TEMPDIR
A crate for cargo-test to create temporary directories.  
The temporary directories are always created in the `OUT_DIR`.

# Usage
Add dependency to your `Cargo.toml`.
```toml
[dev-dependencies]
outdir-tempdir = { git = "https://github.com/niumlaque/outdir-tempdir", branch = "main" }
```

# Examples
Create a temporary directory with automatic removal.
```rs
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    // Create a random named temporary directory
    // and automatically remove it when it is dropped.
    let dir = TempDir::new().unwrap().autorm();

    // Get temporary directory
    // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/test-<random>)
    let tempdir = dir.path();

    // Test your code using tempdir
    // ...

    // Remove the temporary directory when the dir variable is dropped
}
```

Create a temporary directory without automatic removal.
```rs
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    // Create a random named temporary directory
    let dir = TempDir::new().unwrap();

    // Get temporary directory
    // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/test-<random>)
    let tempdir = dir.path();

    // Test your code using tempdir
    // ...

    // The temporary directory will not be deleted even when dir is dropped.
}
```

Create a temporary directory with the desired path.
```rs
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    // Create a temporary directory with a specified path of 'foo/bar/baz'
    // and automatically remove it when it is dropped.
    let dir = TempDir::with_path("foo/bar/baz").unwrap().autorm();

    // Get temporary directory
    // (/path/to/crate/target/(debug|release)/build/outdir-tempdir-<random>/out/foo/bar/baz)
    let tempdir = dir.path();

    // Test your code using tempdir
    // ...

    // Remove the temporary directory when the dir variable is dropped
}
```
