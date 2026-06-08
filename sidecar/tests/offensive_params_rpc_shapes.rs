//! Missing-parameter validation across offensive modules (no subprocess / network).

#![cfg(feature = "offensive-security")]

mod common;

use common::{call_method_unchecked, test_registry};
use std::sync::Once;

static OFFENSIVE_TEST_HARNESS: Once = Once::new();

fn offensive_test_harness() {
    OFFENSIVE_TEST_HARNESS.call_once(|| {
        std::env::set_var("AURA_OFFENSIVE_SKIP_RATE_LIMIT", "1");
    });
}

#[tokio::test]
async fn offensive_dns_missing_domain_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.DNS.Query", None)
        .await
        .expect_err("missing domain");
    assert!(err.to_string().contains("Missing domain"));
}

#[tokio::test]
async fn offensive_portscan_missing_target_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.PortScan.TcpSyn", None)
        .await
        .expect_err("missing target");
    assert!(err.to_string().contains("Missing target"));
}

#[tokio::test]
async fn offensive_subdomain_missing_domain_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Security.Offensive.Subdomain.Enumerate",
        None,
    )
    .await
    .expect_err("missing domain");
    assert!(err.to_string().contains("Missing domain"));
}

#[tokio::test]
async fn offensive_cve_missing_cve_id_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.CVE.Search", None)
        .await
        .expect_err("missing cve_id");
    assert!(err.to_string().contains("Missing cve_id"));
}

#[tokio::test]
async fn offensive_osint_missing_email_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.OSINT.Email", None)
        .await
        .expect_err("missing email");
    assert!(err.to_string().contains("Missing email"));
}

#[tokio::test]
async fn offensive_exploitdb_missing_query_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(
        &registry,
        "Security.Offensive.ExploitDB.Search",
        None,
    )
    .await
    .expect_err("missing query");
    assert!(err.to_string().contains("Missing query"));
}

#[tokio::test]
async fn offensive_shell_missing_port_errors() {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, "Security.Offensive.Shell.Listen", None)
        .await
        .expect_err("missing port");
    assert!(err.to_string().contains("Missing port"));
}

async fn assert_offensive_missing(method: &str, params: Option<serde_json::Value>, needle: &str) {
    offensive_test_harness();
    let registry = test_registry();
    let err = call_method_unchecked(&registry, method, params)
        .await
        .expect_err("expected validation error");
    assert!(
        err.to_string().contains(needle),
        "method={method} err={err}"
    );
}

#[tokio::test]
async fn offensive_dns_zone_transfer_missing_domain_errors() {
    assert_offensive_missing("Security.Offensive.DNS.ZoneTransfer", None, "Missing domain").await;
}

#[tokio::test]
async fn offensive_dns_reverse_lookup_missing_ip_range_errors() {
    assert_offensive_missing(
        "Security.Offensive.DNS.ReverseLookup",
        None,
        "Missing ip_range",
    )
    .await;
}

#[tokio::test]
async fn offensive_portscan_udp_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.PortScan.Udp",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_portscan_service_version_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.PortScan.ServiceVersion",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_portscan_os_detection_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.PortScan.OsDetection",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_subdomain_bruteforce_missing_wordlist_errors() {
    assert_offensive_missing(
        "Security.Offensive.Subdomain.BruteForce",
        Some(serde_json::json!({ "domain": "example.com" })),
        "Missing wordlist",
    )
    .await;
}

#[tokio::test]
async fn offensive_subdomain_ct_missing_domain_errors() {
    assert_offensive_missing(
        "Security.Offensive.Subdomain.CertificateTransparency",
        None,
        "Missing domain",
    )
    .await;
}

#[tokio::test]
async fn offensive_cve_missing_product_errors() {
    assert_offensive_missing(
        "Security.Offensive.CVE.SearchByProduct",
        None,
        "Missing product",
    )
    .await;
}

#[tokio::test]
async fn offensive_osint_missing_ip_errors() {
    assert_offensive_missing("Security.Offensive.OSINT.IP", None, "Missing ip").await;
}

#[tokio::test]
async fn offensive_osint_missing_domain_errors() {
    assert_offensive_missing("Security.Offensive.OSINT.Domain", None, "Missing domain").await;
}

#[tokio::test]
async fn offensive_exploitdb_get_missing_exploit_id_errors() {
    assert_offensive_missing(
        "Security.Offensive.ExploitDB.GetExploit",
        None,
        "Missing exploit_id",
    )
    .await;
}

#[tokio::test]
async fn offensive_exploitdb_search_by_cve_missing_cve_errors() {
    assert_offensive_missing(
        "Security.Offensive.ExploitDB.SearchByCVE",
        None,
        "Missing cve",
    )
    .await;
}

#[tokio::test]
async fn offensive_metasploit_search_missing_query_errors() {
    assert_offensive_missing(
        "Security.Offensive.Metasploit.SearchExploit",
        None,
        "Missing query",
    )
    .await;
}

#[tokio::test]
async fn offensive_metasploit_use_module_missing_path_errors() {
    assert_offensive_missing(
        "Security.Offensive.Metasploit.UseModule",
        None,
        "Missing module_path",
    )
    .await;
}

#[tokio::test]
async fn offensive_payload_msvenom_missing_type_errors() {
    assert_offensive_missing(
        "Security.Offensive.Payload.Msvenom",
        Some(serde_json::json!({ "lhost": "127.0.0.1", "lport": 4444 })),
        "Missing type",
    )
    .await;
}

#[tokio::test]
async fn offensive_payload_msvenom_missing_lhost_errors() {
    assert_offensive_missing(
        "Security.Offensive.Payload.Msvenom",
        Some(serde_json::json!({ "type": "linux/x64/shell_reverse_tcp", "lport": 4444 })),
        "Missing lhost",
    )
    .await;
}

#[tokio::test]
async fn offensive_payload_msvenom_missing_lport_errors() {
    assert_offensive_missing(
        "Security.Offensive.Payload.Msvenom",
        Some(serde_json::json!({ "type": "linux/x64/shell_reverse_tcp", "lhost": "127.0.0.1" })),
        "Missing lport",
    )
    .await;
}

#[tokio::test]
async fn offensive_hashcat_missing_hash_file_errors() {
    assert_offensive_missing(
        "Security.Offensive.Password.Hashcat.Crack",
        Some(serde_json::json!({ "wordlist": "/tmp/wl.txt" })),
        "Missing hash_file",
    )
    .await;
}

#[tokio::test]
async fn offensive_john_missing_hash_file_errors() {
    assert_offensive_missing(
        "Security.Offensive.Password.John.Crack",
        None,
        "Missing hash_file",
    )
    .await;
}

#[tokio::test]
async fn offensive_hash_identify_missing_hash_errors() {
    assert_offensive_missing(
        "Security.Offensive.Password.Hash.Identify",
        None,
        "Missing hash",
    )
    .await;
}

#[tokio::test]
async fn offensive_wireless_capture_missing_interface_errors() {
    assert_offensive_missing(
        "Security.Offensive.Wireless.CaptureHandshake",
        Some(serde_json::json!({ "network": "6" })),
        "Missing interface",
    )
    .await;
}

#[tokio::test]
async fn offensive_wireless_wps_missing_network_errors() {
    assert_offensive_missing(
        "Security.Offensive.Wireless.WPS.Test",
        None,
        "Missing network",
    )
    .await;
}

#[tokio::test]
async fn offensive_social_qrcode_missing_url_errors() {
    assert_offensive_missing(
        "Security.Offensive.Social.QRCode.Generate",
        Some(serde_json::json!({ "output_path": "/tmp/qr.png" })),
        "Missing url",
    )
    .await;
}

#[tokio::test]
async fn offensive_forensics_strings_missing_file_errors() {
    assert_offensive_missing(
        "Security.Offensive.Forensics.File.Strings",
        None,
        "Missing file_path",
    )
    .await;
}

#[tokio::test]
async fn offensive_stego_missing_image_errors() {
    assert_offensive_missing(
        "Security.Offensive.Stego.Extract",
        None,
        "Missing image_path",
    )
    .await;
}

#[tokio::test]
async fn offensive_tcpdump_missing_output_errors() {
    assert_offensive_missing(
        "Security.Offensive.Network.Tcpdump.Capture",
        Some(serde_json::json!({ "interface": "lo" })),
        "Missing output",
    )
    .await;
}

#[tokio::test]
async fn offensive_arp_spoof_missing_gateway_errors() {
    assert_offensive_missing(
        "Security.Offensive.Network.ArpSpoof",
        Some(serde_json::json!({ "target": "10.0.0.2" })),
        "Missing gateway",
    )
    .await;
}

#[tokio::test]
async fn offensive_mitm_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.Network.Mitm.Start",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_session_create_missing_name_errors() {
    assert_offensive_missing(
        "Security.Offensive.Session.Create",
        Some(serde_json::json!({ "target": "10.0.0.1" })),
        "Missing name",
    )
    .await;
}

#[tokio::test]
async fn offensive_session_load_missing_session_id_errors() {
    assert_offensive_missing(
        "Security.Offensive.Session.Load",
        None,
        "Missing session_id",
    )
    .await;
}

#[tokio::test]
async fn offensive_screenshot_capture_missing_url_errors() {
    assert_offensive_missing(
        "Security.Offensive.Screenshot.Capture",
        None,
        "Missing url",
    )
    .await;
}

#[tokio::test]
async fn offensive_report_export_missing_report_id_errors() {
    assert_offensive_missing(
        "Security.Offensive.Report.Export",
        None,
        "Missing report_id",
    )
    .await;
}

#[tokio::test]
async fn offensive_report_add_finding_missing_finding_errors() {
    assert_offensive_missing(
        "Security.Offensive.Report.AddFinding",
        Some(serde_json::json!({ "session_id": "session_x" })),
        "Missing finding",
    )
    .await;
}

#[tokio::test]
async fn offensive_web_tech_stack_missing_url_errors() {
    assert_offensive_missing(
        "Security.Offensive.Web.TechStack",
        None,
        "Missing url",
    )
    .await;
}

#[tokio::test]
async fn offensive_web_screenshot_missing_urls_errors() {
    assert_offensive_missing(
        "Security.Offensive.Web.Screenshot",
        None,
        "Missing urls",
    )
    .await;
}

#[tokio::test]
async fn offensive_web_sqli_sqlmap_missing_url_errors() {
    assert_offensive_missing(
        "Security.Offensive.Web.SQLi.Sqlmap",
        None,
        "Missing url",
    )
    .await;
}

#[tokio::test]
async fn offensive_web_xss_test_missing_parameter_errors() {
    assert_offensive_missing(
        "Security.Offensive.Web.XSS.Test",
        Some(serde_json::json!({ "url": "http://127.0.0.1" })),
        "Missing parameter",
    )
    .await;
}

#[tokio::test]
async fn offensive_web_csrf_missing_action_errors() {
    assert_offensive_missing(
        "Security.Offensive.Web.CSRF.Test",
        Some(serde_json::json!({ "url": "http://127.0.0.1" })),
        "Missing action",
    )
    .await;
}

#[tokio::test]
async fn offensive_nmap_quick_scan_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.Nmap.QuickScan",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_nmap_full_scan_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.Nmap.FullScan",
        None,
        "Missing target",
    )
    .await;
}

#[tokio::test]
async fn offensive_nmap_scan_results_missing_scan_id_errors() {
    assert_offensive_missing(
        "Security.Offensive.Nmap.ScanResults",
        None,
        "Missing scan_id",
    )
    .await;
}

#[tokio::test]
async fn offensive_nmap_save_results_missing_scan_id_errors() {
    assert_offensive_missing(
        "Security.Offensive.Nmap.SaveResults",
        None,
        "Missing scan_id",
    )
    .await;
}

#[tokio::test]
async fn offensive_vulnscan_missing_target_errors() {
    assert_offensive_missing(
        "Security.Offensive.VulnScan.NmapScripts",
        None,
        "Missing target",
    )
    .await;
}
