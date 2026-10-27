//! Desktop integration, compatible configuration files and backup operations.
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use tablet_config::AppInfo;
/// Data directories in lookup order: `$XDG_DATA_HOME`, then `$XDG_DATA_DIRS`.
fn data_dirs() -> Vec<PathBuf> {
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    let dirs =
        std::env::var_os("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    data_home
        .into_iter()
        .chain(std::env::split_paths(&dirs))
        .collect()
}

/// What decides whether a desktop entry shows in the app grid.
pub struct DesktopEnv {
    /// `$XDG_CURRENT_DESKTOP` names, for `OnlyShowIn` and `NotShowIn`.
    pub desktops: Vec<String>,
    /// Locale keys for `Name[..]`, most specific first.
    pub langs: Vec<String>,
    /// `$PATH`, where `TryExec` and `Exec` programs must exist.
    pub path: Vec<PathBuf>,
}

impl DesktopEnv {
    pub fn current() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let locale = var("LC_ALL")
            .or_else(|| var("LC_MESSAGES"))
            .or_else(|| var("LANG"))
            .unwrap_or_default();
        let locale = locale.split(['.', '@']).next().unwrap_or_default();
        let mut langs = Vec::new();
        if !locale.is_empty() && locale != "C" && locale != "POSIX" {
            langs.push(locale.to_string());
            if let Some((lang, _)) = locale.split_once('_') {
                langs.push(lang.to_string());
            }
        }
        Self {
            desktops: var("XDG_CURRENT_DESKTOP")
                .unwrap_or_default()
                .split(':')
                .filter(|d| !d.is_empty())
                .map(String::from)
                .collect(),
            langs,
            path: std::env::var_os("PATH")
                .map(|p| std::env::split_paths(&p).collect())
                .unwrap_or_default(),
        }
    }
    fn has_program(&self, program: &str) -> bool {
        if program.contains('/') {
            Path::new(program).is_file()
        } else {
            self.path.iter().any(|dir| dir.join(program).is_file())
        }
    }
}

/// Apps the desktop shows in its app grid, as GLib's `g_app_info_should_show`.
pub fn installed_apps() -> Vec<AppInfo> {
    let env = DesktopEnv::current();
    let mut apps = Vec::new();
    let mut seen = HashSet::new();
    for dir in data_dirs() {
        let base = dir.join("applications");
        scan(&base, &base, &env, &mut seen, &mut apps);
    }
    apps.sort_by_cached_key(|a| a.name.to_lowercase());
    apps
}
/// Installed apps with a live process, from the systemd scope GNOME (or
/// Flatpak) starts each app in: `app-gnome-org.gnome.Calculator-1234.scope`.
/// Works without the Shell extension.
pub fn running_apps(installed: &[AppInfo]) -> Vec<AppInfo> {
    let mut ids = HashSet::new();
    for entry in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        if let Ok(text) = std::fs::read_to_string(entry.path().join("cgroup")) {
            ids.extend(text.split('/').filter_map(scope_app_id));
        }
    }
    installed
        .iter()
        .filter(|a| ids.contains(&a.id.to_lowercase()))
        .cloned()
        .collect()
}
/// The desktop id in a unit name like `app-gnome-foo\x2dbar-12.scope`, lowercased.
pub fn scope_app_id(unit: &str) -> Option<String> {
    let unit = unit.trim().strip_prefix("app-")?;
    let name = unit
        .strip_suffix(".scope")
        .or_else(|| unit.strip_suffix(".service"))?;
    let name = name
        .rsplit_once('-')
        .filter(|(_, n)| n.chars().all(|c| c.is_ascii_digit()))
        .map_or(name, |(id, _)| id);
    let name = ["gnome-", "flatpak-"]
        .iter()
        .find_map(|p| name.strip_prefix(p))
        .unwrap_or(name);
    Some(name.replace("\\x2d", "-").to_lowercase())
}
fn scan(
    dir: &Path,
    base: &Path,
    env: &DesktopEnv,
    seen: &mut HashSet<String>,
    apps: &mut Vec<AppInfo>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            scan(&path, base, env, seen, apps);
            continue;
        }
        if path.extension().is_none_or(|s| s != "desktop") {
            continue;
        }
        let Ok(relative) = path.strip_prefix(base) else {
            continue;
        };
        let id = relative.to_string_lossy().replace('/', "-");
        // The first directory wins, even when its entry hides the app.
        if !seen.insert(id.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Some(app) = parse_with(&id, &text, env) {
            apps.push(app);
        }
    }
}
pub fn parse(id: &str, text: &str) -> Option<AppInfo> {
    parse_with(id, text, &DesktopEnv::current())
}
pub fn parse_with(id: &str, text: &str, env: &DesktopEnv) -> Option<AppInfo> {
    let mut fields = HashMap::new();
    let mut desktop = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            desktop = line == "[Desktop Entry]";
        } else if desktop && !line.starts_with('#') {
            if let Some((k, v)) = line.split_once('=') {
                fields.insert(k.trim(), v.trim());
            }
        }
    }
    let listed = |key: &str| {
        fields.get(key).map(|v| {
            v.split(';')
                .any(|d| env.desktops.iter().any(|c| c.eq_ignore_ascii_case(d)))
        })
    };
    // First word of Exec, which may be quoted.
    let exec = fields.get("Exec").and_then(|e| {
        e.strip_prefix('"')
            .and_then(|q| q.split('"').next())
            .or_else(|| e.split_whitespace().next())
    });
    if fields.get("Type") != Some(&"Application")
        || ["Hidden", "NoDisplay"]
            .iter()
            .any(|k| fields.get(k) == Some(&"true"))
        || listed("OnlyShowIn") == Some(false)
        || listed("NotShowIn") == Some(true)
        || fields
            .get("TryExec")
            .is_some_and(|p| !p.is_empty() && !env.has_program(p))
        || exec.is_some_and(|p| !env.has_program(p))
    {
        return None;
    }
    let name = env
        .langs
        .iter()
        .find_map(|lang| fields.get(format!("Name[{lang}]").as_str()))
        .or_else(|| fields.get("Name"))?;
    Some(AppInfo {
        id: id.strip_suffix(".desktop").unwrap_or(id).into(),
        name: name.to_string(),
        icon: fields.get("Icon").map(|v| v.to_string()),
    })
}

/// Encoded image of an app icon: its loader URI (the extension picks the
/// decoder) and the file bytes.
pub type IconData = (String, std::sync::Arc<[u8]>);

/// Reads the art for an `Icon=` value or a GIcon string: a file path, or theme
/// names looked up in hicolor, Adwaita and pixmaps, preferring scalable art.
pub fn icon_data(icon: &str) -> Option<IconData> {
    // GThemedIcon strings list fallback names: ". GThemedIcon name1 name2".
    let names: Vec<&str> = match icon.strip_prefix(". GThemedIcon ") {
        Some(rest) => rest.split_whitespace().collect(),
        None => vec![icon],
    };
    let path = names.iter().find_map(|name| icon_file(name))?;
    let bytes = std::fs::read(&path).ok()?;
    // Some launchers point at files without an extension; sniff those.
    let ext = if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"))
        || bytes.starts_with(b"<svg")
        || bytes.starts_with(b"<?xml")
    {
        "svg"
    } else {
        return None;
    };
    Some((
        format!("bytes://app-icon/{}.{ext}", path.display()),
        bytes.into(),
    ))
}
fn icon_file(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    let mut roots: Vec<PathBuf> = data_dirs().into_iter().map(|d| d.join("icons")).collect();
    if let Some(home) = std::env::var_os("HOME") {
        roots.insert(0, PathBuf::from(home).join(".icons"));
    }
    const SIZES: [&str; 8] = [
        "scalable", "64x64", "48x48", "128x128", "96x96", "256x256", "32x32", "512x512",
    ];
    // Any context folder: apps, categories, legacy...
    for theme in ["hicolor", "Adwaita", "AdwaitaLegacy"] {
        for size in SIZES {
            for root in &roots {
                let Ok(contexts) = std::fs::read_dir(root.join(theme).join(size)) else {
                    continue;
                };
                for context in contexts.flatten() {
                    for ext in ["svg", "png"] {
                        let file = context.path().join(format!("{name}.{ext}"));
                        if file.is_file() {
                            return Some(file);
                        }
                    }
                }
            }
        }
    }
    ["svg", "png"].iter().find_map(|ext| {
        let file = PathBuf::from(format!("/usr/share/pixmaps/{name}.{ext}"));
        file.is_file().then_some(file)
    })
}

use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tablet_config::Config;
pub fn restart_daemon() -> Result<(), String> {
    let ok = Command::new("systemctl")
        .args(["--user", "restart", "tabletd.service"])
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok {
        Ok(())
    } else {
        Err("systemctl --user restart tabletd.service failed".into())
    }
}
fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".into())
}

pub fn autostart_path() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/autostart/tablet-center.desktop"))
}

pub fn autostart_enabled() -> Result<bool, String> {
    Ok(autostart_path()?.is_file())
}

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let path = autostart_path()?;
    if enabled {
        fs::create_dir_all(path.parent().ok_or("Invalid autostart path")?)
            .map_err(|e| e.to_string())?;
        let binary = home()?.join(".local/bin/tablet-center");
        let binary = binary
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('$', "\\$")
            .replace('%', "%%");
        fs::write(path, format!("[Desktop Entry]\nType=Application\nName=Tablet Center\nExec=\"{binary}\"\nIcon=tablet-center\nX-GNOME-Autostart-enabled=true\n"))
            .map_err(|e| e.to_string())
    } else {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn backups_dir() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/tablet-center/backups"))
}

fn backup_path(filename: &str) -> Result<PathBuf, String> {
    if !filename.ends_with(".json")
        || filename.starts_with('.')
        || !filename
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("Invalid backup filename".into());
    }
    let path = backups_dir()?.join(filename);
    if fs::symlink_metadata(&path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err("Invalid backup file".into());
    }
    Ok(path)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Backup {
    title: String,
    config: Config,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupInfo {
    pub filename: String,
    pub title: String,
    pub created: u64,
}

pub fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let dir = backups_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut backups = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        let Ok(path) = backup_path(&filename) else {
            continue;
        };
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let Ok(backup) =
            serde_json::from_slice::<Backup>(&fs::read(path).map_err(|e| e.to_string())?)
        else {
            continue;
        };
        let created = entry
            .metadata()
            .and_then(|m| m.modified())
            .map_err(|e| e.to_string())?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        backups.push(BackupInfo {
            filename,
            title: backup.title,
            created,
        });
    }
    backups.sort_by(|a, b| {
        b.created
            .cmp(&a.created)
            .then_with(|| a.filename.cmp(&b.filename))
    });
    Ok(backups)
}

pub fn create_backup(title: String) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err("Backup title must contain 1 to 120 characters".into());
    }
    fs::create_dir_all(backups_dir()?).map_err(|e| e.to_string())?;
    let name: String = title
        .chars()
        .take(60)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = backup_path(&format!("{name}-{timestamp}.json"))?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(
        &mut file,
        &Backup {
            title: title.into(),
            config: tablet_config::load(),
        },
    )
    .map_err(|e| e.to_string())
}

pub fn restore_backup(filename: String) -> Result<bool, String> {
    let bytes = fs::read(backup_path(&filename)?).map_err(|e| e.to_string())?;
    let backup: Backup = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    save_config(backup.config)
}

pub fn delete_backup(filename: String) -> Result<(), String> {
    fs::remove_file(backup_path(&filename)?).map_err(|e| e.to_string())
}

pub fn open_backups_folder() -> Result<(), String> {
    let dir = backups_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if Command::new("xdg-open")
        .arg(dir)
        .status()
        .map_err(|e| e.to_string())?
        .success()
    {
        Ok(())
    } else {
        Err("Could not open backup folder".into())
    }
}

pub fn reset_config() -> Result<bool, String> {
    save_config(Config::default())
}

pub fn save_config(config: Config) -> Result<bool, String> {
    let _guard = CONFIG_IO.lock().map_err(|e| e.to_string())?;
    CONFIG_EPOCH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    tablet_config::save(&config).map_err(|e| e.to_string())?;
    Ok(crate::daemon::Client::default()
        .ask(&tablet_config::Request::Reload)
        .is_ok())
}
pub fn save_shell_menus(menus: serde_json::Value) -> Result<(), String> {
    let path = tablet_config::shell_menus_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, menus.to_string())
        .and_then(|()| fs::rename(&tmp, path))
        .map_err(|e| e.to_string())
}

/// The extension consumes the same resolved menu schema as the HTML frontend.
pub fn shell_menus(config: &Config, locale: &crate::i18n::Locale) -> serde_json::Value {
    use crate::{classic::shortcuts as menus, theme::shortcuts_layout as sizes};
    use tablet_config::Shortcut;
    let mut result = serde_json::Map::new();
    for key in [
        Shortcut::ExpressMenu,
        Shortcut::RadialMenu,
        Shortcut::AppShortcuts,
        Shortcut::Keypad,
    ] {
        let menu = menus::menu(&config.shortcuts, key);
        let items: Vec<_> = menus::items(&config.shortcuts, key).iter().enumerate().map(|(index,item)| {
            let name=menus::item_icon(item);
            let icon=match name.as_str() {
                "undo"=>"edit-undo-symbolic", "redo"=>"edit-redo-symbolic", "copy"=>"edit-copy-symbolic", "paste"=>"edit-paste-symbolic", "save"=>"document-save-symbolic", "keyboard"=>"input-keyboard-symbolic",
                "left_click"|"right_click"|"middle_click"|"double_click"|"back"|"forward"=>match name.as_str(){"back"=>"go-previous-symbolic","forward"=>"go-next-symbolic",_=>"input-mouse-symbolic"},
                "display_toggle"=>"video-display-symbolic", "pan_scroll"|"pan_scroll_legacy"|"scroll"|"wheel"=>"input-tablet-symbolic", "pan_zoom"=>"zoom-fit-best-symbolic", "zoom"=>"zoom-in-symbolic", "express_menu"|"radial_menu"=>"view-more-symbolic", "app_shortcuts"=>"view-app-grid-symbolic", "keypad"=>"input-dialpad-symbolic", _=>"",
            };
            serde_json::json!({"label": menus::item_label(locale,item),"icon":if key==Shortcut::Keypad{""}else{icon},"span":if key==Shortcut::Keypad&&index==16{2}else{1},"action":item.action})
        }).collect();
        let size = usize::from(menu.size.clamp(1, 5) - 1);
        let text = usize::from(menu.text_size.clamp(1, 5) - 1);
        result.insert(key.name().into(),serde_json::json!({"kind":if matches!(key,Shortcut::ExpressMenu|Shortcut::RadialMenu){"radial"}else if key==Shortcut::Keypad{"keypad"}else{"grid"},"title":locale.t(menus::title(key)),"close":locale.t("Close"),"diameter":sizes::RADIAL_SIZES[size],"cell":sizes::CELL_SIZES[size],"text":sizes::TEXT_SIZES[text],"at_cursor":menu.at_cursor,"items":items}));
    }
    result.into()
}

pub fn backup_date(created: u64, language: &str) -> String {
    use chrono::{Datelike, Timelike};
    let Some(date) = i64::try_from(created)
        .ok()
        .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
        .map(|t| t.with_timezone(&chrono::Local))
    else {
        return String::new();
    };
    let time = format!("{}:{:02}:{:02}", date.hour(), date.minute(), date.second());
    match language {
        "en" => format!(
            "{}/{}/{}, {}:{:02}:{:02} {}",
            date.month(),
            date.day(),
            date.year(),
            (date.hour() + 11) % 12 + 1,
            date.minute(),
            date.second(),
            if date.hour() < 12 { "AM" } else { "PM" }
        ),
        "ja" | "zh-CN" | "zh-TW" => {
            format!("{}/{}/{} {time}", date.year(), date.month(), date.day())
        }
        "de" | "ru" | "pl" => format!(
            "{:02}.{:02}.{}, {time}",
            date.day(),
            date.month(),
            date.year()
        ),
        _ => format!("{}/{}/{} {time}", date.day(), date.month(), date.year()),
    }
}

// A restored/reset configuration invalidates older saves still queued by the UI.
pub(crate) static CONFIG_IO: std::sync::Mutex<()> = std::sync::Mutex::new(());
static CONFIG_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub(crate) fn config_epoch() -> u64 {
    CONFIG_EPOCH.load(std::sync::atomic::Ordering::SeqCst)
}
