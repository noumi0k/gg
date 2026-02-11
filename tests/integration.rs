use std::process::{Command, Stdio};

fn gg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_gg"))
}

fn gg_with_config(config_path: &str) -> Command {
    let mut cmd = gg();
    cmd.env("GG_CONFIG", config_path);
    cmd
}

#[test]
fn test_no_args_shows_usage() {
    let output = gg().output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("Usage:"));
    assert!(stderr.contains("gg"));
}

#[test]
fn test_help_flag() {
    let output = gg().arg("--help").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("Usage:"));
    assert!(stderr.contains("--dump-config"));
}

#[test]
fn test_short_help_flag() {
    let output = gg().arg("-h").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_version_flag() {
    let output = gg().arg("--version").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("gg "));
}

#[test]
fn test_short_version_flag() {
    let output = gg().arg("-V").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_default_deny_blocks_unknown_command() {
    // With no config, deny_by_default=true, so any command should be blocked
    let output = gg()
        .env("GG_CONFIG", "/nonexistent/path")
        .env_remove("HOME")
        .arg("foobar")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should be blocked (exit 77) or unrecognized (exit 78)
    assert!(!output.status.success());
    assert!(stderr.contains("BLOCKED"));
}

#[test]
fn test_config_allow_rule() {
    let dir = std::env::temp_dir().join("gg_test_allow");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
deny_by_default = true
log = false

[git.rules]
allow = ["status*"]
"#,
    )
    .unwrap();

    let output = gg_with_config(config.to_str().unwrap())
        .args(["--git", "status"])
        .output()
        .unwrap();

    // git status should be allowed and executed
    // (may fail if git is not installed, but should not be exit 77)
    assert_ne!(output.status.code().unwrap_or(0), 77);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_config_deny_rule() {
    let dir = std::env::temp_dir().join("gg_test_deny");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
deny_by_default = false
log = false

[git.rules]
deny = ["push --force*"]
"#,
    )
    .unwrap();

    let output = gg_with_config(config.to_str().unwrap())
        .args(["--git", "push", "--force", "origin", "main"])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 77);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BLOCKED"));
    assert!(stderr.contains("denied by policy"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_dump_config() {
    let output = gg().arg("--dump-config").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.contains("deny_by_default"));
}

#[test]
fn test_force_git_flag() {
    let dir = std::env::temp_dir().join("gg_test_force_git");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
log = false
[git.rules]
allow = ["version"]
"#,
    )
    .unwrap();

    let output = gg_with_config(config.to_str().unwrap())
        .args(["--git", "version"])
        .output()
        .unwrap();

    // git version should succeed
    assert!(output.status.success());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_logging_creates_file() {
    let dir = std::env::temp_dir().join("gg_test_logging");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let log_file = dir.join("audit.log");
    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        format!(
            r#"
[options]
log = true
log_file = "{}"
[git.rules]
allow = ["version"]
"#,
            log_file.to_str().unwrap().replace('\\', "\\\\")
        ),
    )
    .unwrap();

    let _ = gg_with_config(config.to_str().unwrap())
        .args(["--git", "version"])
        .output()
        .unwrap();

    assert!(log_file.exists());
    let content = std::fs::read_to_string(&log_file).unwrap();
    assert!(content.contains("ALLOW"));
    assert!(content.contains("git version"));

    let _ = std::fs::remove_dir_all(&dir);
}

// --- Confirm prompt: non-TTY stdin should deny ---

#[test]
fn test_confirm_denied_when_stdin_not_tty() {
    let dir = std::env::temp_dir().join("gg_test_confirm_nontty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
deny_by_default = true
log = false

[git.rules]
confirm = ["push*"]
"#,
    )
    .unwrap();

    // Pipe stdin (not a TTY) — confirm should auto-deny
    let output = gg_with_config(config.to_str().unwrap())
        .args(["--git", "push", "origin", "main"])
        .stdin(Stdio::piped())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a terminal"));
}

// --- --gh force flag ---

#[test]
fn test_force_gh_flag() {
    let dir = std::env::temp_dir().join("gg_test_force_gh");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
log = false

[gh.rules]
allow = ["status"]
"#,
    )
    .unwrap();

    let output = gg_with_config(config.to_str().unwrap())
        .args(["--gh", "status"])
        .output()
        .unwrap();

    // gh status should be allowed (exit code depends on gh being installed & authed,
    // but should NOT be 77 = blocked by policy)
    assert_ne!(output.status.code().unwrap_or(0), 77);

    let _ = std::fs::remove_dir_all(&dir);
}

// --- Exit code passthrough ---

#[test]
fn test_exit_code_passthrough() {
    let dir = std::env::temp_dir().join("gg_test_exitcode");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
log = false

[git.rules]
allow = ["log*"]
"#,
    )
    .unwrap();

    // `git log --invalid-option` should fail with git's own exit code (not 77)
    let output = gg_with_config(config.to_str().unwrap())
        .args(["--git", "log", "--invalid-option-that-does-not-exist"])
        .output()
        .unwrap();

    let code = output.status.code().unwrap();
    assert_ne!(code, 0, "should fail");
    assert_ne!(code, 77, "should not be policy block");
    assert_ne!(code, 78, "should not be ambiguous");
}

// --- Default deny blocks gh command too ---

#[test]
fn test_default_deny_blocks_gh_command() {
    let dir = std::env::temp_dir().join("gg_test_gh_deny");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let config = dir.join("gg.toml");
    std::fs::write(
        &config,
        r#"
[options]
deny_by_default = true
log = false

[gh.rules]
allow = ["pr list*"]
"#,
    )
    .unwrap();

    let output = gg_with_config(config.to_str().unwrap())
        .args(["--gh", "repo", "delete", "foo"])
        .output()
        .unwrap();

    assert_eq!(output.status.code().unwrap(), 77);

    let _ = std::fs::remove_dir_all(&dir);
}
