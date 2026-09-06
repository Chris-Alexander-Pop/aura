//! Golden `pacman` fixture contracts (malformed lines and repo-shaped search output).

mod common;

use ags_sidecar::contract_parsers::{parse_pacman_q, parse_pacman_qu, parse_pacman_search};
use common::load_fixture;

#[test]
fn contract_pacman_qu_valid_and_malformed_lines() {
    let text = load_fixture("packages/pacman_qu_malformed.txt");
    let pkgs = parse_pacman_qu(&text);
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0].name, "linux");
    assert_eq!(pkgs[0].version, "6.12.1-1");
    assert!(pkgs[0].installed);
}

#[test]
fn contract_pacman_qu_empty_fixture() {
    let text = load_fixture("packages/pacman_qu_empty.txt");
    assert!(parse_pacman_qu(&text).is_empty());
}

#[test]
fn contract_pacman_q_installed_fixture() {
    let text = load_fixture("packages/pacman_q.txt");
    let pkgs = parse_pacman_q(&text);
    assert_eq!(pkgs.len(), 3);
    assert_eq!(pkgs[2].name, "vim");
    assert_eq!(pkgs[2].version, "9.1-1");
}

#[test]
fn contract_pacman_search_repo_lines_fixture() {
    let text = load_fixture("packages/pacman_ss.txt");
    let pkgs = parse_pacman_search(&text);
    assert_eq!(pkgs.len(), 2);
    assert_eq!(pkgs[0].name, "firefox");
    assert!(!pkgs[0].installed);
}

#[test]
fn contract_pacman_search_malformed_lines_ignored() {
    let text = load_fixture("packages/pacman_ss_malformed.txt");
    assert!(parse_pacman_search(&text).is_empty());
}
