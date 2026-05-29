fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use outdir_tempdir::TempDir;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const DOCKER_SENTINEL_ENV: &str = "OUTDIR_TEMPDIR_CONSUMER_DOCKER_TEST";
    const CHILD_MODE_ENV: &str = "OUTDIR_TEMPDIR_CONSUMER_CHILD_MODE";
    const CHILD_MODE_OUT_DIR_FALLBACK: &str = "out-dir-fallback";
    const CHILD_MODE_TMPDIR_CREATE_FAILS: &str = "tmpdir-create-fails";
    const CHILD_MODE_EMPTY_ENV_SKIPPED: &str = "empty-env-skipped";
    const CHILD_MODE_PLATFORM_TEMP_DIR_FIRST: &str = "platform-temp-dir-first";
    const CASE_TMPDIR_FIRST: &str = "tmpdir-first";
    const CASE_CARGO_TARGET_TMPDIR_SECOND: &str = "cargo-target-tmpdir-second";
    const CASE_OUT_DIR_LAST: &str = "out-dir-last";
    const CASE_TMPDIR_CREATE_FAILS: &str = "tmpdir-create-fails";
    const CASE_EMPTY_ENV_SKIPPED: &str = "empty-env-skipped";
    const CASE_PLATFORM_TEMP_DIR_FIRST: &str = "platform-temp-dir-first";

    fn consumer_docker_test_enabled() -> bool {
        env::var_os(DOCKER_SENTINEL_ENV).is_some()
    }

    fn marker_rel_path(case_name: &str) -> PathBuf {
        Path::new("fallback-order").join(case_name)
    }

    fn configured_tmpdir() -> PathBuf {
        PathBuf::from(env::var_os("TMPDIR").expect("TMPDIR must be set for this test"))
    }

    fn configured_cargo_target_tmpdir() -> PathBuf {
        PathBuf::from(
            env::var_os("OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR")
                .expect("expected consumer cargo target tmpdir must be set"),
        )
    }

    fn detect_out_dir_root() -> PathBuf {
        let dir = TempDir::new().autorm();
        dir.path()
            .parent()
            .expect("OUT_DIR temp dir must have a parent")
            .to_path_buf()
    }

    fn find_private_roots_with_marker(root: &Path, case_name: &str) -> Vec<PathBuf> {
        let mut matches = Vec::new();
        let marker_rel = marker_rel_path(case_name);

        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(_) => return matches,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let Some(name) = path.file_name() else {
                continue;
            };

            if !name.to_string_lossy().starts_with("test-") {
                continue;
            }

            if path.join(&marker_rel).exists() {
                matches.push(path);
            }
        }

        matches
    }

    fn assert_builder_selected_root(
        dir: &TempDir,
        expected_root: &Path,
        case_name: &str,
    ) -> PathBuf {
        let path = dir.path().to_path_buf();
        let marker_rel = marker_rel_path(case_name);

        assert!(
            path.starts_with(expected_root),
            "temp dir {:?} should start with {:?}",
            path,
            expected_root
        );
        assert!(
            path.ends_with(&marker_rel),
            "temp dir {:?} should end with {:?}",
            path,
            marker_rel
        );
        assert!(path.exists());
        assert!(path.is_dir());

        let relative = path
            .strip_prefix(expected_root)
            .expect("builder path must be under the expected root");
        let private_top = relative
            .iter()
            .next()
            .expect("builder path must include a private top-level directory");
        let private_root = expected_root.join(private_top);

        assert!(
            private_top.to_string_lossy().starts_with("test-"),
            "private top-level directory {:?} should start with test-",
            private_top
        );

        private_root
    }

    fn build_with_fallback_order(case_name: &str) -> TempDir {
        TempDir::builder()
            .env("TMPDIR")
            .cargo_target_tmpdir()
            .out_dir()
            .build_with_path(marker_rel_path(case_name))
            .expect("failed to create temporary directory with builder")
            .autorm()
    }

    fn build_with_missing_env_then_fallback(case_name: &str) -> TempDir {
        TempDir::builder()
            .env("THIS_ENV_SHOULD_NOT_EXIST")
            .cargo_target_tmpdir()
            .out_dir()
            .build_with_path(marker_rel_path(case_name))
            .expect("failed to create temporary directory with builder")
            .autorm()
    }

    fn build_with_tmpdir_then_fallback(case_name: &str) -> TempDir {
        TempDir::builder()
            .env("TMPDIR")
            .cargo_target_tmpdir()
            .out_dir()
            .build_with_path(marker_rel_path(case_name))
            .expect("failed to fall back after TMPDIR creation failure")
            .autorm()
    }

    fn build_with_empty_env_then_fallback(case_name: &str) -> TempDir {
        TempDir::builder()
            .env("OUTDIR_TEMPDIR_EMPTY_ROOT")
            .cargo_target_tmpdir()
            .out_dir()
            .build_with_path(marker_rel_path(case_name))
            .expect("failed to fall back after empty env root")
            .autorm()
    }

    fn build_with_platform_temp_dir_then_fallback(case_name: &str) -> TempDir {
        TempDir::builder()
            .platform_temp_dir()
            .cargo_target_tmpdir()
            .out_dir()
            .build_with_path(marker_rel_path(case_name))
            .expect("failed to create temporary directory under platform temp dir")
            .autorm()
    }

    #[test]
    fn temp_dir_without_autorm_remains_after_drop() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this consumer-crate test is intended to run inside Docker");
            return;
        }

        let path: PathBuf = {
            let dir = TempDir::new();
            let path = dir.path().to_path_buf();

            assert!(path.exists());
            assert!(path.is_dir());

            path
        };

        assert!(path.exists());
        assert!(path.is_dir());

        fs::remove_dir_all(&path).expect("failed to clean up temp dir left without autorm");
        assert!(!path.exists());
    }

    #[test]
    fn temp_dir_with_autorm_is_removed_after_drop() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this consumer-crate test is intended to run inside Docker");
            return;
        }

        let path: PathBuf = {
            let dir = TempDir::new().autorm();
            let path = dir.path().to_path_buf();

            assert!(path.exists());
            assert!(path.is_dir());

            path
        };

        assert!(!path.exists());
    }

    #[test]
    fn builder_build_with_autorm_removes_only_generated_private_root() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this consumer-crate test is intended to run inside Docker");
            return;
        }

        let tmpdir = configured_tmpdir();

        let private_root: PathBuf = {
            let dir = TempDir::builder()
                .env("TMPDIR")
                .out_dir()
                .build()
                .expect("failed to create builder temp dir")
                .autorm();

            let path = dir.path().to_path_buf();

            assert!(path.starts_with(&tmpdir));
            assert!(path.exists());
            assert!(path.is_dir());

            let private_top = path
                .file_name()
                .expect("builder temp dir must have a final path component");
            assert!(private_top.to_string_lossy().starts_with("test-"));

            path
        };

        assert!(!private_root.exists());
        assert!(tmpdir.exists());
        assert!(tmpdir.is_dir());
    }

    #[test]
    fn builder_with_path_autorm_removes_only_private_root() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this deletion-safety test is intended to run inside Docker");
            return;
        }

        let tmpdir = std::env::var_os("TMPDIR").expect("TMPDIR must be set for this test");
        let tmpdir = PathBuf::from(tmpdir);

        let shared_foo = tmpdir.join("foo");
        let sentinel = shared_foo.join("sentinel.txt");

        fs::create_dir_all(&shared_foo).expect("failed to create sentinel parent directory");
        fs::write(&sentinel, "do not delete").expect("failed to write sentinel file");

        let private_root: PathBuf = {
            let dir = TempDir::builder()
                .env("TMPDIR")
                .out_dir()
                .build_with_path("foo/bar/baz")
                .expect("failed to create builder temp dir")
                .autorm();

            let path = dir.path().to_path_buf();

            assert!(path.starts_with(&tmpdir));
            assert!(path.ends_with(Path::new("foo/bar/baz")));
            assert!(path.exists());
            assert!(path.is_dir());

            let relative = path
                .strip_prefix(&tmpdir)
                .expect("builder temp dir must be under TMPDIR");

            let private_top = relative
                .iter()
                .next()
                .expect("builder temp dir must have a private top-level directory");

            assert!(private_top.to_string_lossy().starts_with("test-"));

            tmpdir.join(private_top)
        };

        assert!(!private_root.exists());
        assert!(sentinel.exists());
        assert!(shared_foo.exists());

        let _ = fs::remove_file(&sentinel);
        let _ = fs::remove_dir(&shared_foo);
    }

    #[test]
    fn builder_prefers_tmpdir_over_later_candidates() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        let tmpdir = configured_tmpdir();
        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        let dir = build_with_fallback_order(CASE_TMPDIR_FIRST);
        let private_root = assert_builder_selected_root(&dir, &tmpdir, CASE_TMPDIR_FIRST);

        let tmp_matches = find_private_roots_with_marker(&tmpdir, CASE_TMPDIR_FIRST);
        let cargo_matches = find_private_roots_with_marker(&cargo_target_tmpdir, CASE_TMPDIR_FIRST);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_TMPDIR_FIRST);

        assert_eq!(tmp_matches, vec![private_root.clone()]);
        assert!(cargo_matches.is_empty());
        assert!(out_matches.is_empty());

        drop(dir);
        assert!(!private_root.exists());
    }

    #[test]
    fn builder_prefers_cargo_target_tmpdir_when_env_is_missing() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        let dir = build_with_missing_env_then_fallback(CASE_CARGO_TARGET_TMPDIR_SECOND);
        let private_root = assert_builder_selected_root(
            &dir,
            &cargo_target_tmpdir,
            CASE_CARGO_TARGET_TMPDIR_SECOND,
        );

        let cargo_matches =
            find_private_roots_with_marker(&cargo_target_tmpdir, CASE_CARGO_TARGET_TMPDIR_SECOND);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_CARGO_TARGET_TMPDIR_SECOND);

        assert_eq!(cargo_matches, vec![private_root.clone()]);
        assert!(out_matches.is_empty());

        drop(dir);
        assert!(!private_root.exists());
    }

    #[test]
    fn builder_prefers_platform_temp_dir_over_later_candidates() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        match env::var(CHILD_MODE_ENV).as_deref() {
            Ok(CHILD_MODE_PLATFORM_TEMP_DIR_FIRST) => {
                run_platform_temp_dir_first_child_case();
                return;
            }
            Ok(_) => panic!("unexpected child mode"),
            Err(_) => {}
        }

        let current_exe = env::current_exe().expect("failed to locate current test binary");
        let status = Command::new(current_exe)
            .env(DOCKER_SENTINEL_ENV, "1")
            .env(CHILD_MODE_ENV, CHILD_MODE_PLATFORM_TEMP_DIR_FIRST)
            .env(
                "OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR",
                configured_cargo_target_tmpdir(),
            )
            .env("CARGO_TARGET_TMPDIR", configured_cargo_target_tmpdir())
            .env_remove("TMPDIR")
            .arg("--exact")
            .arg("tests::builder_prefers_platform_temp_dir_over_later_candidates")
            .arg("--nocapture")
            .status()
            .expect("failed to run child test process");

        assert!(status.success(), "child platform-temp-dir test failed");
    }

    fn run_platform_temp_dir_first_child_case() {
        assert!(
            env::var_os("TMPDIR").is_none(),
            "TMPDIR must be unset in the child platform-temp-dir test"
        );

        let platform_temp_dir = env::temp_dir();
        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        let dir = build_with_platform_temp_dir_then_fallback(CASE_PLATFORM_TEMP_DIR_FIRST);
        let private_root =
            assert_builder_selected_root(&dir, &platform_temp_dir, CASE_PLATFORM_TEMP_DIR_FIRST);

        let platform_matches =
            find_private_roots_with_marker(&platform_temp_dir, CASE_PLATFORM_TEMP_DIR_FIRST);
        let cargo_matches =
            find_private_roots_with_marker(&cargo_target_tmpdir, CASE_PLATFORM_TEMP_DIR_FIRST);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_PLATFORM_TEMP_DIR_FIRST);

        assert_eq!(platform_matches, vec![private_root.clone()]);
        assert!(cargo_matches.is_empty());
        assert!(out_matches.is_empty());

        drop(dir);
        assert!(!private_root.exists());
    }

    #[test]
    fn builder_falls_back_to_out_dir_when_env_and_cargo_target_tmpdir_are_unavailable() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        match env::var(CHILD_MODE_ENV).as_deref() {
            Ok(CHILD_MODE_OUT_DIR_FALLBACK) => {
                run_out_dir_fallback_child_case();
                return;
            }
            Ok(_) => panic!("unexpected child mode"),
            Err(_) => {}
        }

        let current_exe = env::current_exe().expect("failed to locate current test binary");
        let status = Command::new(current_exe)
            .env(DOCKER_SENTINEL_ENV, "1")
            .env(CHILD_MODE_ENV, CHILD_MODE_OUT_DIR_FALLBACK)
            .env(
                "OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR",
                configured_cargo_target_tmpdir(),
            )
            .env("TMPDIR", configured_tmpdir())
            .env_remove("CARGO_TARGET_TMPDIR")
            .arg("--exact")
            .arg("tests::builder_falls_back_to_out_dir_when_env_and_cargo_target_tmpdir_are_unavailable")
            .arg("--nocapture")
            .status()
            .expect("failed to run child test process");

        assert!(status.success(), "child fallback-order test failed");
    }

    #[test]
    fn builder_falls_back_after_tmpdir_creation_failure() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        match env::var(CHILD_MODE_ENV).as_deref() {
            Ok(CHILD_MODE_TMPDIR_CREATE_FAILS) => {
                run_tmpdir_creation_failure_child_case();
                return;
            }
            Ok(_) => panic!("unexpected child mode"),
            Err(_) => {}
        }

        let blocked_tmpdir = PathBuf::from("/tmp/outdir-tempdir-blocked-root");
        let _ = fs::remove_file(&blocked_tmpdir);
        fs::write(&blocked_tmpdir, "blocked").expect("failed to create blocked TMPDIR file");

        let current_exe = env::current_exe().expect("failed to locate current test binary");
        let status = Command::new(current_exe)
            .env(DOCKER_SENTINEL_ENV, "1")
            .env(CHILD_MODE_ENV, CHILD_MODE_TMPDIR_CREATE_FAILS)
            .env(
                "OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR",
                configured_cargo_target_tmpdir(),
            )
            .env("TMPDIR", &blocked_tmpdir)
            .env("CARGO_TARGET_TMPDIR", configured_cargo_target_tmpdir())
            .arg("--exact")
            .arg("tests::builder_falls_back_after_tmpdir_creation_failure")
            .arg("--nocapture")
            .status()
            .expect("failed to run child test process");

        let _ = fs::remove_file(&blocked_tmpdir);

        assert!(status.success(), "child fallback-after-failure test failed");
    }

    #[test]
    fn builder_skips_empty_env_root_and_uses_cargo_target_tmpdir() {
        if !consumer_docker_test_enabled() {
            println!("skipped: this fallback-order test is intended to run inside Docker");
            return;
        }

        match env::var(CHILD_MODE_ENV).as_deref() {
            Ok(CHILD_MODE_EMPTY_ENV_SKIPPED) => {
                run_empty_env_skipped_child_case();
                return;
            }
            Ok(_) => panic!("unexpected child mode"),
            Err(_) => {}
        }

        let current_exe = env::current_exe().expect("failed to locate current test binary");
        let status = Command::new(current_exe)
            .env(DOCKER_SENTINEL_ENV, "1")
            .env(CHILD_MODE_ENV, CHILD_MODE_EMPTY_ENV_SKIPPED)
            .env(
                "OUTDIR_TEMPDIR_CONSUMER_CARGO_TARGET_TMPDIR",
                configured_cargo_target_tmpdir(),
            )
            .env("TMPDIR", configured_tmpdir())
            .env("CARGO_TARGET_TMPDIR", configured_cargo_target_tmpdir())
            .env("OUTDIR_TEMPDIR_EMPTY_ROOT", "")
            .arg("--exact")
            .arg("tests::builder_skips_empty_env_root_and_uses_cargo_target_tmpdir")
            .arg("--nocapture")
            .status()
            .expect("failed to run child test process");

        assert!(status.success(), "child empty-env fallback test failed");
    }

    fn run_out_dir_fallback_child_case() {
        let tmpdir = configured_tmpdir();
        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        let dir = build_with_missing_env_then_fallback(CASE_OUT_DIR_LAST);
        let private_root = assert_builder_selected_root(&dir, &out_dir, CASE_OUT_DIR_LAST);

        let tmp_matches = find_private_roots_with_marker(&tmpdir, CASE_OUT_DIR_LAST);
        let cargo_matches = find_private_roots_with_marker(&cargo_target_tmpdir, CASE_OUT_DIR_LAST);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_OUT_DIR_LAST);

        assert!(tmp_matches.is_empty());
        assert!(cargo_matches.is_empty());
        assert_eq!(out_matches, vec![private_root.clone()]);

        drop(dir);
        assert!(!private_root.exists());
    }

    fn run_tmpdir_creation_failure_child_case() {
        let blocked_tmpdir = configured_tmpdir();
        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        assert!(
            blocked_tmpdir.is_file(),
            "TMPDIR override must point to a file"
        );

        let dir = build_with_tmpdir_then_fallback(CASE_TMPDIR_CREATE_FAILS);
        let private_root =
            assert_builder_selected_root(&dir, &cargo_target_tmpdir, CASE_TMPDIR_CREATE_FAILS);

        let cargo_matches =
            find_private_roots_with_marker(&cargo_target_tmpdir, CASE_TMPDIR_CREATE_FAILS);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_TMPDIR_CREATE_FAILS);

        assert_eq!(cargo_matches, vec![private_root.clone()]);
        assert!(out_matches.is_empty());

        drop(dir);
        assert!(!private_root.exists());
    }

    fn run_empty_env_skipped_child_case() {
        let cargo_target_tmpdir = configured_cargo_target_tmpdir();
        let out_dir = detect_out_dir_root();

        assert_eq!(
            env::var("OUTDIR_TEMPDIR_EMPTY_ROOT").as_deref(),
            Ok(""),
            "empty root env must be set to an empty string"
        );

        let dir = build_with_empty_env_then_fallback(CASE_EMPTY_ENV_SKIPPED);
        let private_root =
            assert_builder_selected_root(&dir, &cargo_target_tmpdir, CASE_EMPTY_ENV_SKIPPED);

        let cargo_matches =
            find_private_roots_with_marker(&cargo_target_tmpdir, CASE_EMPTY_ENV_SKIPPED);
        let out_matches = find_private_roots_with_marker(&out_dir, CASE_EMPTY_ENV_SKIPPED);

        assert_eq!(cargo_matches, vec![private_root.clone()]);
        assert!(out_matches.is_empty());

        drop(dir);
        assert!(!private_root.exists());
    }
}
