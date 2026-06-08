use crate::services::ServiceRegistry;
use crate::utils::process;

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Security.Offensive.PortScan.TcpSyn", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("ports").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sS"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.Udp", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("ports").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sU"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.ServiceVersion", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let ports: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("ports").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["nmap", "-sV"];
        if let Some(ref p) = ports {
            cmd.extend_from_slice(&["-p", p]);
        }
        cmd.push(&target);

        let output = process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "output": output }))
    });

    registry.register("Security.Offensive.PortScan.OsDetection", |params| async move {
        let target: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("target").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing target"))?,
        )?;

        let output = process::exec_command(&["nmap", "-O", &target]).await?;
        Ok(serde_json::json!({ "output": output }))
    });
}
