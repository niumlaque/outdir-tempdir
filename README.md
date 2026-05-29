# outdir-tempdir

`outdir-tempdir` is a small Rust testing utility crate for creating temporary
directories under Cargo-provided directories.

By default, temporary directories are created under this crate's `OUT_DIR`.

This is intentional: this crate is not a general-purpose temporary directory
crate and does not use `$TMPDIR`, `%TEMP%`, or `std::env::temp_dir()` by default.

For integration tests and benchmarks, this crate also provides APIs that create
temporary directories under `CARGO_TARGET_TMPDIR`.

## Installation

Add this crate to your `Cargo.toml`.

```toml
[dev-dependencies]
outdir-tempdir = "0.3"
```

## Which API should I use?

Use the `OUT_DIR` APIs when you want the existing default behavior.

Use the `CARGO_TARGET_TMPDIR` APIs when your test is an integration test or
benchmark and you want a test-specific writable directory provided by Cargo.

| Use case | Recommended API |
| --- | --- |
| Default behavior | `TempDir::new()` |
| Default behavior with a fixed relative path | `TempDir::with_path(path)` |
| Fallible default behavior | `TempDir::with_path_safe(path)` |
| Integration test or benchmark temporary directory | `TempDir::new_in_target_tmp()` |
| Integration test or benchmark with a fixed relative path | `TempDir::with_path_in_target_tmp(path)` |
| Fallible `CARGO_TARGET_TMPDIR` behavior | `TempDir::with_path_safe_in_target_tmp(path)` |

`CARGO_TARGET_TMPDIR` is normally available for integration tests and benchmarks.
It may not be available in ordinary unit tests.

The default `OUT_DIR` APIs are kept intentionally so leftover test data can be removed by `cargo clean`.
If your test environment cannot write to the compile-time `OUT_DIR` path, such as some sandboxed packaging environments,
use the `CARGO_TARGET_TMPDIR` APIs instead.

## Basic usage

Create a temporary directory under `OUT_DIR` with automatic removal.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::new().autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.

    // The temporary directory is removed when `dir` is dropped.
}
```

Create a temporary directory under `OUT_DIR` without automatic removal.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::new();

    let tempdir = dir.path();

    // Test your code using `tempdir`.

    // The temporary directory is not removed when `dir` is dropped.
}
```

Create a temporary directory under `OUT_DIR` using a specified relative path.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::with_path("foo/bar/baz").autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

## Using `CARGO_TARGET_TMPDIR`

`OUT_DIR` is a Cargo build-script output directory. This crate captures `OUT_DIR`
when the crate is compiled.

In some environments, that compile-time `OUT_DIR` path may not be suitable for
test runtime writes. For integration tests and benchmarks, Cargo provides
`CARGO_TARGET_TMPDIR`, which is intended for test or benchmark data.

Use the `*_in_target_tmp` APIs when you want to create temporary directories
under `CARGO_TARGET_TMPDIR`.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::new_in_target_tmp().autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

Create a temporary directory under `CARGO_TARGET_TMPDIR` using a specified
relative path.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::with_path_in_target_tmp("foo/bar/baz").autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

If `CARGO_TARGET_TMPDIR` may not be available, use the safe API.

```rust
use outdir_tempdir::{Error, TempDir};

#[test]
fn test_something() {
    let dir = match TempDir::with_path_safe_in_target_tmp("foo/bar/baz") {
        Ok(dir) => dir.autorm(),
        Err(Error::CargoTargetTmpDirNotFound) => {
            println!("skipped: CARGO_TARGET_TMPDIR is not set");
            return;
        }
        Err(error) => panic!("failed to create temporary directory: {error}"),
    };

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

## Automatic removal

Temporary directories are not removed by default.

Call `autorm()` to remove the created directory when the `TempDir` value is
dropped.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::new().autorm();

    // Removed when `dir` is dropped.
}
```

## Path safety

Specified paths must be relative paths inside the selected root directory.

Parent-directory components such as `..` and absolute paths are rejected to avoid
escaping from `OUT_DIR` or `CARGO_TARGET_TMPDIR`.

For example, these paths are rejected:

```rust
use outdir_tempdir::TempDir;

assert!(TempDir::with_path_safe("../foo").is_err());
assert!(TempDir::with_path_safe("/foo").is_err());
```

The same validation is applied to the `CARGO_TARGET_TMPDIR` APIs.

```rust
use outdir_tempdir::TempDir;

assert!(TempDir::with_path_safe_in_target_tmp("../foo").is_err());
assert!(TempDir::with_path_safe_in_target_tmp("/foo").is_err());
```

## Why not `$TMPDIR`?

This crate is intentionally scoped to Cargo-provided directories.

If you need a general-purpose temporary directory that follows the platform's
temporary directory settings, use a general-purpose temporary directory crate
instead.

Use this crate when you specifically want test directories under:

- `OUT_DIR`
- `CARGO_TARGET_TMPDIR`

## License

MIT
