use std::fs;
use std::path::{Path, PathBuf};

use sem_core::config::Config;

const MARKER: &str = "_semaphore";
const MARKER_LAUNCH: &str = "_semaphore_launch";

const ALL_TOOLS: &[&str] = &[
    "cursor",
    "claude-code",
    "codex",
    "gemini-cli",
    "copilot-cli",
];

pub fn run_install(all: bool, tool: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    ensure_binaries()?;

    let tools: Vec<&str> = if all {
        ALL_TOOLS.to_vec()
    } else {
        vec![tool.ok_or("specify a tool or use --all")?]
    };

    for tool in tools {
        install_tool(tool)?;
        println!("installed hooks for {tool}");
    }

    sync_launch_hooks()?;
    Ok(())
}

pub fn run_uninstall(all: bool, tool: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let tools: Vec<&str> = if all {
        ALL_TOOLS.to_vec()
    } else {
        vec![tool.ok_or("specify a tool or use --all")?]
    };

    for tool in tools {
        uninstall_tool(tool)?;
        println!("removed hooks for {tool}");
    }
    Ok(())
}

fn install_tool(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        "cursor" => install_cursor(),
        "claude-code" => install_claude_code(),
        "codex" => install_codex(),
        "gemini-cli" => install_gemini_cli(),
        "copilot-cli" => install_copilot_cli(),
        other => Err(format!("unknown tool: {other}").into()),
    }
}

fn uninstall_tool(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        "cursor" => uninstall_cursor(),
        "claude-code" => uninstall_claude_code(),
        "codex" => uninstall_codex(),
        "gemini-cli" => uninstall_gemini_cli(),
        "copilot-cli" => uninstall_copilot_cli(),
        other => Err(format!("unknown tool: {other}").into()),
    }
}

pub fn prepare_runtime() -> Result<(), Box<dyn std::error::Error>> {
    ensure_binaries()
}

fn ensure_binaries() -> Result<(), Box<dyn std::error::Error>> {
    let bin_dir = Config::bin_dir();
    fs::create_dir_all(&bin_dir)?;

    let sem_hook_path = bin_dir.join(if cfg!(windows) { "sem-hook.bat" } else { "sem-hook" });
    if !sem_hook_path.exists() {
        write_sem_hook(&sem_hook_path)?;
    }

    let _ = crate::deploy::deploy_semctl();
    Ok(())
}

fn write_sem_hook(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        let script = r#"#!/usr/bin/env bash
set -euo pipefail
STATE="${1:-yellow}"
REASON="${2:-event}"
SOURCE="${3:-hook}"
SESSION="default"
if [ ! -t 0 ]; then
  INPUT="$(cat)"
  PARSED="$(printf '%s' "$INPUT" | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
  if [ -z "$PARSED" ]; then
    PARSED="$(printf '%s' "$INPUT" | sed -n 's/.*"conversation_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
  fi
  if [ -z "$PARSED" ]; then
    PARSED="$(printf '%s' "$INPUT" | sed -n 's/.*"sessionId"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
  fi
  SESSION="${PARSED:-default}"
fi
SEMCTL="${SEMAPHORE_BIN:-$HOME/.semaphore/bin/semctl}"
if [ -x "$SEMCTL" ]; then
  "$SEMCTL" set "$STATE" --session "$SESSION" --source "$SOURCE" --reason "$REASON" >/dev/null 2>&1 || true
fi
exit 0
"#;
        fs::write(path, script)?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }

    #[cfg(windows)]
    {
        let script = r#"@echo off
setlocal
set STATE=%~1
if "%STATE%"=="" set STATE=yellow
set REASON=%~2
if "%REASON%"=="" set REASON=event
set SOURCE=%~3
if "%SOURCE%"=="" set SOURCE=hook
set SESSION=default
set SEMCTL=%USERPROFILE%\.semaphore\bin\semctl.exe
if exist "%SEMCTL%" (
  "%SEMCTL%" set %STATE% --session %SESSION% --source %SOURCE% --reason %REASON% >nul 2>&1
)
exit /b 0
"#;
        fs::write(path, script)?;
    }
    Ok(())
}

fn hook_command(state: &str, reason: &str) -> String {
    let hook = Config::bin_dir().join(if cfg!(windows) {
        "sem-hook.bat"
    } else {
        "sem-hook"
    });
    format!("{} {} {}", hook.display(), state, reason)
}

fn semctl_subcommand_command(subcommand: &str) -> String {
    let semctl = Config::bin_dir().join(if cfg!(windows) {
        "semctl.exe"
    } else {
        "semctl"
    });
    format!("{} {}", semctl.display(), subcommand)
}

fn launch_hook_command() -> String {
    let semctl = Config::bin_dir().join(if cfg!(windows) {
        "semctl.exe"
    } else {
        "semctl"
    });
    format!("{} launch", semctl.display())
}

/// Add or remove session-start hooks that launch Semaphore when connected AI tools start.
pub fn sync_launch_hooks() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();
    ensure_binaries()?;

    for tool in ALL_TOOLS {
        let connected = crate::detect::hook_file_contains(&crate::detect::tool_config_path(
            &home_dir(),
            tool,
        ));
        if !connected {
            continue;
        }
        if config.launch_with_tools {
            install_launch_hook(tool)?;
        } else {
            remove_launch_hook(tool)?;
        }
    }
    Ok(())
}

fn install_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".cursor/hooks.json");
    remove_marked_hooks(&path, "hooks")?;
    merge_cursor_hooks(&path)
}

fn uninstall_cursor() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".cursor/hooks.json");
    remove_marked_hooks(&path, "hooks")
}

fn install_claude_code() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".claude/settings.json");
    merge_claude_hooks(&path)
}

fn uninstall_claude_code() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".claude/settings.json");
    remove_marked_hooks(&path, "hooks")
}

fn install_codex() -> Result<(), Box<dyn std::error::Error>> {
    enable_codex_hooks()?;
    let path = home_dir().join(".codex/hooks.json");
    merge_codex_hooks(&path)
}

fn uninstall_codex() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".codex/hooks.json");
    remove_marked_hooks(&path, "hooks")
}

fn install_gemini_cli() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".gemini/settings.json");
    merge_gemini_hooks(&path)
}

fn uninstall_gemini_cli() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".gemini/settings.json");
    remove_marked_hooks(&path, "hooks")
}

fn install_copilot_cli() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".copilot/hooks.json");
    merge_copilot_hooks(&path)
}

fn uninstall_copilot_cli() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".copilot/hooks.json");
    remove_marked_hooks(&path, "hooks")
}

fn enable_codex_hooks() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".codex/config.toml");
    fs::create_dir_all(path.parent().unwrap())?;
    let content = if path.exists() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    if content.contains("codex_hooks") {
        return Ok(());
    }
    let mut updated = content;
    if !updated.ends_with('\n') && !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str("\n[features]\ncodex_hooks = true\n");
    fs::write(path, updated)?;
    Ok(())
}

fn merge_cursor_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "version": 1, "hooks": {} })
    };

    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid cursor hooks.json structure")?;

    insert_cursor_semctl_hook(hooks, "beforeSubmitPrompt", "cursor-prompt");
    insert_hook(hooks, "afterAgentThought", "yellow", "thinking", None);
    insert_hook(
        hooks,
        "preToolUse",
        "red",
        "writing",
        Some("Write|Edit"),
    );
    insert_hook(hooks, "afterFileEdit", "red", "writing", None);
    insert_hook(
        hooks,
        "postToolUse",
        "yellow",
        "thinking",
        Some("Write|Edit|Shell"),
    );
    insert_hook(hooks, "beforeShellExecution", "green", "awaiting-input", None);
    insert_hook(hooks, "afterShellExecution", "yellow", "thinking", None);
    insert_hook(hooks, "beforeMCPExecution", "green", "awaiting-input", None);
    insert_hook(hooks, "afterMCPExecution", "yellow", "thinking", None);
    insert_cursor_semctl_hook(hooks, "stop", "cursor-stop");
    insert_hook(hooks, "sessionEnd", "green", "idle", None);

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_codex_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "hooks": {} })
    };
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid codex hooks.json structure")?;

    insert_codex_hook(hooks, "UserPromptSubmit", "yellow", "thinking", "");
    insert_codex_hook(hooks, "PreToolUse", "red", "writing", "Bash");
    insert_codex_hook(hooks, "PostToolUse", "yellow", "thinking", "");
    insert_codex_hook(hooks, "Stop", "green", "awaiting-input", "");

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_gemini_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "hooks": {} })
    };
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid gemini settings.json structure")?;

    insert_gemini_hook(hooks, "BeforeAgent", "yellow", "thinking", "");
    insert_gemini_hook(hooks, "BeforeModel", "yellow", "thinking", "");
    insert_gemini_hook(hooks, "BeforeTool", "red", "writing", "write_.*");
    insert_gemini_hook(hooks, "AfterTool", "yellow", "thinking", "");
    insert_gemini_hook(hooks, "AfterAgent", "green", "awaiting-input", "");
    insert_gemini_hook(hooks, "SessionEnd", "green", "idle", "");

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_copilot_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "hooks": {} })
    };
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid copilot hooks.json structure")?;

    insert_codex_hook(hooks, "UserPromptSubmit", "yellow", "thinking", "");
    insert_codex_hook(hooks, "PreToolUse", "red", "writing", "Write|Edit|Bash");
    insert_codex_hook(hooks, "PostToolUse", "yellow", "thinking", "");
    insert_codex_hook(hooks, "Stop", "green", "awaiting-input", "");

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn insert_codex_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: &str,
) {
    insert_claude_hook(hooks, event, state, reason, matcher);
}

fn insert_gemini_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: &str,
) {
    let hook_entry = serde_json::json!({
        "type": "command",
        "command": hook_command(state, reason),
        "_semaphore": true
    });

    let event_list = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));

    if matcher.is_empty() {
        push_gemini_block(event_list, hook_entry, None);
        return;
    }
    push_gemini_block(event_list, hook_entry, Some(matcher));
}

fn push_gemini_block(
    event_list: &mut serde_json::Value,
    hook_entry: serde_json::Value,
    matcher: Option<&str>,
) {
    if let Some(arr) = event_list.as_array_mut() {
        let exists = arr.iter().any(|block| block.get(MARKER) == Some(&serde_json::Value::Bool(true)));
        if exists {
            return;
        }
        let mut block = serde_json::json!({ "hooks": [hook_entry], "_semaphore": true });
        if let Some(m) = matcher {
            block["matcher"] = serde_json::Value::String(m.to_string());
        }
        arr.push(block);
    }
}

fn merge_claude_hooks(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut root: serde_json::Value = if path.exists() {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else {
        serde_json::json!({ "hooks": {} })
    };

    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid claude settings.json structure")?;

    insert_claude_hook(hooks, "UserPromptSubmit", "yellow", "thinking", "");
    insert_claude_hook(hooks, "PreToolUse", "red", "writing", "Write|Edit|Bash");
    insert_claude_hook(hooks, "PostToolUse", "yellow", "thinking", "");
    insert_claude_hook(hooks, "Stop", "green", "awaiting-input", "");
    insert_claude_hook(hooks, "SessionEnd", "green", "idle", "");

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn insert_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: Option<&str>,
) {
    insert_command_hook(hooks, event, &hook_command(state, reason), matcher);
}

fn insert_cursor_semctl_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    subcommand: &str,
) {
    insert_command_hook(hooks, event, &semctl_subcommand_command(subcommand), None);
}

fn insert_command_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    command: &str,
    matcher: Option<&str>,
) {
    let mut entry = serde_json::json!({
        "command": command,
        "_semaphore": true
    });
    if let Some(m) = matcher {
        entry["matcher"] = serde_json::Value::String(m.to_string());
    }
    let list = hooks.entry(event.to_string()).or_insert_with(|| serde_json::json!([]));
    if let Some(arr) = list.as_array_mut() {
        if !arr.iter().any(|v| v.get(MARKER) == Some(&serde_json::Value::Bool(true))) {
            arr.push(entry);
        }
    }
}

fn insert_claude_hook(
    hooks: &mut serde_json::Map<String, serde_json::Value>,
    event: &str,
    state: &str,
    reason: &str,
    matcher: &str,
) {
    let hook_entry = serde_json::json!({
        "type": "command",
        "command": hook_command(state, reason),
        "_semaphore": true
    });

    let event_list = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));

    if matcher.is_empty() {
        if let Some(arr) = event_list.as_array_mut() {
            if !arr.iter().any(|v| {
                v.get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
            }) {
                arr.push(serde_json::json!({
                    "hooks": [hook_entry]
                }));
            }
        }
        return;
    }

    if let Some(arr) = event_list.as_array_mut() {
        let exists = arr.iter().any(|block| {
            block.get(MARKER) == Some(&serde_json::Value::Bool(true))
                || block
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
        });
        if !exists {
            arr.push(serde_json::json!({
                "matcher": matcher,
                "hooks": [hook_entry],
                "_semaphore": true
            }));
        }
    }
}

fn remove_marked_hooks(path: &Path, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(hooks) = root.get_mut(key).and_then(|v| v.as_object_mut()) else {
        return Ok(());
    };

    for (_event, value) in hooks.clone().iter() {
        // handled per event below
        let _ = value;
    }

    let events: Vec<String> = hooks.keys().cloned().collect();
    for event in events {
        let Some(value) = hooks.get_mut(&event) else { continue };
        if let Some(arr) = value.as_array_mut() {
            arr.retain(|entry| entry.get(MARKER) != Some(&serde_json::Value::Bool(true)));
            arr.retain(|entry| {
                !entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
            });
            for entry in arr.iter_mut() {
                if let Some(inner) = entry.get_mut("hooks").and_then(|v| v.as_array_mut()) {
                    inner.retain(|h| h.get(MARKER) != Some(&serde_json::Value::Bool(true)));
                }
            }
            if arr.is_empty() {
                hooks.remove(&event);
            }
        }
    }

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn install_launch_hook(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    match tool {
        "cursor" => merge_cursor_launch_hook(),
        "claude-code" => merge_claude_launch_hook(),
        "codex" => merge_codex_launch_hook(),
        "gemini-cli" => merge_gemini_launch_hook(),
        "copilot-cli" => merge_copilot_launch_hook(),
        _ => Ok(()),
    }
}

fn remove_launch_hook(tool: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = match tool {
        "cursor" => home_dir().join(".cursor/hooks.json"),
        "claude-code" => home_dir().join(".claude/settings.json"),
        "codex" => home_dir().join(".codex/hooks.json"),
        "gemini-cli" => home_dir().join(".gemini/settings.json"),
        "copilot-cli" => home_dir().join(".copilot/hooks.json"),
        _ => return Ok(()),
    };
    remove_marked_launch_hooks(&path, "hooks")
}

fn merge_cursor_launch_hook() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".cursor/hooks.json");
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid cursor hooks.json structure")?;
    insert_launch_hook(hooks, "sessionStart");
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_claude_launch_hook() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".claude/settings.json");
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid claude settings.json structure")?;
    insert_claude_launch_hook(hooks, "SessionStart");
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_codex_launch_hook() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".codex/hooks.json");
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid codex hooks.json structure")?;
    insert_claude_launch_hook(hooks, "SessionStart");
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_gemini_launch_hook() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".gemini/settings.json");
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid gemini settings.json structure")?;
    insert_claude_launch_hook(hooks, "SessionStart");
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn merge_copilot_launch_hook() -> Result<(), Box<dyn std::error::Error>> {
    let path = home_dir().join(".copilot/hooks.json");
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let hooks = root
        .as_object_mut()
        .and_then(|o| o.get_mut("hooks"))
        .and_then(|v| v.as_object_mut())
        .ok_or("invalid copilot hooks.json structure")?;
    insert_claude_launch_hook(hooks, "SessionStart");
    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn insert_launch_hook(hooks: &mut serde_json::Map<String, serde_json::Value>, event: &str) {
    let entry = serde_json::json!({
        "command": launch_hook_command(),
        MARKER_LAUNCH: true
    });
    let list = hooks.entry(event.to_string()).or_insert_with(|| serde_json::json!([]));
    if let Some(arr) = list.as_array_mut() {
        if !arr.iter().any(|v| v.get(MARKER_LAUNCH) == Some(&serde_json::Value::Bool(true))) {
            arr.push(entry);
        }
    }
}

fn insert_claude_launch_hook(hooks: &mut serde_json::Map<String, serde_json::Value>, event: &str) {
    let hook_entry = serde_json::json!({
        "type": "command",
        "command": launch_hook_command(),
        MARKER_LAUNCH: true
    });
    let event_list = hooks
        .entry(event.to_string())
        .or_insert_with(|| serde_json::json!([]));
    if let Some(arr) = event_list.as_array_mut() {
        let exists = arr.iter().any(|block| {
            block.get(MARKER_LAUNCH) == Some(&serde_json::Value::Bool(true))
                || block
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER_LAUNCH) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
        });
        if !exists {
            arr.push(serde_json::json!({
                "hooks": [hook_entry],
                MARKER_LAUNCH: true
            }));
        }
    }
}

fn remove_marked_launch_hooks(path: &Path, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(());
    }
    let mut root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(hooks) = root.get_mut(key).and_then(|v| v.as_object_mut()) else {
        return Ok(());
    };

    let events: Vec<String> = hooks.keys().cloned().collect();
    for event in events {
        let Some(value) = hooks.get_mut(&event) else {
            continue;
        };
        if let Some(arr) = value.as_array_mut() {
            arr.retain(|entry| entry.get(MARKER_LAUNCH) != Some(&serde_json::Value::Bool(true)));
            arr.retain(|entry| {
                !entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|h| {
                            h.get(MARKER_LAUNCH) == Some(&serde_json::Value::Bool(true))
                        })
                    })
                    .unwrap_or(false)
            });
            for entry in arr.iter_mut() {
                if let Some(inner) = entry.get_mut("hooks").and_then(|v| v.as_array_mut()) {
                    inner.retain(|h| h.get(MARKER_LAUNCH) != Some(&serde_json::Value::Bool(true)));
                }
            }
            if arr.is_empty() {
                hooks.remove(&event);
            }
        }
    }

    fs::write(path, serde_json::to_string_pretty(&root)?)?;
    Ok(())
}

fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile);
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn merge_cursor_hooks_preserves_existing_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hooks.json");
        fs::write(
            &path,
            r#"{
  "version": 1,
  "hooks": {
    "customEvent": [{ "command": "echo hello" }]
  }
}"#,
        )
        .unwrap();

        merge_cursor_hooks(&path).unwrap();

        let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let hooks = content.get("hooks").unwrap().as_object().unwrap();
        assert!(hooks.contains_key("customEvent"));
        assert!(hooks.contains_key("stop"));
        let stop_hooks = hooks.get("stop").unwrap().as_array().unwrap();
        assert!(stop_hooks
            .iter()
            .any(|h| h.get(MARKER) == Some(&serde_json::Value::Bool(true))));
        assert!(stop_hooks.iter().any(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.contains("cursor-stop"))
        }));
        let prompt_hooks = hooks.get("beforeSubmitPrompt").unwrap().as_array().unwrap();
        assert!(prompt_hooks.iter().any(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.contains("cursor-prompt"))
        }));
    }

    #[test]
    fn merge_cursor_hooks_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hooks.json");
        merge_cursor_hooks(&path).unwrap();
        let first = fs::read_to_string(&path).unwrap();
        merge_cursor_hooks(&path).unwrap();
        let second = fs::read_to_string(&path).unwrap();
        assert_eq!(first, second);
    }
}
