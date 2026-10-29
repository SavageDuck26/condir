//! Tests for the config macros and the functions behind them.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

use condir::{config_path, ensure_config_file, get_config_dir, make_config, pack_name};

#[test]
fn config_dir_matches_the_os() {
    let dir = get_config_dir().expect("the test environment should have a config folder");

    if cfg!(target_os = "windows") {
        assert!(!dir.as_os_str().is_empty());
    } else if cfg!(target_os = "macos") {
        assert!(
            dir.ends_with("Library/Application Support"),
            "{}",
            dir.display()
        );
    } else {
        let xdg = std::env::var_os("XDG_CONFIG_HOME");
        assert!(
            dir.ends_with(".config") || xdg.is_some(),
            "{}",
            dir.display()
        );
    }
}

#[test]
fn config_path_uses_the_calling_package_name() {
    let path = config_path!("spm.blob").expect("the folder should resolve");

    assert_eq!(
        path,
        get_config_dir()
            .expect("the folder should resolve")
            .join(pack_name!())
            .join("spm.blob")
    );
    // Called from this crate, the calling package is condir itself.
    assert!(path.ends_with("condir/spm.blob"), "{}", path.display());
}

#[test]
fn ensure_config_file_creates_folders_and_the_file() {
    let base = std::env::temp_dir().join(format!("condir-test-{}", std::process::id()));
    let path = base.join("nested").join("spm.blob");

    let made = ensure_config_file(path.clone()).expect("creating the file should work");

    assert_eq!(made, path);
    assert!(path.is_file(), "{}", path.display());
    fs::remove_dir_all(&base).expect("the test folder should be removable");
}

#[test]
fn config_path_appends_the_file_name() {
    let path = config_path!("settings.json").expect("the folder should resolve");

    assert!(path.ends_with("settings.json"), "{}", path.display());
}

/// Set by `rerun_with_env` in the child, so the child takes the inner path and
/// doesn't spawn grandchildren.
const INNER_MARKER: &str = "CONDIR_TEST_INNER";

fn is_inner_run() -> bool {
    std::env::var_os(INNER_MARKER).is_some()
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("condir-test-{}-{name}", std::process::id()))
}

/// Runs one test again in a child of this same binary with the environment
/// swapped out. That is the only way to point the OS env vars somewhere else
/// from inside the tests folder.
fn rerun_with_env(name: &str, env: &[(&str, Option<&str>)]) -> Output {
    let exe = std::env::current_exe().expect("the test binary path should be known");
    let mut cmd = Command::new(exe);
    cmd.args(["--exact", name, "--nocapture"]);
    cmd.env(INNER_MARKER, name);

    for (key, value) in env {
        match value {
            Some(value) => cmd.env(key, value),
            None => cmd.env_remove(key),
        };
    }

    cmd.output().expect("the child test should run")
}

fn assert_child_passed(out: &Output) {
    assert!(
        out.status.success(),
        "the child test failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn make_config_creates_the_file_in_the_os_folder() {
    const NAME: &str = "make_config_creates_the_file_in_the_os_folder";

    if !is_inner_run() {
        // Every OS gets its own scratch folder, so nothing lands in the real one.
        let home = scratch("home");
        let out = rerun_with_env(
            NAME,
            &[
                ("HOME", home.to_str()),
                ("XDG_CONFIG_HOME", home.join("xdg").to_str()),
                ("APPDATA", home.join("appdata").to_str()),
            ],
        );
        assert_child_passed(&out);
        fs::remove_dir_all(&home).expect("the scratch folder should be removable");
        return;
    }

    let path = make_config!("spm.blob").expect("the file should be created");

    assert_eq!(
        path,
        config_path!("spm.blob").expect("the folder should resolve")
    );
    assert!(path.starts_with(std::env::temp_dir()), "{}", path.display());
    assert!(path.ends_with("condir/spm.blob"), "{}", path.display());
    assert!(path.is_file(), "{}", path.display());

    fs::remove_file(&path).expect("the created file should be removable");
}

#[test]
fn missing_env_vars_report_a_condir_error() {
    const NAME: &str = "missing_env_vars_report_a_condir_error";

    if !is_inner_run() {
        let out = rerun_with_env(
            NAME,
            &[("HOME", None), ("XDG_CONFIG_HOME", None), ("APPDATA", None)],
        );
        assert_child_passed(&out);
        return;
    }

    let err = get_config_dir().expect_err("there should be no env var left to read");

    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    let message = err.to_string();
    assert!(message.starts_with("condir: "), "{message}");
    assert!(message.ends_with("is not set"), "{message}");
}
