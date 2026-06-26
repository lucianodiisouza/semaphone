use std::path::PathBuf;

use sem_core::config::Config;
use sem_core::ipc::send_set;
use sem_core::state::{LightState, AWAITING_INPUT_REASON};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorComposerMode {
    Ask,
    Agent,
}

pub fn parse_conversation_id(input: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(input).ok()?;
    ["conversation_id", "session_id", "sessionId"]
        .iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_str()))
        .map(str::to_string)
}

pub fn parse_composer_mode(input: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(input).ok()?;
    ["composer_mode", "mode", "current_mode"]
        .iter()
        .find_map(|key| value.get(*key).and_then(|v| v.as_str()))
        .map(|mode| mode.to_ascii_lowercase())
}

pub fn classify_composer_mode(mode: &str) -> CursorComposerMode {
    match mode {
        "chat" | "ask" => CursorComposerMode::Ask,
        _ => CursorComposerMode::Agent,
    }
}

pub fn stop_waits_for_user_input(cached_mode: Option<&str>) -> bool {
    match cached_mode.map(classify_composer_mode) {
        Some(CursorComposerMode::Ask) => false,
        Some(CursorComposerMode::Agent) => true,
        None => true,
    }
}

pub fn stop_light_reason(cached_mode: Option<&str>) -> &'static str {
    if stop_waits_for_user_input(cached_mode) {
        AWAITING_INPUT_REASON
    } else {
        "idle"
    }
}

fn sessions_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SEMAPHORE_SESSIONS_DIR") {
        return PathBuf::from(dir);
    }
    Config::config_dir().join("sessions")
}

fn mode_cache_path(conversation_id: &str) -> PathBuf {
    let safe: String = conversation_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    sessions_dir().join(format!("{safe}.mode"))
}

pub fn cache_composer_mode(conversation_id: &str, mode: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(sessions_dir())?;
    std::fs::write(mode_cache_path(conversation_id), mode)
}

pub fn load_cached_mode(conversation_id: &str) -> Option<String> {
    std::fs::read_to_string(mode_cache_path(conversation_id))
        .ok()
        .map(|mode| mode.trim().to_ascii_lowercase())
}

pub async fn handle_cursor_prompt(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let session = parse_conversation_id(input).unwrap_or_else(|| "default".to_string());
    if let Some(mode) = parse_composer_mode(input) {
        let _ = cache_composer_mode(&session, &mode);
    }
    let _ = send_set(LightState::Yellow, &session, "cursor", "thinking").await?;
    Ok(())
}

pub async fn handle_cursor_stop(input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let session = parse_conversation_id(input).unwrap_or_else(|| "default".to_string());
    let cached = load_cached_mode(&session);
    let reason = stop_light_reason(cached.as_deref());
    let _ = send_set(LightState::Green, &session, "cursor", reason).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn parse_conversation_id_prefers_conversation_id() {
        let input = r#"{"conversation_id":"abc","session_id":"def"}"#;
        assert_eq!(parse_conversation_id(input).as_deref(), Some("abc"));
    }

    #[test]
    fn parse_composer_mode_reads_chat_as_ask() {
        let input = r#"{"composer_mode":"chat","conversation_id":"x"}"#;
        assert_eq!(parse_composer_mode(input).as_deref(), Some("chat"));
        assert_eq!(
            classify_composer_mode(parse_composer_mode(input).unwrap().as_str()),
            CursorComposerMode::Ask
        );
    }

    #[test]
    fn stop_reason_is_idle_for_ask_mode() {
        assert_eq!(stop_light_reason(Some("chat")), "idle");
        assert_eq!(stop_light_reason(Some("ask")), "idle");
    }

    #[test]
    fn stop_reason_is_awaiting_input_for_agent_mode() {
        assert_eq!(stop_light_reason(Some("agent")), AWAITING_INPUT_REASON);
        assert_eq!(stop_light_reason(None), AWAITING_INPUT_REASON);
    }

    #[test]
    fn caches_and_loads_composer_mode_per_conversation() {
        let _guard = test_lock();
        let tmp = tempfile::tempdir().unwrap();
        let sessions = tmp.path().to_path_buf();
        std::env::set_var("SEMAPHORE_SESSIONS_DIR", &sessions);
        let id = "test-conversation-cache";
        cache_composer_mode(id, "chat").unwrap();
        assert_eq!(load_cached_mode(id).as_deref(), Some("chat"));
        std::env::remove_var("SEMAPHORE_SESSIONS_DIR");
    }
}
