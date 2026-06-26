use std::fs;
use std::path::{Path, PathBuf};

use sem_core::config::Config;

/// Copy semctl into ~/.semaphore/bin when available (current exe, bundle, or build output).
pub fn deploy_semctl() -> Result<(), Box<dyn std::error::Error>> {
    let dest = Config::bin_dir().join(if cfg!(windows) {
        "semctl.exe"
    } else {
        "semctl"
    });
    fs::create_dir_all(Config::bin_dir())?;

    if dest.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if fs::metadata(&dest)?.permissions().mode() & 0o111 != 0 {
                return Ok(());
            }
        }
        #[cfg(not(unix))]
        {
            return Ok(());
        }
    }

    let Some(source) = locate_semctl_binary() else {
        return Ok(());
    };

    fs::copy(&source, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)?;
    }

    Ok(())
}

fn locate_semctl_binary() -> Option<PathBuf> {
    let current = std::env::current_exe().ok()?;
    if is_semctl_name(current.file_name()?.to_str()?) {
        return Some(current);
    }

    if let Ok(path) = std::env::var("SEMAPHORE_SEMCTL") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    candidate_paths(&current)
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn is_semctl_name(name: &str) -> bool {
    name == "semctl" || name == "semctl.exe"
}

fn candidate_paths(current_exe: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(parent) = current_exe.parent() {
        // Same folder as the Semaphore binary (dev build or Tauri externalBin).
        paths.push(parent.join("semctl"));
        paths.push(parent.join("semctl.exe"));
        paths.push(parent.join("bin").join("semctl"));
        paths.push(parent.join("bin").join("semctl.exe"));
        // Tauri externalBin sidecar (e.g. semctl-aarch64-apple-darwin).
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("semctl-") {
                    paths.push(entry.path());
                }
            }
        }
        // macOS .app bundle Resources (legacy layout).
        if let Ok(resources) = parent.join("../Resources").canonicalize() {
            paths.push(resources.join("semctl"));
        }
    }

    paths
}

pub fn deploy_from_resource(resource_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let name = if cfg!(windows) { "semctl.exe" } else { "semctl" };
    let source = resource_dir.join(name);
    if !source.exists() {
        return deploy_semctl();
    }
    let dest = Config::bin_dir().join(name);
    fs::create_dir_all(Config::bin_dir())?;
    fs::copy(&source, &dest)?;
    Ok(())
}
