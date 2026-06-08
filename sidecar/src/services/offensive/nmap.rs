use crate::services::security::{parse_nmap_xml, NmapPort};
use crate::services::ServiceRegistry;
use crate::utils::process;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref NMAP_SCANS: RwLock<HashMap<String, NmapScanResult>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NmapScanResult {
    scan_id: String,
    target: String,
    status: String,
    ports: Vec<NmapPort>,
    hosts: Vec<NmapHost>,
    output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NmapHost {
    ip: String,
    hostname: Option<String>,
    os: Option<String>,
}

/// Run nmap and read `-oX` output, or use `AURA_NMAP_XML_FIXTURE` (tests; no live nmap).
pub(crate) async fn execute_nmap_with_xml_output(scan_id: &str, cmd: &[&str]) -> Result<(String, String)> {
    let output_file = format!("/tmp/nmap_{}.xml", scan_id);
    if let Ok(fixture) = std::env::var("AURA_NMAP_XML_FIXTURE") {
        let xml_content = tokio::fs::read_to_string(&fixture).await?;
        tokio::fs::write(&output_file, &xml_content).await?;
        Ok((
            "nmap skipped (AURA_NMAP_XML_FIXTURE)".to_string(),
            xml_content,
        ))
    } else {
        let output = process::exec_command(cmd).await?;
        let xml_content = tokio::fs::read_to_string(&output_file).await?;
        Ok((output, xml_content))
    }
}

pub(crate) fn build_nmap_scan_result(
    scan_id: String,
    target: String,
    ports: Vec<NmapPort>,
    output: String,
) -> NmapScanResult {
    NmapScanResult {
        scan_id,
        target,
        status: "completed".to_string(),
        ports,
        hosts: Vec::new(),
        output,
    }
}

pub(crate) fn nmap_scan_summaries(scans: &HashMap<String, NmapScanResult>) -> Vec<serde_json::Value> {
    let mut entries: Vec<serde_json::Value> = scans
        .values()
        .map(|r| {
            serde_json::json!({
                "scan_id": r.scan_id,
                "target": r.target,
                "status": r.status,
                "port_count": r.ports.len(),
            })
        })
        .collect();
    entries.sort_by(|a, b| {
        let a_id = a["scan_id"].as_str().unwrap_or("");
        let b_id = b["scan_id"].as_str().unwrap_or("");
        b_id.cmp(a_id)
    });
    entries
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.Nmap.Scan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_type").cloned())
                .unwrap_or(serde_json::Value::String("syn".to_string())),
        )?;

        let ports: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("ports").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let options: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("options").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let mut cmd = vec!["nmap"];
        match scan_type.as_str() {
            "syn" => cmd.push("-sS"),
            "udp" => cmd.push("-sU"),
            "full" => {
                cmd.push("-sS");
                cmd.push("-sV");
                cmd.push("-O");
            }
            _ => {}
        }

        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }

        cmd.extend_from_slice(&["-oX", &output_file]);
        if let Some(ref opts) = options {
            for opt in opts.split_whitespace() {
                cmd.push(opt);
            }
        }
        cmd.push(&target);

        let (output, xml_content) = execute_nmap_with_xml_output(&scan_id, &cmd).await?;
        let ports = parse_nmap_xml(&xml_content)?;
        let result = build_nmap_scan_result(scan_id.clone(), target, ports, output);

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.QuickScan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let (output, xml_content) = execute_nmap_with_xml_output(
            &scan_id,
            &["nmap", "-F", "-sS", "-oX", &output_file, &target],
        )
        .await?;
        let ports = parse_nmap_xml(&xml_content)?;
        let result = build_nmap_scan_result(scan_id.clone(), target, ports, output);

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.FullScan", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let scan_id = format!("scan_{}", chrono::Utc::now().timestamp_millis());
        let output_file = format!("/tmp/nmap_{}.xml", scan_id);

        let (output, xml_content) = execute_nmap_with_xml_output(
            &scan_id,
            &[
                "nmap", "-sS", "-sV", "-O", "-A", "-oX", &output_file, &target,
            ],
        )
        .await?;
        let ports = parse_nmap_xml(&xml_content)?;
        let result = build_nmap_scan_result(scan_id.clone(), target, ports, output);

        let mut scans = NMAP_SCANS.write().await;
        scans.insert(scan_id.clone(), result.clone());

        Ok(serde_json::to_value(&result)?)
    });

    registry.register("Security.Offensive.Nmap.ListScans", |_params| async move {
        let scans = NMAP_SCANS.read().await;
        Ok(serde_json::to_value(&nmap_scan_summaries(&scans))?)
    });

    registry.register("Security.Offensive.Nmap.ScanResults", |params| async move {
        let scan_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing scan_id"))?,
        )?;

        let scans = NMAP_SCANS.read().await;
        if let Some(result) = scans.get(&scan_id) {
            Ok(serde_json::to_value(result)?)
        } else {
            anyhow::bail!("Scan not found")
        }
    });

    registry.register("Security.Offensive.Nmap.SaveResults", |params| async move {
        let scan_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scan_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing scan_id"))?,
        )?;

        let format: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("format").cloned())
                .unwrap_or(serde_json::Value::String("json".to_string())),
        )?;

        let scans = NMAP_SCANS.read().await;
        if let Some(result) = scans.get(&scan_id) {
            match format.as_str() {
                "json" => Ok(serde_json::to_value(result)?),
                "xml" => {
                    let xml_file = format!("/tmp/nmap_{}.xml", scan_id);
                    if let Ok(content) = tokio::fs::read_to_string(&xml_file).await {
                        Ok(serde_json::json!({ "xml": content }))
                    } else {
                        anyhow::bail!("XML file not found")
                    }
                }
                _ => anyhow::bail!("Unsupported format")
            }
        } else {
            anyhow::bail!("Scan not found")
        }
    });

    registry.register("Security.Offensive.VulnScan.NmapScripts", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let script_category: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("script_category").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "--script"];
        if let Some(ref cat) = script_category {
            cmd.push(cat);
        } else {
            cmd.push("vuln");
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_nmap_scan_result_completed_shape() {
        let fixture = format!(
            "{}/tests/fixtures/security/nmap_minimal.xml",
            env!("CARGO_MANIFEST_DIR")
        );
        let xml = std::fs::read_to_string(&fixture).expect("fixture");
        let ports = parse_nmap_xml(&xml).expect("parse");
        let result = build_nmap_scan_result(
            "scan_1".into(),
            "10.0.0.1".into(),
            ports,
            "stdout".into(),
        );
        assert_eq!(result.scan_id, "scan_1");
        assert_eq!(result.target, "10.0.0.1");
        assert_eq!(result.status, "completed");
        assert_eq!(result.ports.len(), 2);
        assert!(result.hosts.is_empty());
    }

    #[test]
    fn nmap_scan_summaries_newest_scan_id_first() {
        let mut scans = HashMap::new();
        scans.insert(
            "a".into(),
            build_nmap_scan_result("scan_100".into(), "t1".into(), vec![], "".into()),
        );
        scans.insert(
            "b".into(),
            build_nmap_scan_result("scan_200".into(), "t2".into(), vec![], "".into()),
        );
        let summaries = nmap_scan_summaries(&scans);
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0]["scan_id"], "scan_200");
        assert_eq!(summaries[0]["target"], "t2");
        assert_eq!(summaries[0]["status"], "completed");
        assert_eq!(summaries[0]["port_count"], 0);
    }

    #[tokio::test]
    async fn execute_nmap_fixture_skips_live_command() {
        let fixture = format!(
            "{}/tests/fixtures/security/nmap_minimal.xml",
            env!("CARGO_MANIFEST_DIR")
        );
        std::env::set_var("AURA_NMAP_XML_FIXTURE", &fixture);
        let scan_id = "scan_unit_fixture";
        let (output, xml) = execute_nmap_with_xml_output(scan_id, &["nmap", "-sn", "127.0.0.1"])
            .await
            .expect("fixture scan");
        assert!(output.contains("AURA_NMAP_XML_FIXTURE"));
        assert!(xml.contains("<nmaprun>"));
        let ports = parse_nmap_xml(&xml).expect("parse");
        assert_eq!(ports.len(), 2);
        let first = serde_json::to_value(&ports[0]).expect("port json");
        assert_eq!(first["port"], 22);
        std::env::remove_var("AURA_NMAP_XML_FIXTURE");
    }
}
