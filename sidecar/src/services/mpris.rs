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
                let mut parts = s.splitn(2, '\t');
                let title = parts.next().unwrap_or("").to_string();
                let artist = parts.next().unwrap_or("").to_string();
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
