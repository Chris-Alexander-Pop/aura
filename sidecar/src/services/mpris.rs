//! Best-effort MPRIS via `playerctl` for the WebKit bar (replaces GTK AstalMpris).
use crate::services::ServiceRegistry;
use crate::utils::process;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerctlPlayback {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

pub fn parse_playerctl_status(s: &str) -> PlayerctlPlayback {
    match s.trim() {
        "Playing" => PlayerctlPlayback::Playing,
        "Paused" => PlayerctlPlayback::Paused,
        "Stopped" => PlayerctlPlayback::Stopped,
        _ => PlayerctlPlayback::Unknown,
    }
}

pub fn playing_from_status(status: PlayerctlPlayback) -> bool {
    status == PlayerctlPlayback::Playing
}

pub fn parse_now_playing_line(s: &str) -> (String, String) {
    let mut parts = s.trim().splitn(2, '\t');
    let title = parts.next().unwrap_or("").to_string();
    let artist = parts.next().unwrap_or("").to_string();
    (title, artist)
}

/// Non-empty lines from `playerctl -l` (active MPRIS player names).
pub fn parse_playerctl_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

async fn playerctl_status() -> PlayerctlPlayback {
    match process::exec_command(&["playerctl", "status"]).await {
        Ok(s) => parse_playerctl_status(&s),
        Err(_) => PlayerctlPlayback::Unknown,
    }
}

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Media.GetNowPlaying", |_p| async move {
        let status = playerctl_status().await;
        let playing = playing_from_status(status);
        match process::exec_command(&["playerctl", "metadata", "--format", "{{title}}\t{{artist}}"])
            .await
        {
            Ok(s) => {
                let (title, artist) = parse_now_playing_line(&s);
                let players = process::exec_command(&["playerctl", "-l"])
                    .await
                    .unwrap_or_default();
                let player_name = parse_playerctl_list(&players).into_iter().next();
                Ok(json!({
                    "playing": playing,
                    "paused": status == PlayerctlPlayback::Paused,
                    "stopped": status == PlayerctlPlayback::Stopped,
                    "title": title,
                    "artist": artist,
                    "player_name": player_name,
                }))
            }
            Err(_) => Ok(json!({
                "playing": false,
                "paused": false,
                "stopped": status == PlayerctlPlayback::Stopped,
                "title": "",
                "artist": "",
                "player_name": null,
            })),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn parse_playerctl_metadata() {
        let (title, artist) = parse_now_playing_line("Song\tArtist");
        assert_eq!(title, "Song");
        assert_eq!(artist, "Artist");
        let (empty, _) = parse_now_playing_line("\n");
        assert!(empty.is_empty());
    }

    #[test]
    fn parse_playerctl_status_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio/playerctl_status.txt");
        let text = fs::read_to_string(path).unwrap();
        assert_eq!(parse_playerctl_status(&text), PlayerctlPlayback::Playing);
        assert!(playing_from_status(PlayerctlPlayback::Playing));
        assert!(!playing_from_status(PlayerctlPlayback::Paused));
    }

    #[test]
    fn parse_playerctl_status_variants() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/audio");
        let paused = fs::read_to_string(dir.join("playerctl_status_paused.txt")).unwrap();
        assert_eq!(parse_playerctl_status(&paused), PlayerctlPlayback::Paused);
        let stopped = fs::read_to_string(dir.join("playerctl_status_stopped.txt")).unwrap();
        assert_eq!(parse_playerctl_status(&stopped), PlayerctlPlayback::Stopped);
        assert_eq!(parse_playerctl_status("unknown"), PlayerctlPlayback::Unknown);
    }

    #[test]
    fn parse_playerctl_metadata_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio/playerctl_metadata.txt");
        let text = fs::read_to_string(path).unwrap();
        let (title, artist) = parse_now_playing_line(&text);
        assert_eq!(title, "Creep");
        assert_eq!(artist, "Radiohead");
    }

    #[test]
    fn parse_playerctl_list_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/audio/playerctl_list.txt");
        let text = fs::read_to_string(path).unwrap();
        let players = parse_playerctl_list(&text);
        assert_eq!(players, vec!["spotify", "firefox"]);
    }
}
