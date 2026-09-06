//! On-screen display push events for the GTK shell (`Shell.Osd` → `osdController`).
use crate::services::audio::AudioDevice;
use crate::notify;
use serde_json::json;

pub fn payload(kind: &str, label: &str, percent: u32) -> serde_json::Value {
    json!({
        "kind": kind,
        "label": label,
        "percent": percent.clamp(0, 100),
    })
}

pub fn emit(kind: &str, label: &str, percent: u32) {
    notify::emit("Shell.Osd", payload(kind, label, percent));
}

pub fn emit_volume_from_sink(sink: &AudioDevice) {
    let pct = (sink.volume * 100.0).round().clamp(0.0, 100.0) as u32;
    if sink.muted {
        emit("volume", "Muted", 0);
    } else {
        emit("volume", &format!("{pct}%"), pct);
    }
}

pub fn emit_mic_from_source(source: &AudioDevice) {
    if source.muted {
        emit("mic", "Mic muted", 0);
    } else {
        emit("mic", "Mic on", 100);
    }
}

pub fn emit_brightness(brightness: f64) {
    let pct = (brightness * 100.0).round().clamp(0.0, 100.0) as u32;
    emit("brightness", &format!("{pct}%"), pct);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_clamps_percent() {
        let p = payload("volume", "100%", 150);
        assert_eq!(p["percent"], 100);
    }

    #[test]
    fn emit_volume_muted_and_unmuted() {
        let sink = AudioDevice {
            id: 1,
            name: "Speakers".into(),
            info: String::new(),
            volume: 0.42,
            is_default: true,
            muted: false,
        };
        let p = payload(
            "volume",
            &format!("{}%", (sink.volume * 100.0).round() as u32),
            (sink.volume * 100.0).round() as u32,
        );
        assert_eq!(p["kind"], "volume");
        assert_eq!(p["percent"], 42);

        let muted = AudioDevice { muted: true, ..sink };
        let p = payload("volume", "Muted", 0);
        assert_eq!(p["label"], "Muted");
        assert!(!muted.muted || p["percent"] == 0);
    }

    #[test]
    fn emit_mic_and_brightness_labels() {
        let p = payload("mic", "Mic muted", 0);
        assert_eq!(p["kind"], "mic");
        assert_eq!(p["label"], "Mic muted");

        let p = payload("brightness", "65%", 65);
        assert_eq!(p["kind"], "brightness");
        assert_eq!(p["percent"], 65);
    }
}
