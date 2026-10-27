//! Desktop commands run only in a subprocess with an isolated HOME and fake tools.
use std::{
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    process::Command,
};
use tablet_center::system;
#[test]
fn settings_files_and_commands_are_compatible() {
    let temp = tempfile::Builder::new()
        .prefix("tablet-settings %$")
        .tempdir()
        .unwrap();
    let home = temp.path().join("home");
    let config = temp.path().join("xdg");
    let tools = temp.path().join("bin");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&tools).unwrap();
    for name in ["systemctl", "xdg-open"] {
        let path = tools.join(name);
        fs::write(&path,"#!/bin/sh\nprintf '%s\\n' \"$0\" \"$@\" >> \"$HOME/commands\"\n[ ! -f \"$HOME/fail-command\" ]\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "isolated_settings_commands", "--nocapture"])
        .env("TABLET_SETTINGS_CHILD", "1")
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_RUNTIME_DIR", temp.path())
        .env("PATH", format!("{}:/usr/bin:/bin", tools.display()))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn isolated_settings_commands() {
    if std::env::var_os("TABLET_SETTINGS_CHILD").is_none() {
        return;
    }
    let home = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
    assert!(!system::autostart_enabled().unwrap());
    system::set_autostart(true).unwrap();
    assert!(system::autostart_enabled().unwrap());
    let desktop = fs::read_to_string(home.join(".config/autostart/tablet-center.desktop")).unwrap();
    assert!(desktop.contains("%%\\$"));
    assert!(desktop.contains("/.local/bin/tablet-center\""));
    system::set_autostart(false).unwrap();
    system::set_autostart(false).unwrap();
    assert!(!system::autostart_enabled().unwrap());
    assert!(system::list_backups().unwrap().is_empty());
    assert!(system::create_backup(" ".into()).is_err());
    assert!(system::create_backup("あ".repeat(121)).is_err());
    let mut config = tablet_config::Config::default();
    config.pen.click_threshold = 42;
    assert!(!system::save_config(config.clone()).unwrap());
    system::create_backup("  Drawing 日本語  ".into()).unwrap();
    let backups = system::list_backups().unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].title, "Drawing 日本語");
    let filename = backups[0].filename.clone();
    let path = home.join(".config/tablet-center/backups").join(&filename);
    let data: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(data["title"], "Drawing 日本語");
    assert_eq!(data["config"]["pen"]["click_threshold"], 42);
    assert!(!system::reset_config().unwrap());
    assert_eq!(tablet_config::load(), tablet_config::Config::default());
    assert!(!system::restore_backup(filename.clone()).unwrap());
    assert_eq!(tablet_config::load(), config);
    for name in [
        "../config.json",
        "/tmp/backup.json",
        "nested/a.json",
        "..json",
        "bad.txt",
        "bad\\file.json",
    ] {
        assert!(system::restore_backup(name.into()).is_err());
        assert!(system::delete_backup(name.into()).is_err());
    }
    symlink(
        tablet_config::config_path(),
        system::backups_dir().unwrap().join("link.json"),
    )
    .unwrap();
    assert!(system::restore_backup("link.json".into()).is_err());
    assert!(system::delete_backup("link.json".into()).is_err());
    fs::write(system::backups_dir().unwrap().join("invalid.json"), "oops").unwrap();
    assert_eq!(system::list_backups().unwrap().len(), 1);
    let menus = system::shell_menus(&config, &tablet_center::i18n::Locale::new("es"));
    system::save_shell_menus(menus.clone()).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &fs::read(tablet_config::shell_menus_path()).unwrap()
        )
        .unwrap(),
        menus
    );
    assert_eq!(menus["keypad"]["items"][16]["span"], 2);
    assert_eq!(menus["keypad"]["items"][0]["icon"], "");
    assert_eq!(menus["express_menu"]["items"].as_array().unwrap().len(), 8);
    assert_eq!(
        menus["express_menu"]["items"][0]["icon"],
        "edit-undo-symbolic"
    );
    assert_ne!(menus["express_menu"]["title"], "Express menu");
    system::open_backups_folder().unwrap();
    system::restart_daemon().unwrap();
    let commands = fs::read_to_string(home.join("commands")).unwrap();
    assert!(commands.contains("--user\nrestart\ntabletd.service\n"));
    assert!(commands.contains(system::backups_dir().unwrap().to_str().unwrap()));
    fs::write(home.join("fail-command"), "").unwrap();
    assert!(system::restart_daemon().is_err());
    assert!(system::open_backups_folder().is_err());
    system::delete_backup(filename).unwrap();
    assert!(system::list_backups().unwrap().is_empty());
    native_settings_ui_round_trip();
}

fn native_settings_ui_round_trip() {
    use egui_kittest::{kittest::Queryable, Harness};
    let mut app = tablet_center::App::simulated();
    app.tab = 2;
    app.settings.native = true;
    // Tall enough to show the backups section without scrolling.
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 1600.))
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    fn settle(h: &mut Harness<'_, tablet_center::App>) {
        for _ in 0..200 {
            h.run_steps(1);
            if !h.state().settings.busy {
                h.run_steps(2);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        panic!("settings job did not finish");
    }
    h.run_steps(3);
    settle(&mut h);
    h.state_mut().config.pen.click_threshold = 61;
    h.get_by_label("Create backup").click();
    h.run_steps(2);
    h.state_mut().settings.title = "From UI".into();
    h.run_steps(1);
    h.get_by_label("Create").click();
    h.run_steps(1);
    settle(&mut h);
    assert_eq!(h.state().settings.backups.len(), 1);
    h.get_by_label("Reset all settings").click();
    h.run_steps(2);
    h.get_all_by_label("Reset all settings")
        .last()
        .unwrap()
        .click();
    h.run_steps(1);
    settle(&mut h);
    assert_eq!(h.state().config, tablet_config::Config::default());
    h.get_by_label("Restore").click();
    h.run_steps(2);
    h.get_all_by_label("Restore").last().unwrap().click();
    h.run_steps(1);
    settle(&mut h);
    assert_eq!(h.state().config.pen.click_threshold, 61);
    h.get_by_label("Delete").click();
    h.run_steps(2);
    h.get_all_by_label("Delete").last().unwrap().click();
    h.run_steps(1);
    settle(&mut h);
    assert!(h.state().settings.backups.is_empty());
}
