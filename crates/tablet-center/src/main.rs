//! Tablet Center: settings UI. The tablet itself is driven by
//! `tabletd`; this app edits the shared config and asks the daemon to reload.

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    os::unix::net::UnixStream,
    process::Command,
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use gtk::prelude::*;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tablet_config::{Config, KeyAction, Request, Response};

// Commands that touch the disk, the daemon or other programs are `async`,
// which Tauri runs off the main thread, so a slow answer never freezes the
// window.

/// One connection to tabletd, kept open: the UI asks for the status up to 25
/// times a second, and each new connection costs the daemon a thread.
static DAEMON: Mutex<Option<BufReader<UnixStream>>> = Mutex::new(None);

fn ask(request: &Request) -> Result<Response, String> {
    let mut daemon = DAEMON.lock().unwrap();
    // A kept connection may have died with a daemon restart: retry once.
    for _ in 0..2 {
        if daemon.is_none() {
            let stream = UnixStream::connect(tablet_config::socket_path())
                .map_err(|_| "tabletd is not running".to_string())?;
            stream.set_read_timeout(Some(Duration::from_millis(500))).map_err(|e| e.to_string())?;
            *daemon = Some(BufReader::new(stream));
        }
        match exchange(daemon.as_mut().unwrap(), request) {
            Ok(response) => return Ok(response),
            Err(_) => *daemon = None,
        }
    }
    Err("tabletd is not responding".into())
}

fn exchange(conn: &mut BufReader<UnixStream>, request: &Request) -> io::Result<Response> {
    let mut line = serde_json::to_vec(request)?;
    line.push(b'\n');
    conn.get_mut().write_all(&line)?;
    let mut reply = String::new();
    if conn.read_line(&mut reply)? == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    Ok(serde_json::from_str(&reply)?)
}

#[derive(serde::Serialize)]
struct Application {
    command: String,
    label: String,
}

#[tauri::command(async)]
fn installed_applications() -> Vec<Application> {
    fn scan(dir: &std::path::Path, base: &std::path::Path, seen: &mut std::collections::HashSet<String>, apps: &mut Vec<Application>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() { scan(&path, base, seen, apps); continue; }
            if path.extension().is_none_or(|e| e != "desktop") { continue; }
            let Ok(relative) = path.strip_prefix(base) else { continue };
            let id = relative.to_string_lossy().replace('/', "-");
            if !seen.insert(id.clone()) { continue; }
            let Ok(text) = fs::read_to_string(&path) else { continue };
            let mut fields = std::collections::HashMap::new();
            let mut desktop = false;
            for line in text.lines().map(str::trim) {
                if line.starts_with('[') { desktop = line == "[Desktop Entry]"; }
                else if desktop && !line.starts_with('#') {
                    if let Some((key, value)) = line.split_once('=') { fields.insert(key, value); }
                }
            }
            if fields.get("Type") != Some(&"Application") || ["NoDisplay", "Hidden"].iter().any(|key| fields.get(key) == Some(&"true")) { continue; }
            if let Some(name) = fields.get("Name") {
                apps.push(Application { command: format!("desktop:{id}"), label: (*name).into() });
            }
        }
    }
    let data_home = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from)
        .or_else(|| home().ok().map(|p| p.join(".local/share")));
    let data_dirs = std::env::var_os("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    let dirs = data_home.into_iter().chain(std::env::split_paths(&data_dirs));
    let mut seen = std::collections::HashSet::new();
    let mut apps = Vec::new();
    for dir in dirs { let base = dir.join("applications"); scan(&base, &base, &mut seen, &mut apps); }
    apps.sort_by_cached_key(|app| app.label.to_lowercase());
    apps
}

#[tauri::command(async)]
fn get_config() -> Config {
    tablet_config::load()
}

/// Saves the settings; returns whether the running daemon applied them.
#[tauri::command(async)]
fn save_config(config: Config) -> Result<bool, String> {
    tablet_config::save(&config).map_err(|e| e.to_string())?;
    Ok(ask(&Request::Reload).is_ok())
}

/// Menus for the GNOME Shell extension, as the UI resolves them (see
/// shellMenus in app.js); the extension reads the file when it opens one.
#[tauri::command(async)]
fn save_shell_menus(menus: serde_json::Value) -> Result<(), String> {
    let path = tablet_config::shell_menus_path();
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, menus.to_string()).and_then(|()| fs::rename(&tmp, &path)).map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn status() -> Response {
    ask(&Request::Status).unwrap_or_else(|error| Response { error: Some(error), ..Default::default() })
}

#[tauri::command(async)]
fn restart_daemon() -> Result<(), String> {
    let ok = Command::new("systemctl")
        .args(["--user", "restart", "tabletd.service"])
        .status()
        .map_err(|e| e.to_string())?
        .success();
    if ok { Ok(()) } else { Err("systemctl --user restart tabletd.service failed".into()) }
}

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| "HOME is not set".into())
}

fn autostart_path() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/autostart/tablet-center.desktop"))
}

#[tauri::command(async)]
fn autostart_enabled() -> Result<bool, String> {
    Ok(autostart_path()?.is_file())
}

#[tauri::command(async)]
fn set_autostart(enabled: bool) -> Result<(), String> {
    let path = autostart_path()?;
    if enabled {
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let binary = home()?.join(".local/bin/tablet-center");
        let binary = binary.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"")
            .replace('`', "\\`").replace('$', "\\$").replace('%', "%%");
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

fn backups_dir() -> Result<PathBuf, String> {
    Ok(home()?.join(".config/tablet-center/backups"))
}

fn backup_path(filename: &str) -> Result<PathBuf, String> {
    if !filename.ends_with(".json") || filename.starts_with('.')
        || !filename.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err("Invalid backup filename".into());
    }
    let path = backups_dir()?.join(filename);
    if fs::symlink_metadata(&path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
        return Err("Invalid backup file".into());
    }
    Ok(path)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Backup {
    title: String,
    config: Config,
}

#[derive(serde::Serialize)]
struct BackupInfo {
    filename: String,
    title: String,
    created: u64,
}

#[tauri::command(async)]
fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let dir = backups_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut backups = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        let Ok(path) = backup_path(&filename) else { continue };
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() { continue; }
        let Ok(backup) = serde_json::from_slice::<Backup>(&fs::read(path).map_err(|e| e.to_string())?) else { continue };
        let created = entry.metadata().and_then(|m| m.modified()).map_err(|e| e.to_string())?
            .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        backups.push(BackupInfo { filename, title: backup.title, created });
    }
    backups.sort_by(|a, b| b.created.cmp(&a.created).then_with(|| a.filename.cmp(&b.filename)));
    Ok(backups)
}

#[tauri::command(async)]
fn create_backup(title: String) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err("Backup title must contain 1 to 120 characters".into());
    }
    fs::create_dir_all(backups_dir()?).map_err(|e| e.to_string())?;
    let name: String = title.chars().take(60).map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
    let path = backup_path(&format!("{name}-{timestamp}.json"))?;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut file, &Backup { title: title.into(), config: get_config() }).map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn restore_backup(filename: String) -> Result<bool, String> {
    let bytes = fs::read(backup_path(&filename)?).map_err(|e| e.to_string())?;
    let backup: Backup = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    save_config(backup.config)
}

#[tauri::command(async)]
fn delete_backup(filename: String) -> Result<(), String> {
    fs::remove_file(backup_path(&filename)?).map_err(|e| e.to_string())
}

#[tauri::command(async)]
fn open_backups_folder() -> Result<(), String> {
    let dir = backups_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if Command::new("xdg-open").arg(dir).status().map_err(|e| e.to_string())?.success() {
        Ok(())
    } else {
        Err("Could not open backup folder".into())
    }
}

#[tauri::command(async)]
fn reset_config() -> Result<bool, String> {
    save_config(Config::default())
}

/// Runs an action picked from an on-screen shortcut. The window hides first
/// so focus goes back to the app underneath before the keys arrive.
#[tauri::command]
fn run_shortcut_action(window: tauri::WebviewWindow, action: KeyAction) {
    hide_shortcut(&window);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        let _ = ask(&Request::Run { action });
    });
}

#[tauri::command]
fn close_shortcut(window: tauri::WebviewWindow) {
    hide_shortcut(&window);
}

/// Whether the shortcut window is on screen, and showing which menu.
static SHOWN: Mutex<Option<String>> = Mutex::new(None);
/// The last menu asked for, for a page that loads after the request came.
static PENDING: Mutex<Option<serde_json::Value>> = Mutex::new(None);

#[tauri::command]
fn pending_shortcut() -> Option<serde_json::Value> {
    PENDING.lock().unwrap().clone()
}

fn hide_shortcut(window: &tauri::WebviewWindow) {
    *SHOWN.lock().unwrap() = None;
    let _ = window.hide();
}

/// A request from tabletd: one JSON line on stdin per button press.
#[derive(serde::Deserialize)]
struct ShowRequest {
    name: String,
    at: Option<(i32, i32)>,
    #[serde(default)]
    centered: bool,
}

/// `tablet-center --shortcut-host`: started once by tabletd and kept running,
/// so on-screen shortcuts open at once instead of after the half second a
/// new WebKit app takes. The window is created hidden and reused.
fn shortcut_host(app: &tauri::App) -> tauri::Result<()> {
    shortcut_window(app.handle())?;
    let handle = app.handle().clone();
    thread::spawn(move || {
        for line in io::stdin().lines() {
            let Ok(request) = serde_json::from_str::<ShowRequest>(&line.unwrap_or_default()) else { continue };
            let app = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                if let Err(e) = show_shortcut(&app, &request) {
                    eprintln!("tablet-center: cannot show {}: {e}", request.name);
                }
            });
        }
        // tabletd went away.
        handle.exit(0);
    });
    Ok(())
}

/// A transparent maximized window, so the menu can be drawn where the pen
/// is (Wayland lets no client place a window itself). Not full screen:
/// mutter paints full-screen windows over black.
fn shortcut_window(app: &tauri::AppHandle) -> tauri::Result<tauri::WebviewWindow> {
    if let Some(window) = app.get_webview_window("shortcut") {
        return Ok(window);
    }
    WebviewWindowBuilder::new(app, "shortcut", WebviewUrl::App("shortcut.html".into()))
        .title("Tablet Center")
        .decorations(false)
        .transparent(true)
        .background_color(tauri::window::Color(0, 0, 0, 0))
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .maximized(true)
        .visible(false)
        .build()
}

/// Shows the menu under the pen, or hides it when the same button is
/// pressed again. tabletd moves the mouse pointer under the pen first, and
/// mutter maps the window on the pointer's monitor.
fn show_shortcut(app: &tauri::AppHandle, request: &ShowRequest) -> tauri::Result<()> {
    let window = shortcut_window(app)?;
    let mut shown = SHOWN.lock().unwrap();
    if shown.as_deref() == Some(request.name.as_str()) {
        // The page zooms the menu out, then calls close_shortcut.
        *shown = None;
        return window.eval("window.hideShortcut?.()");
    }
    let monitors: Vec<gtk::gdk::Rectangle> = gtk::gdk::Display::default()
        .map(|d| (0..d.n_monitors()).filter_map(|i| d.monitor(i)).map(|m| m.geometry()).collect())
        .unwrap_or_default();
    let (x, y) = request.at.unwrap_or((i32::MIN, i32::MIN));
    let monitor = monitors
        .iter()
        .find(|g| x >= g.x() && x < g.x() + g.width() && y >= g.y() && y < g.y() + g.height());
    // The pen inside that monitor, and the monitor's size so the page can
    // subtract the top bar the maximized window leaves out. Without them it
    // centres the menu.
    let payload = serde_json::json!({
        "name": request.name,
        "at": monitor.map(|g| (x - g.x(), y - g.y())),
        "monitor": monitor.map(|g| (g.width(), g.height())),
        "centered": request.centered,
        // GNOME Shell animates the window as it maps; the page waits for
        // that to end so the menu does not move with it.
        "mapping": !window.is_visible()?,
    });
    window.eval(format!("window.showShortcut?.({payload})"))?;
    *PENDING.lock().unwrap() = Some(payload);
    if let Some(g) = monitor {
        window.set_size(tauri::LogicalSize::new(f64::from(g.width()), f64::from(g.height())))?;
    }
    window.show()?;
    window.set_focus()?;
    *shown = Some(request.name.clone());
    Ok(())
}

/// Interfaces: (id, page, draws its own title bar). Keep in step with
/// UI_STYLES in app.js.
const UI_STYLES: [(&str, &str, bool); 2] = [("classic", "index.html", false), ("glass", "glass.html", true)];

fn ui_style_path() -> PathBuf {
    tablet_config::config_path().with_file_name("interface")
}

/// Remembers the interface, so the next start opens it directly (with or
/// without the system title bar) instead of redirecting after loading.
#[tauri::command(async)]
fn set_ui_style(style: String) -> Result<(), String> {
    if !UI_STYLES.iter().any(|(id, ..)| *id == style) {
        return Err(format!("unknown interface {style}"));
    }
    fs::write(ui_style_path(), style).map_err(|e| e.to_string())
}

fn main_window(app: &tauri::App) -> tauri::Result<()> {
    let Some(config) = app.config().app.windows.iter().find(|w| w.label == "main").cloned() else {
        return Ok(());
    };
    let chosen = fs::read_to_string(ui_style_path()).unwrap_or_default();
    let (_, page, frameless) = UI_STYLES.iter().find(|(id, ..)| *id == chosen.trim()).unwrap_or(&UI_STYLES[0]);
    let mut config = config;
    config.url = tauri::WebviewUrl::App((*page).into());
    config.decorations = !frameless;
    WebviewWindowBuilder::from_config(app.handle(), &config)?.build()?;
    Ok(())
}

fn main() {
    let host = std::env::args().any(|a| a == "--shortcut-host");
    // GTK uses this as the Wayland app_id, which GNOME matches to
    // tablet-center.desktop for the window's icon and name.
    gtk::glib::set_prgname(Some("tablet-center"));
    gtk::glib::set_application_name("Tablet Center");
    tauri::Builder::default()
        .setup(move |app| {
            if host {
                shortcut_host(app)?;
            } else {
                main_window(app)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![installed_applications, get_config, save_config, save_shell_menus, status, restart_daemon,
            autostart_enabled, set_autostart, list_backups, create_backup, restore_backup,
            delete_backup, open_backups_folder, reset_config, run_shortcut_action, close_shortcut, pending_shortcut, set_ui_style])
        .build(tauri::generate_context!())
        .expect("error while running Tablet Center")
        .run(move |_, event| {
            // The host lives on with its window hidden, until tabletd exits.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                if host {
                    api.prevent_exit();
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_filenames_cannot_escape_the_backup_folder() {
        for filename in ["../config.json", "/tmp/backup.json", "nested/backup.json", "..json", "backup.txt", "backup\\name.json"] {
            assert!(backup_path(filename).is_err(), "{filename}");
        }
        assert_eq!(backup_path("My-tablet-123.json").unwrap(), backups_dir().unwrap().join("My-tablet-123.json"));
    }
}
