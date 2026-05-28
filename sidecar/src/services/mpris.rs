//! Best-effort MPRIS via `playerctl` for the WebKit bar (replaces GTK AstalMpris).
use crate::services::ServiceRegistry;
use crate::utils::process;
use serde_json::json;

pub fn register(registry: &mut ServiceRegistry) {
    registry.register("Media.GetNowPlaying", |_p| async move {
        match process::exec_command(&["playerctl", "metadata", "--format", "{{title}}\t{{artist}}"])
            .await
        {
            Ok(s) => {
                let (title, artist) = parse_now_playing_line(&s);
                Ok(json!({
                    "playing": !title.is_empty(),
                    "title": title,
                    "artist": artist,
                }))
            }
            Err(_) => Ok(json!({
                "playing": false,
                "title": "",
                "artist": "",
            })),
        }
    });
}

pub fn parse_now_playing_line(s: &str) -> (String, String) {
    let mut parts = s.trim().splitn(2, '\t');
    let title = parts.next().unwrap_or("").to_string();
    let artist = parts.next().unwrap_or("").to_string();
    (title, artist)
}

#[cfg(test)]
mod tests {
    use super::parse_now_playing_line;

    #[test]
    fn parse_playerctl_metadata() {
        let (title, artist) = parse_now_playing_line("Song\tArtist");
        assert_eq!(title, "Song");
        assert_eq!(artist, "Artist");
        let (empty, _) = parse_now_playing_line("\n");
        assert!(empty.is_empty());
    }
}
