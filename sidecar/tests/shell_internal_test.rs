//! Desktop entry fixture contracts (session/launch helpers tested in shell.rs unit tests).

mod common;

use ags_sidecar::services::launcher::{build_desktop_index, parse_desktop_file};
use common::load_fixture;
use std::collections::HashSet;
use std::path::PathBuf;

#[test]
fn desktop_fixture_index_skips_hidden_and_parses_firefox() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/desktop");
    let firefox = dir.join("firefox.desktop");
    let entry = parse_desktop_file(&firefox).expect("firefox.desktop");
    assert_eq!(entry.id, "firefox");
    assert_eq!(entry.name, "Firefox");
    assert_eq!(entry.icon.as_deref(), Some("firefox"));

    let hidden = dir.join("hidden-app.desktop");
    assert!(parse_desktop_file(&hidden).is_none());

    let entries = build_desktop_index(&[dir]);
    let ids: HashSet<_> = entries.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains("firefox"));
    assert!(ids.contains("aura-test"));
    assert!(!ids.contains("hidden-app"));
}

#[test]
fn desktop_fixture_aura_test_exec_line() {
    let text = load_fixture("desktop/aura-test.desktop");
    assert!(text.contains("Exec=echo aura-test"));
    assert!(text.contains("Type=Application"));
}

#[test]
fn desktop_fixture_firefox_categories_and_keywords() {
    let text = load_fixture("desktop/firefox.desktop");
    assert!(text.contains("Categories=Network;WebBrowser;"));
    assert!(text.contains("Keywords=browser;web;"));
}
