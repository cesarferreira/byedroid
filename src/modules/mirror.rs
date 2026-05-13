//! Launch scrcpy for the selected device.
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

pub fn launch_scrcpy(serial: &str, extra_args: &[String]) -> Result<()> {
    let scrcpy = which("scrcpy").map_err(|_| anyhow!("scrcpy not found in PATH"))?;
    let mut cmd = Command::new(scrcpy);
    cmd.arg("-s").arg(serial);
    for a in extra_args {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn()
        .map_err(|e| anyhow!("failed to spawn scrcpy: {e}"))?;
    Ok(())
}

#[allow(dead_code)]
pub fn scrcpy_path() -> Option<PathBuf> {
    which("scrcpy").ok()
}

/// Query `dumpsys display` and return the display ID of any active scrcpy virtual display.
/// Used to target `am start --display <id>` when the user has scrcpy running with `--new-display`.
pub fn detect_scrcpy_display(adb_path: &Path, serial: &str) -> Option<u32> {
    let out = Command::new(adb_path)
        .args(["-s", serial, "shell", "dumpsys", "display"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        if line.contains("mBaseDisplayInfo") && line.contains("scrcpy") {
            if let Some(pos) = line.find("displayId ") {
                let rest = &line[pos + "displayId ".len()..];
                let id_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(id) = id_str.parse::<u32>() {
                    return Some(id);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_display_id_from_dumpsys_line() {
        // Simulate the dumpsys output line containing a scrcpy virtual display
        let line = r#"    mBaseDisplayInfo=DisplayInfo{"scrcpy", displayId 13, displayGroupId 1, FLAG_PRESENTATION}"#;
        // Extract directly since we can't call adb in unit tests — verify the parsing logic
        let pos = line.find("displayId ").unwrap();
        let rest = &line[pos + "displayId ".len()..];
        let id_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        assert_eq!(id_str.parse::<u32>().unwrap(), 13);
    }
}
