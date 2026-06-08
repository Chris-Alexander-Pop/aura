use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref OFFENSIVE_SESSIONS: RwLock<HashMap<String, OffensiveSession>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OffensiveSession {
    session_id: String,
    name: String,
    target: String,
    created_at: i64,
    notes: Vec<String>,
    findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Finding {
    id: String,
    title: String,
    description: String,
    severity: String,
    category: String,
    timestamp: i64,
}

pub(crate) fn format_report_markdown(report: &serde_json::Value) -> String {
    format!("# Pentest Report\n\n{:?}", report)
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.Session.Create", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let session_id = format!("session_{}", chrono::Utc::now().timestamp_millis());
        let session = OffensiveSession {
            session_id: session_id.clone(),
            name,
            target,
            created_at: chrono::Utc::now().timestamp_millis(),
            notes: Vec::new(),
            findings: Vec::new(),
        };

        let mut sessions = OFFENSIVE_SESSIONS.write().await;
        sessions.insert(session_id.clone(), session.clone());

        storage::init().await?;
        storage::set_kv("offensive_sessions", &session_id, &serde_json::to_value(&session)?).await?;

        Ok(serde_json::to_value(&session)?)
    });

    registry.register("Security.Offensive.Session.List", |_params| async move {
        let sessions = OFFENSIVE_SESSIONS.read().await;
        let session_list: Vec<&OffensiveSession> = sessions.values().collect();
        Ok(serde_json::to_value(session_list)?)
    });

    registry.register("Security.Offensive.Session.Load", |params| async move {
        let session_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("session_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?,
        )?;

        storage::init().await?;
        if let Some(session_value) = storage::get_kv("offensive_sessions", &session_id).await? {
            let session: OffensiveSession = serde_json::from_value(session_value)?;
            let mut sessions = OFFENSIVE_SESSIONS.write().await;
            sessions.insert(session_id.clone(), session.clone());
            Ok(serde_json::to_value(&session)?)
        } else {
            anyhow::bail!("Session not found")
        }
    });

    registry.register("Security.Offensive.Notes.Add", |params| async move {
        let note: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("note").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing note"))?,
        )?;

        let category: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("category").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        storage::init().await?;
        let note_id = format!("note_{}", chrono::Utc::now().timestamp_millis());
        let note_data = serde_json::json!({
            "note": note,
            "category": category,
            "timestamp": chrono::Utc::now().timestamp_millis()
        });
        storage::set_kv("offensive_notes", &note_id, &note_data).await?;

        Ok(serde_json::json!({ "success": true, "note_id": note_id }))
    });

    registry.register("Security.Offensive.Screenshot.Capture", |params| async move {
        let url: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("url").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing url"))?,
        )?;

        let output_path: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("output_path").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let output = output_path.unwrap_or_else(|| format!("/tmp/screenshot_{}.png", chrono::Utc::now().timestamp_millis()));
        process::exec_command(&["cutycapt", "--url", &url, "--out", &output]).await?;

        Ok(serde_json::json!({ "success": true, "file": output }))
    });

    registry.register("Security.Offensive.Report.Create", |params| async move {
        let session_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("session_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?,
        )?;

        let template: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("template").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let sessions = OFFENSIVE_SESSIONS.read().await;
        if let Some(session) = sessions.get(&session_id) {
            let report = serde_json::json!({
                "session_id": session_id,
                "session_name": session.name,
                "target": session.target,
                "findings": session.findings,
                "created_at": chrono::Utc::now().timestamp_millis(),
                "template": template
            });

            storage::init().await?;
            let report_id = format!("report_{}", chrono::Utc::now().timestamp_millis());
            storage::set_kv("offensive_reports", &report_id, &report).await?;

            Ok(serde_json::json!({ "success": true, "report_id": report_id }))
        } else {
            anyhow::bail!("Session not found")
        }
    });

    registry.register("Security.Offensive.Report.Export", |params| async move {
        let report_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("report_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing report_id"))?,
        )?;

        let format: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("format").cloned())
                .unwrap_or(serde_json::Value::String("markdown".to_string())),
        )?;

        storage::init().await?;
        if let Some(report_value) = storage::get_kv("offensive_reports", &report_id).await? {
            match format.as_str() {
                "markdown" => Ok(serde_json::json!({
                    "markdown": format_report_markdown(&report_value)
                })),
                "json" => Ok(report_value),
                _ => anyhow::bail!("Unsupported format")
            }
        } else {
            anyhow::bail!("Report not found")
        }
    });

    registry.register("Security.Offensive.Report.AddFinding", |params| async move {
        let finding: Finding = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("finding").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing finding"))?,
        )?;

        let session_id: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("session_id").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        if let Some(ref sid) = session_id {
            let mut sessions = OFFENSIVE_SESSIONS.write().await;
            if let Some(session) = sessions.get_mut(sid) {
                session.findings.push(finding.clone());
                storage::init().await?;
                storage::set_kv("offensive_sessions", sid, &serde_json::to_value(session)?).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Security.Offensive.GetAuditLog", |params| async move {
        let limit: usize = params
            .as_ref()
            .and_then(|p| p.get("limit").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(50);
        let offset: usize = params
            .as_ref()
            .and_then(|p| p.get("offset").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(0);
        let entries = super::super::offensive_policy::get_audit_log(limit, offset).await?;
        Ok(serde_json::to_value(&entries)?)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_report_markdown_includes_heading() {
        let report = serde_json::json!({
            "session_id": "session_1",
            "target": "10.0.0.1"
        });
        let md = format_report_markdown(&report);
        assert!(md.starts_with("# Pentest Report"));
        assert!(md.contains("session_1"));
    }

    #[test]
    fn finding_serializes_expected_fields() {
        let finding = Finding {
            id: "f1".into(),
            title: "Open port".into(),
            description: "22/tcp".into(),
            severity: "low".into(),
            category: "network".into(),
            timestamp: 1,
        };
        let json = serde_json::to_value(&finding).expect("json");
        assert_eq!(json["severity"], "low");
        assert_eq!(json["category"], "network");
    }
}
