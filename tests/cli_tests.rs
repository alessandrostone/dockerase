use std::process::Command;

fn dockerase() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dockerase"))
}

#[test]
fn test_help_flag() {
    let output = dockerase().arg("--help").output().expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Docker cleaning utility CLI"));
    assert!(stdout.contains("--nuclear"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("purge"));
    assert!(stdout.contains("select"));
}

#[test]
fn test_version_flag() {
    let output = dockerase()
        .arg("--version")
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dockerase"));
}

#[test]
fn test_help_contains_banner() {
    let output = dockerase().arg("--help").output().expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check for part of the ASCII banner
    assert!(stdout.contains("___"));
    assert!(stdout.contains("DOCKERASE") || stdout.contains("l_____j"));
}

#[test]
fn test_purge_help() {
    let output = dockerase()
        .args(["purge", "--help"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Safely remove unused Docker resources"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn test_select_help() {
    let output = dockerase()
        .args(["select", "--help"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Interactively select"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn test_invalid_command() {
    let output = dockerase()
        .arg("invalid-command")
        .output()
        .expect("Failed to run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("invalid"));
}

#[test]
fn test_default_command_runs() {
    // This test requires Docker to be running
    let output = dockerase().output().expect("Failed to run");

    // If Docker is available, it should succeed and show disk usage
    // If not, it should fail gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        assert!(
            stdout.contains("Docker Space Usage") || stdout.contains("TYPE"),
            "Expected disk usage output"
        );
    } else {
        assert!(
            stderr.contains("Docker") || stdout.contains("Docker"),
            "Expected Docker-related error message"
        );
    }
}

#[test]
fn test_nuclear_dry_run() {
    let output = dockerase()
        .args(["--nuclear", "--dry-run", "--force"])
        .output()
        .expect("Failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(stdout.contains("DRY RUN") || stdout.contains("NUCLEAR"));
    }
    // If Docker isn't available, the command will fail, which is acceptable
}

#[test]
fn test_purge_dry_run() {
    let output = dockerase()
        .args(["purge", "--dry-run", "--force"])
        .output()
        .expect("Failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        // Should either show dry run message or "nothing to clean"
        assert!(
            stdout.contains("DRY RUN")
                || stdout.contains("Dry run")
                || stdout.contains("Nothing to clean")
                || stdout.contains("tidy")
        );
    }
}

#[test]
fn test_select_dry_run_force() {
    let output = dockerase()
        .args(["select", "--dry-run", "--force"])
        .output()
        .expect("Failed to run");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        assert!(
            stdout.contains("DRY RUN")
                || stdout.contains("Dry run")
                || stdout.contains("Nothing to clean")
                || stdout.contains("tidy")
                || stdout.contains("Selected")
        );
    }
}
