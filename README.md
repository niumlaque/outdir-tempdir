# outdir-tempdir

`outdir-tempdir` is a small Rust testing utility crate for creating temporary
directories under Cargo-provided or caller-selected roots.

By default, temporary directories are created under this crate's `OUT_DIR`.

This is intentional: this crate is not a general-purpose temporary directory
crate and does not use `$TMPDIR`, `%TEMP%`, or `std::env::temp_dir()` by default.

If you want to use an environment-provided temporary directory such as `$TMPDIR`,
use the builder API explicitly, for example `TempDir::builder().env("TMPDIR")`.

For integration tests and benchmarks, this crate also provides APIs that create
temporary directories under `CARGO_TARGET_TMPDIR`.

When you need a caller-defined fallback order across multiple writable roots,
including environment-provided roots, this crate also provides a builder API.

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
Prefer the safe API if `CARGO_TARGET_TMPDIR` may not be set.

Use the builder API when you need to try several roots in a specific order, such
as sandboxed packaging environments where only a temporary directory exposed
through `TMPDIR` may be writable.

| Use case | Recommended API |
| --- | --- |
| Default behavior | `TempDir::new()` |
| Default behavior with a fixed relative path | `TempDir::with_path(path)` |
| Fallible default behavior | `TempDir::with_path_safe(path)` |
| Integration test or benchmark where `CARGO_TARGET_TMPDIR` may be unavailable | `TempDir::with_path_safe_in_target_tmp(path)` |
| Integration test or benchmark temporary directory, panicking if unavailable | `TempDir::new_in_target_tmp()` |
| Integration test or benchmark with a fixed relative path, panicking if unavailable | `TempDir::with_path_in_target_tmp(path)` |
| Caller-defined fallback order across multiple roots | `TempDir::builder()` |

`CARGO_TARGET_TMPDIR` is normally available for integration tests and benchmarks.
It may not be available in ordinary unit tests.

The default `OUT_DIR` APIs are kept intentionally so leftover test data can be removed by `cargo clean`.
If your test environment cannot write to the compile-time `OUT_DIR` path, such as some sandboxed packaging environments,
use the `CARGO_TARGET_TMPDIR` APIs instead.

If neither `OUT_DIR` nor `CARGO_TARGET_TMPDIR` is suitable, use the builder API
to try `TMPDIR` first and then fall back to Cargo roots. Add the platform
temporary directory explicitly only when you want OS-default fallback.

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

When `autorm()` is used with `TempDir::with_path("foo/bar/baz")`,
the top-level component under `OUT_DIR` is removed when the value is dropped.
In this example, that means `OUT_DIR/foo` is removed, not only
`OUT_DIR/foo/bar/baz`.

## Using `CARGO_TARGET_TMPDIR`

`OUT_DIR` is a Cargo build-script output directory. This crate captures `OUT_DIR`
when the crate is compiled.

In some environments, that compile-time `OUT_DIR` path may not be suitable for
test runtime writes. For integration tests and benchmarks, Cargo provides
`CARGO_TARGET_TMPDIR`, which is intended for test or benchmark data.

Use the `*_in_target_tmp` APIs when you want to create temporary directories under `CARGO_TARGET_TMPDIR`.
If `CARGO_TARGET_TMPDIR` may not be set, use `TempDir::with_path_safe_in_target_tmp(...)` instead of the panicking APIs.

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

The same removal rule applies to `TempDir::with_path_in_target_tmp("foo/bar/baz")`.
When `autorm()` is used, the top-level component under `CARGO_TARGET_TMPDIR` is removed.
In this example, that means `CARGO_TARGET_TMPDIR/foo` is removed,
not only `CARGO_TARGET_TMPDIR/foo/bar/baz`.

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

## Builder API

Use the builder API when you need explicit fallback order. Candidates are tried
in the exact order you add them.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::builder()
        .env("TMPDIR")         // Try TMPDIR first when the environment variable is set.
        .cargo_target_tmpdir() // Otherwise try runtime CARGO_TARGET_TMPDIR.
        .out_dir()             // Otherwise fall back to the crate's compile-time OUT_DIR.
        .build()
        .expect("failed to create temporary directory with builder")
        .autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

- `.env("TMPDIR")` uses `TMPDIR` only when it is set and non-empty.
- `.cargo_target_tmpdir()` uses runtime `CARGO_TARGET_TMPDIR` if it is set.
- `.out_dir()` uses the crate's compile-time `OUT_DIR`.
- Builder-created directories always live under a random private top-level
  directory such as `test-<uuid>`.

If you explicitly want OS-default temporary-directory fallback, add
`.platform_temp_dir()`. This uses `std::env::temp_dir()` and may choose `/tmp`
or another platform default even when `TMPDIR` is not set.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::builder()
        .env("TMPDIR")
        .platform_temp_dir()
        .cargo_target_tmpdir()
        .out_dir()
        .build()
        .expect("failed to create temporary directory with builder")
        .autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

You can also specify a relative path explicitly.

```rust
use outdir_tempdir::TempDir;

#[test]
fn test_something() {
    let dir = TempDir::builder()
        .platform_temp_dir()
        .out_dir()
        .build_with_path("foo/bar/baz")
        .expect("failed to create temporary directory with builder path")
        .autorm();

    let tempdir = dir.path();

    // Test your code using `tempdir`.
}
```

With `build_with_path("foo/bar/baz")`, the builder creates
`root/test-<uuid>/foo/bar/baz`.

This private top-level directory is important when the builder uses shared roots
such as `$TMPDIR` or `/tmp`. Calling `.autorm()` removes only `root/test-<uuid>`,
not `root/foo`, so it does not risk deleting data owned by another process or
project.

This differs from the non-builder `with_path(...)` APIs,
where `autorm()` removes the top-level component of the specified relative path
under the selected Cargo root.

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

## Why not `$TMPDIR` by default?

This crate is intentionally scoped to Cargo-provided directories.

If you need a general-purpose temporary directory that follows the platform's
temporary directory settings, use a general-purpose temporary directory crate
instead.

Use this crate when you specifically want test directories under:

- `OUT_DIR`
- `CARGO_TARGET_TMPDIR`

## License

MIT
