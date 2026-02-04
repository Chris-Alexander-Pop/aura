use hyprland::data::Workspace;
use hyprland::shared::HyprDataActive;
use serde::Serialize;
use std::fs;
use std::time::Duration;
use tokio::time;

#[derive(Serialize)]
struct SystemState {
    time: String,
    battery: u8,
    is_charging: bool,
    workspace: i32,
}

fn get_battery() -> (u8, bool) {
    // Some systems use BAT0, some use BAT1. Checking BAT0 first.
    let capacity_path = "/sys/class/power_supply/BAT0/capacity";
    let status_path = "/sys/class/power_supply/BAT0/status";

    let capacity = fs::read_to_string(capacity_path)
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/capacity"))
        .unwrap_or_else(|_| "0".to_string())
        .trim()
        .parse::<u8>()
        .unwrap_or(0);

    let status = fs::read_to_string(status_path)
        .or_else(|_| fs::read_to_string("/sys/class/power_supply/BAT1/status"))
        .unwrap_or_else(|_| "Unknown".to_string());

    let is_charging = status.trim() == "Charging";

    (capacity, is_charging)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut interval = time::interval(Duration::from_millis(500));

    loop {
        interval.tick().await;

        let (battery, is_charging) = get_battery();
        
        // Get active workspace from Hyprland
        let workspace = Workspace::get_active()
            .map(|ws| ws.id)
            .unwrap_or(1);

        let state = SystemState {
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
            battery,
            is_charging,
            workspace,
        };

        if let Ok(json) = serde_json::to_string(&state) {
            println!("{}", json);
        }
    }
}