use std::process::Command;
use std::env;

#[test]
fn test_changelog_init_success_mocked() {
    // Set the environment variable to bypass external script execution
    env::set_var("CONVY_TEST_MODE", "true");

    // Path to the compiled binary (adjust if necessary)
    let binary_path = env!("CARGO_BIN_EXE_convy"); // Uses env var set by Cargo

    let output = Command::new(binary_path)
        .arg("changelog")
        .arg("init")
        .output()
        .expect("Failed to execute command");

    // Unset the environment variable after the test
    env::remove_var("CONVY_TEST_MODE");

    assert!(output.status.success(), "Command did not execute successfully. Stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Changelog initialized successfully."), "Stdout did not contain success message. Stdout: {}", stdout);
}
