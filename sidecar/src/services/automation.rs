use crate::services::ServiceRegistry;
use crate::utils::{automation_log, process, storage};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::sync::Once;
use tokio::time::{timeout, Duration};

static CRON_TICK: Once = Once::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub triggers: serde_json::Value,
    pub actions: serde_json::Value,
    pub created_at: i64,
    #[serde(default)]
    pub run_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub content: String,
    pub interpreter: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    let list_workflows = |_params| async move {
        storage::init().await?;
        let items = storage::scan_namespace("automation_workflows").await?;
        Ok(serde_json::to_value(&workflows_from_storage_values(items))?)
    };

    registry.register("Automation.GetWorkflows", list_workflows.clone());
    registry.register("Automation.ListRules", list_workflows);

    registry.register("Automation.CreateWorkflow", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow!("Missing name"))?,
        )?;

        let triggers: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("triggers").cloned())
            .unwrap_or(serde_json::json!([]));

        let actions: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("actions").cloned())
            .ok_or_else(|| anyhow!("Missing actions"))?;

        validate_workflow_triggers(&triggers)?;
        validate_workflow_actions(&actions)?;

        let workflow_id = format!("workflow_{}", chrono::Utc::now().timestamp_millis());
        let workflow = Workflow {
            id: workflow_id.clone(),
            name,
            enabled: true,
            triggers,
            actions,
            created_at: chrono::Utc::now().timestamp_millis(),
            run_count: 0,
            last_run_at: None,
        };

        storage::init().await?;
        storage::set_kv(
            "automation_workflows",
            &workflow_id,
            &serde_json::to_value(&workflow)?,
        )
        .await?;
        spawn_cron_tick();
        Ok(serde_json::to_value(&workflow)?)
    });

    registry.register("Automation.UpdateWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow!("Missing workflow_id"))?,
        )?;

        let updates: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("updates").cloned())
            .ok_or_else(|| anyhow!("Missing updates"))?;

        storage::init().await?;
        let Some(mut workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await?
        else {
            return Err(anyhow!("Workflow not found"));
        };
        if let Some(triggers) = updates.get("triggers") {
            validate_workflow_triggers(triggers)?;
        }
        if let Some(actions) = updates.get("actions") {
            validate_workflow_actions(actions)?;
        }
        if let Some(workflow_obj) = workflow_value.as_object_mut() {
            for (key, value) in updates.as_object().unwrap() {
                workflow_obj.insert(key.clone(), value.clone());
            }
            storage::set_kv("automation_workflows", &workflow_id, &workflow_value).await?;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.DeleteWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow!("Missing workflow_id"))?,
        )?;

        storage::init().await?;
        let deleted = storage::delete_kv("automation_workflows", &workflow_id).await?;
        Ok(serde_json::json!({ "success": true, "deleted": deleted }))
    });

    registry.register("Automation.EnableWorkflow", |params| async move {
        set_workflow_enabled(params, true).await
    });

    registry.register("Automation.DisableWorkflow", |params| async move {
        set_workflow_enabled(params, false).await
    });

    let run_workflow = |params: Option<serde_json::Value>| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow!("Missing workflow_id"))?,
        )?;
        run_workflow_by_id(&workflow_id).await
    };

    registry.register("Automation.RunWorkflow", run_workflow.clone());
    registry.register("Automation.Trigger", run_workflow);

    registry.register("Automation.GetWorkflowHistory", |params| async move {
        let workflow_id: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("workflow_id").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let limit: usize = params
            .as_ref()
            .and_then(|p| p.get("limit").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(50);

        let runs = automation_log::read_workflow_runs(workflow_id.as_deref(), limit).await?;
        Ok(serde_json::to_value(runs)?)
    });

    registry.register("Automation.CreateScript", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow!("Missing name"))?,
        )?;

        let content: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("content").cloned())
                .ok_or_else(|| anyhow!("Missing content"))?,
        )?;

        let interpreter: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("interpreter").cloned())
                .unwrap_or(serde_json::Value::String("bash".to_string())),
        )?;

        let script_id = format!("script_{}", chrono::Utc::now().timestamp_millis());
        let script = Script {
            id: script_id.clone(),
            name,
            content,
            interpreter,
        };

        storage::init().await?;
        storage::set_kv("automation_scripts", &script_id, &serde_json::to_value(&script)?).await?;
        Ok(serde_json::to_value(&script)?)
    });

    registry.register("Automation.GetScripts", |_params| async move {
        storage::init().await?;
        let items = storage::scan_namespace("automation_scripts").await?;
        Ok(serde_json::to_value(&scripts_from_storage_values(items))?)
    });

    registry.register("Automation.RunScript", |params| async move {
        let script_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("script_id").cloned())
                .ok_or_else(|| anyhow!("Missing script_id"))?,
        )?;

        storage::init().await?;
        let script_value = storage::get_kv("automation_scripts", &script_id)
            .await?
            .ok_or_else(|| anyhow!("Script not found"))?;
        let script: Script = serde_json::from_value(script_value)?;
        let output = run_script_file(&script).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Automation.GetTriggers", |_params| async move {
        Ok(serde_json::json!([
            "manual",
            "time",
            "file_change",
            "system_event",
            "network_event"
        ]))
    });

    registry.register("Automation.GetActions", |_params| async move {
        Ok(serde_json::json!([
            "execute_command",
            "send_notification",
            "run_script"
        ]))
    });

    spawn_cron_tick();
}

async fn set_workflow_enabled(
    params: Option<serde_json::Value>,
    enabled: bool,
) -> Result<serde_json::Value> {
    let workflow_id: String = serde_json::from_value(
        params
            .as_ref()
            .and_then(|p| p.get("workflow_id").cloned())
            .ok_or_else(|| anyhow!("Missing workflow_id"))?,
    )?;

    storage::init().await?;
    let Some(mut workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await? else {
        return Err(anyhow!("Workflow not found"));
    };
    if let Some(workflow_obj) = workflow_value.as_object_mut() {
        workflow_obj.insert("enabled".to_string(), serde_json::json!(enabled));
        storage::set_kv("automation_workflows", &workflow_id, &workflow_value).await?;
    }

    Ok(serde_json::json!({ "success": true }))
}

pub async fn run_workflow_by_id(workflow_id: &str) -> Result<serde_json::Value> {
    storage::init().await?;
    let workflow_value = storage::get_kv("automation_workflows", workflow_id)
        .await?
        .ok_or_else(|| anyhow!("Workflow not found"))?;
    let workflow: Workflow = serde_json::from_value(workflow_value)?;
    if !workflow.enabled {
        return Err(anyhow!("Workflow is disabled"));
    }

    let result = timeout(
        Duration::from_secs(30),
        execute_actions(&workflow.actions),
    )
    .await;

    let run_finished_at = Utc::now().timestamp_millis();
    match result {
        Ok(Ok(())) => {
            automation_log::log_workflow_run(workflow_id, true, None).await;
            record_workflow_run(workflow_id, run_finished_at).await?;
            Ok(serde_json::json!({ "success": true }))
        }
        Ok(Err(e)) => {
            let msg = e.to_string();
            automation_log::log_workflow_run(workflow_id, false, Some(msg.clone())).await;
            record_workflow_run(workflow_id, run_finished_at).await.ok();
            Err(anyhow!(msg))
        }
        Err(_) => {
            let msg = "Workflow execution timed out after 30s".to_string();
            automation_log::log_workflow_run(workflow_id, false, Some(msg.clone())).await;
            record_workflow_run(workflow_id, run_finished_at).await.ok();
            Err(anyhow!(msg))
        }
    }
}

async fn record_workflow_run(workflow_id: &str, at_ms: i64) -> Result<()> {
    let Some(mut workflow_value) = storage::get_kv("automation_workflows", workflow_id).await? else {
        return Ok(());
    };
    if let Some(obj) = workflow_value.as_object_mut() {
        let count = obj
            .get("run_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .saturating_add(1);
        obj.insert("run_count".to_string(), serde_json::json!(count));
        obj.insert("last_run_at".to_string(), serde_json::json!(at_ms));
        storage::set_kv("automation_workflows", workflow_id, &workflow_value).await?;
    }
    Ok(())
}

pub fn spawn_cron_tick() {
    CRON_TICK.call_once(|| {
        tokio::spawn(async {
            let mut tick = tokio::time::interval(Duration::from_secs(60));
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tick.tick().await;
                if let Err(e) = automation_cron_tick().await {
                    tracing::debug!("automation cron tick: {}", e);
                }
            }
        });
    });
}

async fn automation_cron_tick() -> Result<()> {
    storage::init().await?;
    let items = storage::scan_namespace("automation_workflows").await?;
    let workflows = workflows_from_storage_values(items);
    let now = Utc::now();
    for workflow in workflows {
        if !workflow.enabled {
            continue;
        }
        let Some(cron_expr) = workflow_cron_expr(&workflow.triggers) else {
            continue;
        };
        let after_ms = workflow.last_run_at.unwrap_or(workflow.created_at);
        let after = DateTime::from_timestamp_millis(after_ms).unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0).unwrap_or_else(|| Utc::now())
        });
        let Some(next) = cron_next_fire_after(&cron_expr, after) else {
            continue;
        };
        if next > now {
            continue;
        }
        let _ = run_workflow_by_id(&workflow.id).await;
    }
    Ok(())
}

fn workflow_cron_expr(triggers: &serde_json::Value) -> Option<String> {
    let arr = triggers.as_array()?;
    for trigger in arr {
        let obj = trigger.as_object()?;
        if obj.get("type").and_then(|v| v.as_str()) == Some("time") {
            return obj
                .get("cron")
                .and_then(|v| v.as_str())
                .map(str::to_string);
        }
    }
    None
}

pub(crate) fn workflows_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Workflow> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub(crate) fn scripts_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Script> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub(crate) fn validate_workflow_triggers(triggers: &serde_json::Value) -> Result<()> {
    let Some(triggers_array) = triggers.as_array() else {
        return Err(anyhow!("triggers must be an array"));
    };
    for trigger in triggers_array {
        let trigger_obj = trigger
            .as_object()
            .ok_or_else(|| anyhow!("each trigger must be an object"))?;
        let trigger_type = trigger_obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("trigger missing type"))?;
        match trigger_type {
            "manual" | "system_event" | "network_event" | "file_change" => {}
            "time" => {
                let cron = trigger_obj
                    .get("cron")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("time trigger missing cron"))?;
                if cron_next_fire_after(cron, Utc::now()).is_none() {
                    return Err(anyhow!("invalid cron expression: {}", cron));
                }
            }
            other => return Err(anyhow!("unknown trigger type: {}", other)),
        }
    }
    Ok(())
}

/// Next fire time strictly after `after` for a five-field cron (minute hour dom month dow).
pub fn cron_next_fire_after(expr: &str, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return None;
    }
    let mut probe = after + chrono::Duration::minutes(1);
    probe = probe
        .with_second(0)
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(probe);
    for _ in 0..(48 * 60) {
        if cron_matches(&parts, probe) {
            return Some(probe);
        }
        probe += chrono::Duration::minutes(1);
    }
    None
}

fn cron_matches(parts: &[&str], t: DateTime<Utc>) -> bool {
    cron_field_matches(parts[0], t.minute() as u32, 0, 59)
        && cron_field_matches(parts[1], t.hour() as u32, 0, 23)
        && cron_field_matches(parts[2], t.day() as u32, 1, 31)
        && cron_field_matches(parts[3], t.month() as u32, 1, 12)
        && cron_field_matches(parts[4], t.weekday().num_days_from_sunday(), 0, 6)
}

fn cron_field_matches(field: &str, value: u32, min: u32, max: u32) -> bool {
    if field == "*" {
        return true;
    }
    if let Some(step_s) = field.strip_prefix("*/") {
        let step: u32 = match step_s.parse() {
            Ok(n) if n > 0 => n,
            _ => return false,
        };
        return value >= min && value <= max && value % step == 0;
    }
    if let Ok(exact) = field.parse::<u32>() {
        return value == exact;
    }
    false
}

pub(crate) fn validate_workflow_actions(actions: &serde_json::Value) -> Result<()> {
    let Some(actions_array) = actions.as_array() else {
        return Err(anyhow!("actions must be an array"));
    };
    for action in actions_array {
        let action_obj = action
            .as_object()
            .ok_or_else(|| anyhow!("each action must be an object"))?;
        let action_type = action_obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("action missing type"))?;
        match action_type {
            "execute_command" => {
                let command = action_obj
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("execute_command missing command"))?;
                if !command_allowed(command) {
                    return Err(anyhow!("command not allowed: {}", command));
                }
            }
            "send_notification" => {}
            "run_script" => {
                let _script_id = action_obj
                    .get("script_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("run_script missing script_id"))?;
            }
            other => return Err(anyhow!("unknown action type: {}", other)),
        }
    }
    Ok(())
}

pub(crate) fn command_allowed(command: &str) -> bool {
    let c = command.trim();
    if c.is_empty() {
        return false;
    }
    let lower = c.to_lowercase();
    const DENIED: &[&str] = &[
        "rm -rf",
        "sudo ",
        "sudo\t",
        "pkexec",
        "shutdown",
        "reboot",
        "poweroff",
        "init 0",
        "mkfs",
        "dd if=",
        ":(){",
        "chmod -r",
        "> /dev/",
    ];
    for d in DENIED {
        if lower.contains(d) {
            return false;
        }
    }
    if c.starts_with("notify-send") {
        return true;
    }
    if let Ok(scripts_dir) = automation_scripts_dir() {
        if let Some(path) = extract_script_path(c) {
            if path.starts_with(&scripts_dir) {
                return true;
            }
        }
    }
    false
}

fn extract_script_path(command: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    parts.last().map(PathBuf::from)
}

pub(crate) fn automation_scripts_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("AURA_AUTOMATION_SCRIPTS_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ags")
        .join("automation")
        .join("scripts"))
}

async fn execute_actions(actions: &serde_json::Value) -> Result<()> {
    let Some(actions_array) = actions.as_array() else {
        return Ok(());
    };
    for action in actions_array {
        let Some(action_obj) = action.as_object() else {
            continue;
        };
        let Some(action_type) = action_obj.get("type").and_then(|v| v.as_str()) else {
            continue;
        };
        match action_type {
            "execute_command" => {
                if let Some(command) = action_obj.get("command").and_then(|v| v.as_str()) {
                    if command_allowed(command) {
                        let parts: Vec<&str> = command.split_whitespace().collect();
                        if !parts.is_empty() {
                            process::exec_command(&parts).await?;
                        }
                    } else {
                        return Err(anyhow!("command not allowed"));
                    }
                }
            }
            "send_notification" => {
                if let (Some(title), Some(body)) = (
                    action_obj.get("title").and_then(|v| v.as_str()),
                    action_obj.get("body").and_then(|v| v.as_str()),
                ) {
                    process::exec_command(&["notify-send", title, body]).await.ok();
                }
            }
            "run_script" => {
                if let Some(script_id) = action_obj.get("script_id").and_then(|v| v.as_str()) {
                    storage::init().await?;
                    if let Some(script_value) = storage::get_kv("automation_scripts", script_id).await?
                    {
                        let script: Script = serde_json::from_value(script_value)?;
                        run_script_file(&script).await?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

async fn run_script_file(script: &Script) -> Result<String> {
    let scripts_dir = automation_scripts_dir()?;
    tokio::fs::create_dir_all(&scripts_dir).await?;
    let path = scripts_dir.join(format!("{}.sh", script.id));
    tokio::fs::write(&path, &script.content).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&path).await?.permissions();
        perms.set_mode(0o700);
        tokio::fs::set_permissions(&path, perms).await?;
    }
    let cmd = format!("{} {}", script.interpreter, path.display());
    if !command_allowed(&cmd) {
        return Err(anyhow!("script command not allowed"));
    }
    let output = process::exec_command(&[script.interpreter.as_str(), path.to_str().unwrap()]).await?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn workflows_from_storage_skips_corrupt_entries() {
        let valid = Workflow {
            id: "w1".into(),
            name: "n".into(),
            enabled: true,
            triggers: json!([]),
            actions: json!([]),
            created_at: 1,
            run_count: 0,
            last_run_at: None,
        };
        let items = vec![
            serde_json::to_value(&valid).unwrap(),
            json!({ "not_a_workflow": true }),
            json!("bare string"),
        ];
        let out = workflows_from_storage_values(items);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "w1");
    }

    #[test]
    fn command_allowed_rejects_destructive() {
        assert!(!command_allowed("sudo rm -rf /"));
        assert!(!command_allowed("rm -rf /tmp/x"));
        assert!(command_allowed("notify-send Test body"));
    }

    #[test]
    fn validate_workflow_rejects_bad_action() {
        let actions = json!([{ "type": "execute_command", "command": "rm -rf /" }]);
        assert!(validate_workflow_actions(&actions).is_err());
    }

    #[test]
    fn validate_workflow_accepts_notify() {
        let actions = json!([{ "type": "send_notification", "title": "t", "body": "b" }]);
        assert!(validate_workflow_actions(&actions).is_ok());
    }

    #[test]
    fn validate_workflow_rejects_bad_triggers() {
        assert!(validate_workflow_triggers(&json!("x")).is_err());
        assert!(validate_workflow_triggers(&json!([{ "type": "time" }])).is_err());
        assert!(validate_workflow_triggers(&json!([{ "type": "time", "cron": "not cron" }])).is_err());
    }

    #[test]
    fn validate_workflow_accepts_time_trigger() {
        let triggers = json!([{ "type": "time", "cron": "*/5 * * * *" }]);
        assert!(validate_workflow_triggers(&triggers).is_ok());
    }

    #[test]
    fn cron_next_fire_steps_every_five_minutes() {
        let after = Utc.with_ymd_and_hms(2025, 5, 1, 10, 3, 0).unwrap();
        let next = cron_next_fire_after("*/5 * * * *", after).unwrap();
        assert_eq!(next.minute(), 5);
        assert_eq!(next.hour(), 10);
    }

    #[test]
    fn cron_next_fire_fixed_hour() {
        let after = Utc.with_ymd_and_hms(2025, 5, 1, 8, 30, 0).unwrap();
        let next = cron_next_fire_after("0 9 * * *", after).unwrap();
        assert_eq!(next.hour(), 9);
        assert_eq!(next.minute(), 0);
    }
}
