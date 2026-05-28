use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub triggers: serde_json::Value,
    pub actions: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub content: String,
    pub interpreter: String,
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Automation.CreateWorkflow", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let triggers: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("triggers").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing triggers"))?;

        let actions: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("actions").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing actions"))?;

        let workflow_id = format!("workflow_{}", chrono::Utc::now().timestamp_millis());
        let workflow = Workflow {
            id: workflow_id.clone(),
            name,
            enabled: true,
            triggers,
            actions,
            created_at: chrono::Utc::now().timestamp_millis(),
        };

        storage::init().await?;
        storage::set_kv("automation_workflows", &workflow_id, &serde_json::to_value(&workflow)?).await?;
        Ok(serde_json::to_value(&workflow)?)
    });

    registry.register("Automation.GetWorkflows", |_params| async move {
        storage::init().await?;
        let items = storage::scan_namespace("automation_workflows").await?;
        Ok(serde_json::to_value(&workflows_from_storage_values(items))?)
    });

    registry.register("Automation.UpdateWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        let updates: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("updates").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing updates"))?;

        storage::init().await?;
        if let Some(mut workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await? {
            if let Some(workflow_obj) = workflow_value.as_object_mut() {
                for (key, value) in updates.as_object().unwrap() {
                    workflow_obj.insert(key.clone(), value.clone());
                }
                storage::set_kv("automation_workflows", &workflow_id, &workflow_value).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.DeleteWorkflow", |params| async move {
        let _workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        // Would need storage delete method
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.EnableWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        storage::init().await?;
        if let Some(mut workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await? {
            if let Some(workflow_obj) = workflow_value.as_object_mut() {
                workflow_obj.insert("enabled".to_string(), serde_json::json!(true));
                storage::set_kv("automation_workflows", &workflow_id, &workflow_value).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.DisableWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        storage::init().await?;
        if let Some(mut workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await? {
            if let Some(workflow_obj) = workflow_value.as_object_mut() {
                workflow_obj.insert("enabled".to_string(), serde_json::json!(false));
                storage::set_kv("automation_workflows", &workflow_id, &workflow_value).await?;
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.RunWorkflow", |params| async move {
        let workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        storage::init().await?;
        if let Some(workflow_value) = storage::get_kv("automation_workflows", &workflow_id).await? {
            if let Some(workflow) = workflow_value.as_object() {
                if let Some(actions) = workflow.get("actions") {
                    // Execute actions
                    execute_actions(actions).await?;
                }
            }
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Automation.GetWorkflowHistory", |params| async move {
        let _workflow_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("workflow_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing workflow_id"))?,
        )?;

        // Would store execution history
        Ok(serde_json::json!([]))
    });

    registry.register("Automation.CreateScript", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let content: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("content").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing content"))?,
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
        // Would need storage list method
        Ok(serde_json::json!([]))
    });

    registry.register("Automation.RunScript", |params| async move {
        let script_id: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("script_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing script_id"))?,
        )?;

        let args: Vec<String> = params
            .as_ref()
            .and_then(|p| p.get("args").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        storage::init().await?;
        if let Some(script_value) = storage::get_kv("automation_scripts", &script_id).await? {
            if let Some(script) = script_value.as_object() {
                if let (Some(content), Some(interpreter)) = (script.get("content"), script.get("interpreter")) {
                    // Write script to temp file and execute
                    let temp_file = format!("/tmp/script_{}.sh", script_id);
                    tokio::fs::write(&temp_file, content.as_str().unwrap()).await?;
                    // Set executable permissions
                    process::exec_command(&["chmod", "+x", &temp_file]).await.ok();

                    let mut cmd = vec![interpreter.as_str().unwrap(), &temp_file];
                    cmd.extend(args.iter().map(|s| s.as_str()));

                    let output = process::exec_command(&cmd).await?;
                    tokio::fs::remove_file(&temp_file).await.ok();

                    Ok(serde_json::json!({ "output": output }))
                } else {
                    anyhow::bail!("Invalid script format")
                }
            } else {
                anyhow::bail!("Script not found")
            }
        } else {
            anyhow::bail!("Script not found")
        }
    });

    registry.register("Automation.GetTriggers", |_params| async move {
        Ok(serde_json::json!([
            "time",
            "file_change",
            "system_event",
            "network_event",
            "command"
        ]))
    });

    registry.register("Automation.GetActions", |_params| async move {
        Ok(serde_json::json!([
            "execute_command",
            "send_notification",
            "change_settings",
            "run_script"
        ]))
    });
}

pub(crate) fn workflows_from_storage_values(items: Vec<serde_json::Value>) -> Vec<Workflow> {
    items
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn workflows_from_storage_empty_namespace() {
        assert!(workflows_from_storage_values(vec![]).is_empty());
    }
}

async fn execute_actions(actions: &serde_json::Value) -> Result<()> {
    if let Some(actions_array) = actions.as_array() {
        for action in actions_array {
            if let Some(action_obj) = action.as_object() {
                if let Some(action_type) = action_obj.get("type").and_then(|v| v.as_str()) {
                    match action_type {
                        "execute_command" => {
                            if let Some(command) = action_obj.get("command").and_then(|v| v.as_str()) {
                                process::exec_command(&["sh", "-c", command]).await.ok();
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
                            if let Some(_script_id) = action_obj.get("script_id").and_then(|v| v.as_str()) {
                                // Would call RunScript internally
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
