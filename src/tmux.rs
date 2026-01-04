use std::process::Command;

#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub attached: bool,
}

pub fn list_sessions() -> Vec<TmuxSession> {
    // Use tab as delimiter to handle session names containing colons
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}\t#{session_windows}\t#{session_attached}"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        Some(TmuxSession {
                            name: parts[0].to_string(),
                            windows: parts[1].parse().unwrap_or(0),
                            attached: parts[2] == "1",
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

pub fn create_session(name: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["new-session", "-d", "-s", name])
        .status()
        .map_err(|e| format!("Failed to create session: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to create tmux session".to_string())
    }
}

pub fn rename_session(old_name: &str, new_name: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["rename-session", "-t", old_name, new_name])
        .status()
        .map_err(|e| format!("Failed to rename session: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to rename tmux session".to_string())
    }
}

pub fn kill_session(name: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["kill-session", "-t", name])
        .status()
        .map_err(|e| format!("Failed to kill session: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err("Failed to kill tmux session".to_string())
    }
}

pub fn attach_session(name: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["switch-client", "-t", name])
        .status()
        .map_err(|e| format!("Failed to attach session: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        // If switch-client fails (not in tmux), try attach-session
        let status = Command::new("tmux")
            .args(["attach-session", "-t", name])
            .status()
            .map_err(|e| format!("Failed to attach session: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to attach to tmux session".to_string())
        }
    }
}

