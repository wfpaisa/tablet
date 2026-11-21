//! Release updates: checks GitHub for a newer version, then downloads,
//! verifies and installs the portable archive with its own installer.
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const LATEST: &str = "https://api.github.com/repos/wfpaisa/tablet/releases/latest";
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    archive: String,
    checksum: String,
}

impl Release {
    /// A release for screenshots and tests; it is never downloaded.
    pub fn simulated(version: &str) -> Self {
        Self {
            version: version.into(),
            archive: String::new(),
            checksum: String::new(),
        }
    }
}

/// `1.10.0` > `1.9.3`; a leading `v` is ignored, invalid versions never win.
pub fn is_newer(candidate: &str, current: &str) -> bool {
    fn parse(v: &str) -> Option<(u64, u64, u64)> {
        let mut parts = v.trim().trim_start_matches('v').split('.');
        let version = (
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        );
        parts.next().is_none().then_some(version)
    }
    matches!((parse(candidate), parse(current)), (Some(a), Some(b)) if a > b)
}

fn archive_name() -> String {
    format!("tablet-center-{}-linux.tar.gz", std::env::consts::ARCH)
}

/// Picks this machine's archive and checksum from a GitHub release JSON.
pub fn parse_release(json: &str) -> Result<Release, String> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let version = value["tag_name"]
        .as_str()
        .ok_or("Release without a tag")?
        .trim_start_matches('v')
        .to_string();
    let name = archive_name();
    let url = |wanted: &str| {
        value["assets"].as_array().and_then(|assets| {
            assets.iter().find_map(|asset| {
                (asset["name"].as_str() == Some(wanted))
                    .then(|| asset["browser_download_url"].as_str().map(str::to_string))
                    .flatten()
            })
        })
    };
    Ok(Release {
        archive: url(&name).ok_or_else(|| format!("Release {version} has no {name}"))?,
        checksum: url(&format!("{name}.sha256")).ok_or("Release without a checksum")?,
        version,
    })
}

fn run(command: &mut Command) -> Result<String, String> {
    let program = command.get_program().to_string_lossy().into_owned();
    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        let detail = error.trim().lines().chain(out.trim().lines()).last();
        Err(format!(
            "{program} failed: {}",
            detail.unwrap_or("no output")
        ))
    }
}

fn curl(url: &str) -> Command {
    let mut command = Command::new("curl");
    command.args(["-fsSL", "--max-time", "120", "-A", "tablet-center", url]);
    command
}

/// The latest release when it is newer than this build.
pub fn check() -> Result<Option<Release>, String> {
    let json = run(curl(LATEST).args(["-H", "Accept: application/vnd.github+json"]))?;
    let release = parse_release(&json)?;
    Ok(is_newer(&release.version, CURRENT).then_some(release))
}

/// Downloads and verifies the archive, then runs its installer, which
/// replaces the binaries, the service and the GNOME Shell extension.
pub fn install(release: &Release) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!("tablet-center-update-{}", release.version));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = archive_name();
    run(curl(&release.archive).arg("-o").arg(dir.join(&name)))?;
    run(curl(&release.checksum)
        .arg("-o")
        .arg(dir.join(format!("{name}.sha256"))))?;
    run(Command::new("sha256sum")
        .args(["-c", &format!("{name}.sha256")])
        .current_dir(&dir))?;
    run(Command::new("tar").arg("-xzf").arg(&name).current_dir(&dir))?;
    let root = installer_dir(&dir)?;
    run(Command::new("sh").arg("install.sh").current_dir(&root)).map_err(|e| {
        format!(
            "{e}. Run {} in a terminal to finish.",
            root.join("install.sh").display()
        )
    })?;
    Ok(())
}

fn installer_dir(dir: &Path) -> Result<PathBuf, String> {
    let root = dir.join("tablet-center");
    if root.join("install.sh").is_file() {
        Ok(root)
    } else {
        Err("The archive has no installer".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("1.4.1", "1.4.0"));
        assert!(is_newer("v1.10.0", "1.9.9"));
        assert!(is_newer("2.0.0", "1.99.99"));
        assert!(!is_newer("1.4.0", "1.4.0"));
        assert!(!is_newer("1.3.9", "1.4.0"));
        assert!(!is_newer("1.5", "1.4.0"));
        assert!(!is_newer("1.5.0-beta", "1.4.0"));
    }

    #[test]
    fn parses_github_release() {
        let name = archive_name();
        let json = format!(
            r#"{{"tag_name":"v1.5.0","assets":[
                {{"name":"{name}","browser_download_url":"https://x/a"}},
                {{"name":"{name}.sha256","browser_download_url":"https://x/a.sha256"}}]}}"#
        );
        let release = parse_release(&json).unwrap();
        assert_eq!(release.version, "1.5.0");
        assert_eq!(release.archive, "https://x/a");
        assert_eq!(release.checksum, "https://x/a.sha256");
        assert!(parse_release(r#"{"tag_name":"v1.5.0","assets":[]}"#).is_err());
    }
}
