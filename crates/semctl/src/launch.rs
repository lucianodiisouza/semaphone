use std::process::Command;

const BUNDLE_ID: &str = "dev.semaphore.app";
const APP_NAME: &str = "Semaphore";

pub fn launch_app() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        if Command::new("open")
            .args(["-gb", BUNDLE_ID])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Ok(());
        }
        Command::new("open")
            .args(["-a", APP_NAME])
            .status()
            .map_err(|e| format!("failed to launch {APP_NAME}: {e}"))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let started = Command::new("cmd")
            .args(["/C", "start", "", APP_NAME])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if started {
            return Ok(());
        }
        return Err(format!("failed to launch {APP_NAME}").into());
    }

    #[cfg(target_os = "linux")]
    {
        for args in [
            vec!["gtk-launch", "semaphore"],
            vec!["xdg-open", "semaphore://"],
        ] {
            if Command::new(args[0])
                .args(&args[1..])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return Ok(());
            }
        }
        return Err(format!("failed to launch {APP_NAME}").into());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("launch is not supported on this platform".into())
    }
}
