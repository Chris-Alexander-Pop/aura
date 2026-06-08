//! Capture argv builders, path validation, and [`CaptureProcess`] mock runner (no Wayland).

mod common;

use ags_sidecar::services::capture::{
    default_screenshot_file_path, grim_full_argv, grim_region_argv, slurp_argv,
    validate_screenshot_path, wl_copy_argv, CaptureProcess,
};
use ags_sidecar::utils::process::{ExecOpts, MockCommandRunner};

#[test]
fn screenshot_path_validation_branches() {
    assert!(validate_screenshot_path("../etc/passwd").is_err());
    assert!(validate_screenshot_path("").is_err());
    let rel = validate_screenshot_path("Pictures/aura-test.png").unwrap();
    assert!(rel.to_string_lossy().contains("Pictures"));

    let home = std::env::var("HOME").expect("HOME set");
    let abs = format!("{home}/Pictures/aura-abs.png");
    let resolved = validate_screenshot_path(&abs).unwrap();
    assert!(resolved.to_string_lossy().contains("Pictures/aura-abs.png"));
}

#[test]
fn screenshot_argv_tables() {
    assert_eq!(slurp_argv(), vec!["slurp"]);
    assert_eq!(grim_full_argv(), vec!["grim"]);
    let region = grim_region_argv("10,20 30x40");
    assert_eq!(region, vec!["grim", "-g", "10,20 30x40"]);
    let copy = wl_copy_argv();
    assert!(copy.contains(&"wl-copy".to_string()));
    assert!(copy.contains(&"image/png".to_string()));
}

#[test]
fn default_screenshot_path_under_pictures() {
    let path = default_screenshot_file_path().expect("HOME set");
    assert!(path.to_string_lossy().contains("Pictures"));
    assert!(path.to_string_lossy().ends_with(".png"));
}

#[tokio::test]
async fn capture_process_mock_runner_returns_fixture_stdout() {
    let runner = MockCommandRunner {
        stdout: "0,0 100x100".into(),
        detached_ok: true,
    };
    let out: String = CaptureProcess::run(&runner, &["slurp"], ExecOpts::default())
        .await
        .unwrap();
    assert_eq!(out, "0,0 100x100");
}

#[tokio::test]
async fn capture_process_mock_runner_detached_failure() {
    let runner = MockCommandRunner {
        stdout: String::new(),
        detached_ok: false,
    };
    let err = CaptureProcess::run_detached(
        &runner,
        &["wf-recorder", "-f", "out.mp4"],
        ExecOpts::default(),
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("mock detached failure"));
}

#[tokio::test]
async fn capture_process_mock_runner_grim_region_chain() {
    let runner = MockCommandRunner {
        stdout: "10,20 30x40".into(),
        detached_ok: true,
    };
    let geometry = CaptureProcess::run(&runner, &["slurp"], ExecOpts::default())
        .await
        .unwrap();
    let grim_argv = grim_region_argv(&geometry);
    assert_eq!(grim_argv, vec!["grim", "-g", "10,20 30x40"]);
}
