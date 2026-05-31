use crate::notify;
use crate::services::ServiceRegistry;
use crate::utils::{process, storage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioDevice {
    pub id: i32,
    pub name: String,
    pub info: String,
    pub volume: f64,
    pub is_default: bool,
    #[serde(default)]
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioStream {
    pub id: i32,
    pub name: String,
    pub app: String,
    pub volume: i32,
    pub sink_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProfile {
    pub name: String,
    pub timestamp: i64,
    pub routing: serde_json::Value,
    pub volumes: serde_json::Value,
    pub effects: serde_json::Value,
    pub loopbacks: Vec<serde_json::Value>,
}

static AUDIO_EMIT_GEN: AtomicU64 = AtomicU64::new(0);

lazy_static::lazy_static! {
    static ref PIPEWIRE_STATE: RwLock<PipeWireState> = RwLock::new(PipeWireState::default());
    static ref EASYEFFECTS_STATE: RwLock<EasyEffectsState> = RwLock::new(EasyEffectsState::default());
    static ref LAST_AUDIO_SNAPSHOT: RwLock<Option<serde_json::Value>> = RwLock::new(None);
}

#[derive(Debug, Clone, Default)]
struct PipeWireState {
    sinks: Vec<AudioDevice>,
    sources: Vec<AudioDevice>,
    streams: Vec<AudioStream>,
    nodes: Vec<serde_json::Value>,
    links: Vec<serde_json::Value>,
    pulse_id_map: HashMap<i32, i32>,
    ee_sink_id: i32,
    active_hardware_sink_id: i32,
}

#[derive(Debug, Clone, Default)]
struct EasyEffectsState {
    available: bool,
    running: bool,
    current_preset: String,
    available_presets: Vec<String>,
}

pub fn register(registry: &mut ServiceRegistry) {
    // Start PipeWire monitoring
    tokio::spawn(async {
        let mut interval = interval(Duration::from_millis(500));
        loop {
            interval.tick().await;
            refresh_devices().await.ok();
            refresh_streams().await.ok();
        }
    });

    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            refresh_nodes().await.ok();
        }
    });

    // Start EasyEffects monitoring
    tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            check_easyeffects_status().await.ok();
            refresh_easyeffects_presets().await.ok();
        }
    });

    // PipeWire Management Methods
    registry.register("Audio.GetDevices", |_params| async move {
        let state = PIPEWIRE_STATE.read().await;
        Ok(serde_json::json!({
            "sinks": state.sinks,
            "sources": state.sources
        }))
    });

    registry.register("Audio.SetDefaultDevice", |params| async move {
        let device_id: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("device_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing device_id"))?,
        )?;

        let _device_type: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("type").cloned())
                .unwrap_or(serde_json::Value::String("output".to_string())),
        )?;

        process::exec_command(&["wpctl", "set-default", &device_id.to_string()]).await?;
        refresh_devices().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.SetSinkVolume", |params| async move {
        let device_id: i32 = device_id_from_params(&params)?;
        let volume: f64 = volume_from_params(&params)?;
        process::exec_command(&[
            "wpctl",
            "set-volume",
            &device_id.to_string(),
            &format!("{:.2}", volume.clamp(0.0, 1.5)),
        ])
        .await?;
        refresh_devices().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.SetSourceVolume", |params| async move {
        let device_id: i32 = device_id_from_params(&params)?;
        let volume: f64 = volume_from_params(&params)?;
        process::exec_command(&[
            "wpctl",
            "set-volume",
            &device_id.to_string(),
            &format!("{:.2}", volume.clamp(0.0, 1.5)),
        ])
        .await?;
        refresh_devices().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.SetSinkMute", |params| async move {
        let device_id: i32 = device_id_from_params(&params)?;
        let muted: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("muted").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing muted"))?,
        )?;
        process::exec_command(&[
            "wpctl",
            "set-mute",
            &device_id.to_string(),
            if muted { "1" } else { "0" },
        ])
        .await?;
        refresh_devices().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.SetSourceMute", |params| async move {
        let device_id: i32 = device_id_from_params(&params)?;
        let muted: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("muted").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing muted"))?,
        )?;
        process::exec_command(&[
            "wpctl",
            "set-mute",
            &device_id.to_string(),
            if muted { "1" } else { "0" },
        ])
        .await?;
        refresh_devices().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.GetStreams", |_params| async move {
        let state = PIPEWIRE_STATE.read().await;
        Ok(serde_json::to_value(&state.streams)?)
    });

    registry.register("Audio.SetStreamVolume", |params| async move {
        let stream_id: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("stream_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing stream_id"))?,
        )?;

        let volume: f64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("volume").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing volume"))?,
        )?;

        let volume_percent = (volume * 100.0).round() as i32;
        process::exec_command(&[
            "pactl",
            "set-sink-input-volume",
            &stream_id.to_string(),
            &format!("{}%", volume_percent),
        ])
        .await?;

        refresh_streams().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.SetStreamMute", |params| async move {
        let stream_id: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("stream_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing stream_id"))?,
        )?;

        let muted: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("muted").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing muted"))?,
        )?;

        process::exec_command(&[
            "pactl",
            "set-sink-input-mute",
            &stream_id.to_string(),
            if muted { "1" } else { "0" },
        ])
        .await?;

        refresh_streams().await?;
        schedule_audio_state_emit().await;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.RouteStream", |params| async move {
        let stream_id: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("stream_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing stream_id"))?,
        )?;

        let sink_id: i32 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("sink_id").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing sink_id"))?,
        )?;

        // Find sink node name from state
        let state = PIPEWIRE_STATE.read().await;
        let sink = state.sinks.iter().find(|s| s.id == sink_id);
        let sink_name = sink.map(|s| s.name.clone());

        drop(state);

        if let Some(name) = sink_name {
            // Use pactl to move stream
            process::exec_command(&[
                "pactl",
                "move-sink-input",
                &stream_id.to_string(),
                &name,
            ])
            .await?;
        }

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.CreateLoopback", |params| async move {
        let source_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("source_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing source_name"))?,
        )?;

        let sink_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("sink_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing sink_name"))?,
        )?;

        process::exec_command_detached(&[
            "pw-loopback",
            "-C",
            &source_name,
            "-P",
            &sink_name,
            "--capture-props=media.class=Audio/Source",
            "--playback-props=media.class=Audio/Sink",
        ])
        .await?;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.CreateNullSink", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let description: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("description").cloned())
                .unwrap_or(serde_json::Value::String(name.clone())),
        )?;

        process::exec_command(&[
            "pactl",
            "load-module",
            "module-null-sink",
            &format!("sink_name={}", name),
            &format!("sink_properties=device.description={}", description),
        ])
        .await?;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Refresh", |params| async move {
        let restart_wireplumber: bool = params
            .as_ref()
            .and_then(|p| p.get("restart_wireplumber").cloned())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or(false);

        if restart_wireplumber {
            let _ = process::exec_command(&[
                "systemctl",
                "--user",
                "restart",
                "wireplumber",
            ])
            .await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        refresh_devices().await?;
        refresh_streams().await?;
        refresh_nodes().await?;
        emit_audio_state_now().await;
        Ok(serde_json::json!({ "success": true }))
    });

    // EasyEffects Methods
    registry.register("Audio.Effects.GetStatus", |_params| async move {
        let state = EASYEFFECTS_STATE.read().await;
        Ok(serde_json::json!({
            "available": state.available,
            "running": state.running,
            "current_preset": state.current_preset
        }))
    });

    registry.register("Audio.Effects.Start", |_params| async move {
        process::exec_command_detached(&["easyeffects", "--gapplication-service"]).await?;
        let mut state = EASYEFFECTS_STATE.write().await;
        state.running = true;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.Stop", |_params| async move {
        // Use D-Bus to quit EasyEffects
        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects",
            "com.github.wwmm.easyeffects.Quit",
        ])
        .await;

        let mut state = EASYEFFECTS_STATE.write().await;
        state.running = false;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.GetPresets", |_params| async move {
        let state = EASYEFFECTS_STATE.read().await;
        Ok(serde_json::to_value(&state.available_presets)?)
    });

    registry.register("Audio.Effects.LoadPreset", |params| async move {
        let preset_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("preset_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing preset_name"))?,
        )?;

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects",
            "com.github.wwmm.easyeffects.LoadPreset",
            &format!("string:\"{}\"", preset_name),
        ])
        .await;

        let mut state = EASYEFFECTS_STATE.write().await;
        state.current_preset = preset_name.clone();
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.SavePreset", |params| async move {
        let preset_name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("preset_name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing preset_name"))?,
        )?;

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects",
            "com.github.wwmm.easyeffects.SavePreset",
            &format!("string:\"{}\"", preset_name),
        ])
        .await;

        refresh_easyeffects_presets().await.ok();
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.SetEqBand", |params| async move {
        let band_index: usize = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("band_index").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing band_index"))?,
        )?;

        let gain_db: f64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("gain_db").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing gain_db"))?,
        )?;

        let band_path = format!(
            "/com/github/wwmm/easyeffects/streamoutputs/equalizer/band{}",
            band_index
        );

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            &band_path,
            "org.freedesktop.DBus.Properties.Set",
            "string:\"com.github.wwmm.easyeffects.equalizer.band\"",
            "string:\"gain\"",
            &format!("variant:double:{}", gain_db),
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.SetNoiseSuppression", |params| async move {
        let enabled: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("enabled").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing enabled"))?,
        )?;

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects/streaminputs/rnnoise",
            "org.freedesktop.DBus.Properties.Set",
            "string:\"com.github.wwmm.easyeffects.rnnoise\"",
            "string:\"bypass\"",
            &format!("variant:boolean:{}", !enabled),
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.SetMicGain", |params| async move {
        let gain_db: f64 = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("gain_db").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing gain_db"))?,
        )?;

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects/streaminputs/compressor",
            "org.freedesktop.DBus.Properties.Set",
            "string:\"com.github.wwmm.easyeffects.compressor\"",
            "string:\"makeup\"",
            &format!("variant:double:{}", gain_db),
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.SetNoiseGate", |params| async move {
        let enabled: bool = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("enabled").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing enabled"))?,
        )?;

        let threshold_db: Option<f64> = params
            .as_ref()
            .and_then(|p| p.get("threshold_db").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        if let Some(threshold) = threshold_db {
            let _ = process::exec_command(&[
                "dbus-send",
                "--session",
                "--print-reply",
                "--dest=com.github.wwmm.easyeffects",
                "/com/github/wwmm/easyeffects/streaminputs/gate",
                "org.freedesktop.DBus.Properties.Set",
                "string:\"com.github.wwmm.easyeffects.gate\"",
                "string:\"threshold\"",
                &format!("variant:double:{}", threshold),
            ])
            .await;
        }

        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects/streaminputs/gate",
            "org.freedesktop.DBus.Properties.Set",
            "string:\"com.github.wwmm.easyeffects.gate\"",
            "string:\"bypass\"",
            &format!("variant:boolean:{}", !enabled),
        ])
        .await;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Effects.Reset", |_params| async move {
        let _ = process::exec_command(&[
            "dbus-send",
            "--session",
            "--print-reply",
            "--dest=com.github.wwmm.easyeffects",
            "/com/github/wwmm/easyeffects",
            "com.github.wwmm.easyeffects.Reset",
        ])
        .await;

        let mut state = EASYEFFECTS_STATE.write().await;
        state.current_preset = String::new();
        Ok(serde_json::json!({ "success": true }))
    });

    // Audio Profiles Methods
    registry.register("Audio.Profiles.Save", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        let config: serde_json::Value = params
            .as_ref()
            .and_then(|p| p.get("config").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing config"))?;

        let profile = AudioProfile {
            name: name.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            routing: config.get("routing").cloned().unwrap_or_default(),
            volumes: config.get("volumes").cloned().unwrap_or_default(),
            effects: config.get("effects").cloned().unwrap_or_default(),
            loopbacks: config
                .get("loopbacks")
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default(),
        };

        // Save to SQLite storage
        storage::init().await?;
        storage::set_kv("audio_profiles", &name, &serde_json::to_value(&profile)?).await?;

        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Profiles.Load", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        storage::init().await?;
        let profile_value = storage::get_kv("audio_profiles", &name).await?;

        if let Some(profile_json) = profile_value {
            let profile: AudioProfile = serde_json::from_value(profile_json)?;
            // Apply profile (routing, volumes, effects)
            apply_profile(&profile).await?;
            Ok(serde_json::to_value(&profile)?)
        } else {
            anyhow::bail!("Profile not found: {}", name)
        }
    });

    registry.register("Audio.Profiles.List", |_params| async move {
        storage::init().await?;
        let keys = storage::list_keys("audio_profiles").await?;
        let mut profiles = Vec::new();
        for name in keys {
            if let Some(profile_json) = storage::get_kv("audio_profiles", &name).await? {
                if let Ok(profile) = serde_json::from_value::<AudioProfile>(profile_json) {
                    profiles.push(profile);
                }
            }
        }
        Ok(serde_json::to_value(profiles)?)
    });

    registry.register("Audio.Profiles.Delete", |params| async move {
        let name: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("name").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?,
        )?;

        storage::init().await?;
        let deleted = storage::delete_kv("audio_profiles", &name).await?;
        Ok(serde_json::json!({ "success": deleted }))
    });

    registry.register("Audio.Profiles.GetCurrent", |_params| async move {
        let state = PIPEWIRE_STATE.read().await;
        let effects_state = EASYEFFECTS_STATE.read().await;

        let config = serde_json::json!({
            "routing": {
                "default_sink": state.sinks.iter().find(|s| s.is_default).map(|s| s.id),
                "default_source": state.sources.iter().find(|s| s.is_default).map(|s| s.id),
            },
            "volumes": {
                "devices": state.sinks.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "volume": s.volume,
                    "muted": false
                })).collect::<Vec<_>>(),
                "streams": state.streams.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "volume": s.volume
                })).collect::<Vec<_>>()
            },
            "effects": {
                "preset": effects_state.current_preset,
                "noise_suppression": false, // TODO: Get actual state
            }
        });

        Ok(config)
    });

    registry.register("Audio.Profiles.ApplyScenario", |params| async move {
        let scenario: String = serde_json::from_value(
            params
                .as_ref()
                .and_then(|p| p.get("scenario").cloned())
                .ok_or_else(|| anyhow::anyhow!("Missing scenario"))?,
        )?;

        match scenario.as_str() {
            "streaming" => {
                // Set noise suppression, mic gain, load streaming preset
                let _ = process::exec_command(&[
                    "dbus-send",
                    "--session",
                    "--dest=com.github.wwmm.easyeffects",
                    "/com/github/wwmm/easyeffects/streaminputs/rnnoise",
                    "org.freedesktop.DBus.Properties.Set",
                    "string:\"com.github.wwmm.easyeffects.rnnoise\"",
                    "string:\"bypass\"",
                    "variant:boolean:false",
                ])
                .await;
            }
            "gaming" => {
                // Similar to streaming
            }
            "music-production" => {
                // Load flat preset, disable noise suppression
            }
            _ => {}
        }

        Ok(serde_json::json!({ "success": true }))
    });

    // MPRIS Media Control Methods
    registry.register("Audio.Media.GetPlayers", |_params| async move {
        let output = process::exec_command(&["playerctl", "-l"]).await?;
        let players: Vec<String> = output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(serde_json::to_value(&players)?)
    });

    registry.register("Audio.Media.PlayPause", |params| async move {
        let player_name: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("player_name").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["playerctl", "play-pause"];
        if let Some(ref name) = player_name {
            cmd.push("-p");
            cmd.push(name);
        }

        process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Media.Next", |params| async move {
        let player_name: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("player_name").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["playerctl", "next"];
        if let Some(ref name) = player_name {
            cmd.push("-p");
            cmd.push(name);
        }

        process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "success": true }))
    });

    registry.register("Audio.Media.Previous", |params| async move {
        let player_name: Option<String> = params
            .as_ref()
            .and_then(|p| p.get("player_name").cloned())
            .and_then(|v| serde_json::from_value(v).ok());

        let mut cmd = vec!["playerctl", "previous"];
        if let Some(ref name) = player_name {
            cmd.push("-p");
            cmd.push(name);
        }

        process::exec_command(&cmd).await?;
        Ok(serde_json::json!({ "success": true }))
    });
}

pub(crate) async fn snapshot_audio_tile() -> serde_json::Value {
    let state = PIPEWIRE_STATE.read().await;
    let default_sink = state
        .sinks
        .iter()
        .find(|s| s.is_default)
        .or(state.sinks.first());
    serde_json::json!({
        "sink_count": state.sinks.len(),
        "source_count": state.sources.len(),
        "default_sink": default_sink.map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "volume": s.volume,
            "muted": s.muted,
        })),
    })
}

fn device_id_from_params(params: &Option<serde_json::Value>) -> Result<i32> {
    Ok(serde_json::from_value(
        params
            .as_ref()
            .and_then(|p| p.get("device_id").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing device_id"))?,
    )?)
}

fn volume_from_params(params: &Option<serde_json::Value>) -> Result<f64> {
    Ok(serde_json::from_value(
        params
            .as_ref()
            .and_then(|p| p.get("volume").cloned())
            .ok_or_else(|| anyhow::anyhow!("Missing volume"))?,
    )?)
}

async fn schedule_audio_state_emit() {
    let gen = AUDIO_EMIT_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        if AUDIO_EMIT_GEN.load(Ordering::SeqCst) != gen {
            return;
        }
        let _ = emit_audio_state_now().await;
    });
}

async fn emit_audio_state_now() -> Result<()> {
    let state = PIPEWIRE_STATE.read().await;
    let snapshot = serde_json::json!({
        "sinks": state.sinks,
        "sources": state.sources,
        "streams": state.streams,
    });
    drop(state);

    let mut last = LAST_AUDIO_SNAPSHOT.write().await;
    if last.as_ref() == Some(&snapshot) {
        return Ok(());
    }
    *last = Some(snapshot.clone());
    notify::emit("Audio.StateChanged", snapshot);
    Ok(())
}

async fn refresh_devices() -> Result<()> {
    let output = process::exec_command(&["wpctl", "status"]).await?;
    let (sinks, sources) = parse_devices(&output);

    let mut state = PIPEWIRE_STATE.write().await;
    let changed = state.sinks != sinks || state.sources != sources;
    state.sinks = sinks;
    state.sources = sources;
    drop(state);
    if changed {
        schedule_audio_state_emit().await;
    }
    Ok(())
}

pub fn parse_devices(output: &str) -> (Vec<AudioDevice>, Vec<AudioDevice>) {
    let mut sinks = Vec::new();
    let mut sources = Vec::new();
    let mut in_audio_section = false;
    let mut in_sinks = false;
    let mut in_sources = false;

    for line in output.lines() {
        if line.starts_with("Audio") {
            in_audio_section = true;
            continue;
        }
        if line.starts_with("Video") || line.starts_with("Settings") {
            in_audio_section = false;
            in_sinks = false;
            in_sources = false;
            continue;
        }

        if in_audio_section {
            if line.contains("Sinks:") && !line.contains("Sink Inputs") {
                in_sinks = true;
                in_sources = false;
                continue;
            }
            if line.contains("Sources:") {
                in_sources = true;
                in_sinks = false;
                continue;
            }
            if line.contains("Devices:")
                || line.contains("Streams:")
                || line.contains("Sink Inputs")
            {
                in_sinks = false;
                in_sources = false;
                continue;
            }

            if (in_sinks || in_sources) && line.contains("│") || line.contains("├") || line.contains("└") {
                if let Some(device) = parse_device_line(line) {
                    if in_sinks {
                        sinks.push(device);
                    } else if in_sources {
                        sources.push(device);
                    }
                }
            }
        }
    }

    (sinks, sources)
}

pub fn parse_device_line(line: &str) -> Option<AudioDevice> {
    // Parse lines like: │   ●   47. Headphones [vol: 0.50]
    let re = regex::Regex::new(r"^\s*[│├└]\s*([●\*\s])\s*(\d+)\.\s*(.+)$").ok()?;
    let cap = re.captures(line)?;

    let marker = cap.get(1)?.as_str().trim();
    let id: i32 = cap.get(2)?.as_str().parse().ok()?;
    let rest = cap.get(3)?.as_str();

    let bracket_re = regex::Regex::new(r"^(.+?)\s*\[([^\]]+)\]\s*$").ok()?;
    let (name, info) = if let Some(bracket_cap) = bracket_re.captures(rest) {
        (
            bracket_cap.get(1)?.as_str().trim().to_string(),
            bracket_cap.get(2)?.as_str().trim().to_string(),
        )
    } else {
        (rest.trim().to_string(), String::new())
    };

    let mut volume = 0.0;
    let mut muted = false;
    if info.contains("vol:") {
        if let Some(vol_cap) = regex::Regex::new(r"vol:\s*(\d+\.\d+)")
            .ok()?
            .captures(&info)
        {
            volume = vol_cap.get(1)?.as_str().parse().ok().unwrap_or(0.0);
        }
    }
    if info.to_uppercase().contains("MUTED") {
        muted = true;
    }

    Some(AudioDevice {
        id,
        name,
        info,
        volume,
        is_default: marker == "●" || marker == "*",
        muted,
    })
}

async fn refresh_streams() -> Result<()> {
    let output = process::exec_command(&["pactl", "list", "sink-inputs"]).await?;
    let streams = parse_streams(&output);

    let mut state = PIPEWIRE_STATE.write().await;
    let changed = state.streams != streams;
    state.streams = streams;
    drop(state);
    if changed {
        schedule_audio_state_emit().await;
    }
    Ok(())
}

pub fn parse_streams(output: &str) -> Vec<AudioStream> {
    let mut streams = Vec::new();
    let mut current_stream: Option<AudioStream> = None;

    for line in output.lines() {
        if line.trim().starts_with("Sink Input #") {
            if let Some(stream) = current_stream.take() {
                streams.push(stream);
            }

            if let Some(id_cap) = regex::Regex::new(r"Sink Input #(\d+)")
                .ok()
                .and_then(|re| re.captures(line))
            {
                if let Some(id_match) = id_cap.get(1) {
                    if let Ok(id) = id_match.as_str().parse() {
                        current_stream = Some(AudioStream {
                            id,
                            name: String::new(),
                            app: String::new(),
                            volume: 100,
                            sink_id: -1,
                        });
                    }
                }
            }
        } else if let Some(ref mut stream) = current_stream {
            if line.trim().starts_with("Sink:") {
                if let Some(sink_cap) = regex::Regex::new(r"Sink:\s*(\d+)")
                    .ok()
                    .and_then(|re| re.captures(line))
                {
                    if let Some(sink_match) = sink_cap.get(1) {
                        if let Ok(sink_id) = sink_match.as_str().parse() {
                            stream.sink_id = sink_id;
                        }
                    }
                }
            } else if line.contains("application.name =") {
                if let Some(app) = line.split('=').nth(1) {
                    stream.app = app.trim().trim_matches('"').to_string();
                }
            } else if line.contains("media.name =") {
                if let Some(name) = line.split('=').nth(1) {
                    stream.name = name.trim().trim_matches('"').to_string();
                }
            } else if line.contains("Volume:") {
                if let Some(vol_cap) = regex::Regex::new(r"(\d+)%")
                    .ok()
                    .and_then(|re| re.captures(line))
                {
                    if let Some(vol_match) = vol_cap.get(1) {
                        if let Ok(vol) = vol_match.as_str().parse() {
                            stream.volume = vol;
                        }
                    }
                }
            }
        }
    }

    if let Some(stream) = current_stream {
        streams.push(stream);
    }

    streams
}

async fn refresh_nodes() -> Result<()> {
    let output = process::exec_command(&["pw-dump"]).await?;
    let nodes: Vec<serde_json::Value> = serde_json::from_str(&output)?;

    let pipewire_nodes: Vec<serde_json::Value> = nodes
        .into_iter()
        .filter(|n| {
            n.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "PipeWire:Interface:Node")
                .unwrap_or(false)
        })
        .collect();

    let mut pulse_map = HashMap::new();
    for node in &pipewire_nodes {
        if let Some(id) = node.get("id").and_then(|i| i.as_i64()) {
            let serial = node
                .get("info")
                .and_then(|i| i.get("props"))
                .and_then(|p| p.get("object.serial"))
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .or_else(|| {
                    node.get("props")
                        .and_then(|p| p.get("object.serial"))
                        .and_then(|s| s.as_str())
                        .and_then(|s| s.parse::<i32>().ok())
                });

            if let Some(serial) = serial {
                pulse_map.insert(serial, id as i32);
            }
        }
    }

    let mut state = PIPEWIRE_STATE.write().await;
    state.nodes = pipewire_nodes;
    state.pulse_id_map = pulse_map;

    // Find Easy Effects sink
    state.ee_sink_id = state
        .sinks
        .iter()
        .find(|s| s.name.contains("Easy Effects") || s.name.contains("easyeffects"))
        .map(|s| s.id)
        .unwrap_or(-1);

    Ok(())
}

async fn check_easyeffects_status() -> Result<()> {
    let output = process::exec_command(&[
        "dbus-send",
        "--session",
        "--print-reply",
        "--dest=org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus.ListNames",
    ])
    .await?;

    let available = output.contains("com.github.wwmm.easyeffects");
    let mut state = EASYEFFECTS_STATE.write().await;
    state.available = available;
    state.running = available;
    Ok(())
}

async fn refresh_easyeffects_presets() -> Result<()> {
    let home = std::env::var("HOME")?;
    let preset_dir = format!("{}/.config/easyeffects/output", home);

    if let Ok(entries) = tokio::fs::read_dir(&preset_dir).await {
        let mut presets = Vec::new();
        let mut entries = entries;

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.ends_with(".json") {
                    if let Some(name) = file_name.strip_suffix(".json") {
                        presets.push(name.to_string());
                    }
                }
            }
        }

        let mut state = EASYEFFECTS_STATE.write().await;
        state.available_presets = presets;
    }

    Ok(())
}

async fn apply_profile(profile: &AudioProfile) -> Result<()> {
    // Apply routing
    if let Some(default_sink) = profile.routing.get("defaultSink").and_then(|v| v.as_i64()) {
        process::exec_command(&["wpctl", "set-default", &default_sink.to_string()]).await.ok();
    }

    // Apply volumes
    if let Some(devices) = profile.volumes.get("devices").and_then(|v| v.as_array()) {
        for device in devices {
            if let (Some(id), Some(vol)) = (
                device.get("id").and_then(|v| v.as_i64()),
                device.get("volume").and_then(|v| v.as_f64()),
            ) {
                process::exec_command(&[
                    "wpctl",
                    "set-volume",
                    "-l",
                    "2.0",
                    &id.to_string(),
                    &vol.to_string(),
                ])
                .await
                .ok();
            }
        }
    }

    // Apply effects
    if let Some(preset) = profile.effects.get("preset").and_then(|v| v.as_str()) {
        if !preset.is_empty() {
            let _ = process::exec_command(&[
                "dbus-send",
                "--session",
                "--dest=com.github.wwmm.easyeffects",
                "/com/github/wwmm/easyeffects",
                "com.github.wwmm.easyeffects.LoadPreset",
                &format!("string:\"{}\"", preset),
            ])
            .await;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wpctl_status_fixture() {
        let fixture = include_str!("../../tests/fixtures/audio/wpctl_status.txt");
        let (sinks, sources) = parse_devices(fixture);
        assert_eq!(sinks.len(), 1);
        assert!(sinks[0].muted);
        assert!(sinks[0].is_default);
        assert_eq!(sources.len(), 1);
    }

    #[test]
    fn parse_pactl_streams_fixture() {
        let fixture = include_str!("../../tests/fixtures/audio/pactl_sink_inputs.txt");
        let streams = parse_streams(fixture);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].app, "Firefox");
        assert_eq!(streams[0].volume, 100);
        assert_eq!(streams[0].sink_id, 47);
    }

    #[test]
    fn parse_wpctl_multiple_devices_fixture() {
        let fixture = include_str!("../../tests/fixtures/audio/wpctl_status_multi.txt");
        let (sinks, sources) = parse_devices(fixture);
        assert_eq!(sinks.len(), 2);
        assert!(!sinks[0].is_default);
        assert!(sinks[1].is_default);
        assert!((sinks[1].volume - 0.75).abs() < f64::EPSILON);
        assert_eq!(sources.len(), 1);
        assert!(sources[0].is_default);
    }

    #[test]
    fn parse_device_line_extracts_volume_and_mute() {
        let line = "│  *   99. Speakers [vol: 0.25 MUTED]";
        let dev = parse_device_line(line).expect("device");
        assert_eq!(dev.id, 99);
        assert!(dev.is_default);
        assert!(dev.muted);
        assert!((dev.volume - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_device_line_rejects_malformed() {
        assert!(parse_device_line("not a wpctl device row").is_none());
        assert!(parse_device_line("│   no-id. Broken").is_none());
    }

    #[test]
    fn parse_streams_empty_fixture() {
        let fixture = include_str!("../../tests/fixtures/audio/pactl_sink_inputs_empty.txt");
        assert!(parse_streams(fixture).is_empty());
    }

    #[test]
    fn parse_devices_skips_line_without_brackets() {
        let fixture = include_str!("../../tests/fixtures/audio/wpctl_status_plain_name.txt");
        let (sinks, sources) = parse_devices(fixture);
        assert_eq!(sinks.len(), 1);
        assert_eq!(sinks[0].name, "HDMI Output");
        assert_eq!(sinks[0].info, "");
        assert!(sources.is_empty());
    }

    #[test]
    fn parse_streams_pactl_error_and_malformed_fixtures() {
        let err = include_str!("../../tests/fixtures/audio/pactl_error.txt");
        assert!(parse_streams(err).is_empty());

        let malformed = include_str!("../../tests/fixtures/audio/pactl_sink_inputs_malformed.txt");
        let streams = parse_streams(malformed);
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].id, 12);
        assert_eq!(streams[0].app, "");
        assert_eq!(streams[0].sink_id, -1);
    }
}
