//! User-level checks for editable Pen controls and scrolling.
use egui_kittest::{kittest::Queryable, Harness};
use tablet_center::classic::devices::PEN_LOWER;
fn harness(height: f32) -> Harness<'static, tablet_center::App> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., height))
        .build_ui_state(
            |ui, app: &mut tablet_center::App| app.show(ui),
            tablet_center::App::simulated(),
        );
    h.run_steps(3);
    h
}
#[test]
fn mode_and_defaults_work_without_a_daemon_or_config_writes() {
    let mut h = harness(1600.);
    h.get_by_label("Pressure to click").click();
    h.run_steps(1);
    assert_eq!(h.state().config.pen.click_threshold, 4);
    h.get_by_label("Mouse").click();
    h.run_steps(1);
    assert_eq!(h.state().config.pen.mode, tablet_config::PenMode::Mouse);
    h.get_by_label("Restore defaults").click();
    h.run_steps(1);
    assert_eq!(h.state().config.pen, tablet_config::PenSettings::default());
}
#[test]
fn compact_window_scroll_reaches_the_mode_card() {
    let mut h = harness(720.);
    h.event(egui::Event::PointerMoved(egui::pos2(1000., 650.)));
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        delta: egui::vec2(0., -650.),
        modifiers: Default::default(),
    });
    h.run_steps(20);
    assert!(h.state().pen.scroll > 250.);
    h.get_by_label("Relative mouse").click();
    h.run_steps(1);
    assert_eq!(h.state().config.pen.mode, tablet_config::PenMode::Relative);
}

#[test]
fn mapping_defaults_keep_orientation_and_touch_is_editable() {
    let mut h = harness(720.);
    h.state_mut().page = 2;
    h.state_mut().config.mapping.orientation = tablet_config::Orientation::Portrait;
    h.run_steps(2);
    h.get_all_by_label("Partial area").next().unwrap().click();
    h.run_steps(1);
    assert!(h.state().config.mapping.screen_portion.is_some());
    h.get_by_label("Restore defaults").click();
    h.run_steps(1);
    assert!(h.state().config.mapping.screen_portion.is_none());
    assert_eq!(
        h.state().config.mapping.orientation,
        tablet_config::Orientation::Portrait
    );
    h.state_mut().page = 5;
    h.run_steps(2);
    h.get_by_label("Enable touch").click();
    h.run_steps(1);
    assert!(!h.state().config.touch.enabled);
}
#[test]
fn express_key_actions_and_keyboard_capture_update_only_selected_profile() {
    let mut h = harness(720.);
    h.state_mut().config.apps.push(tablet_config::AppProfile {
        id: "krita".into(),
        name: "Krita".into(),
        ..Default::default()
    });
    h.state_mut().devices.app = Some(0);
    h.state_mut().page = 1;
    h.state_mut().devices.selected = Some(2);
    h.run_steps(3);
    // Undo is both recommended and under Keyboard; either applies it.
    h.get_all_by_label("Undo").next().unwrap().click();
    h.run_steps(1);
    assert!(matches!(
        h.state().config.apps[0].express_keys[2],
        tablet_config::KeyAction::Keystroke { .. }
    ));
    assert!(h
        .state()
        .config
        .express_keys
        .iter()
        .all(|k| *k == tablet_config::KeyAction::Disabled));
    h.get_all_by_label("Keyboard shortcut…")
        .next()
        .unwrap()
        .click();
    h.run_steps(2);
    h.event(egui::Event::Key {
        key: egui::Key::ArrowLeft,
        physical_key: Some(egui::Key::ArrowLeft),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::CTRL,
    });
    h.run_steps(1);
    h.get_by_label("OK").click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.apps[0].express_keys[2],
        tablet_config::KeyAction::Keystroke {
            keys: vec!["KEY_LEFTCTRL".into(), "KEY_LEFT".into()],
            label: "Ctrl+Left".into()
        }
    );
}
#[test]
fn sixth_express_key_action_does_not_change_pen_settings() {
    let mut h = harness(720.);
    h.state_mut().status.device.as_mut().unwrap().product = 0x0314;
    let pen = h.state().config.pen.clone();
    h.state_mut().page = 1;
    h.state_mut().devices.selected = Some(5);
    h.run_steps(3);
    h.get_all_by_label("Right click").next().unwrap().click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.express_keys[5],
        tablet_config::KeyAction::RightClick
    );
    assert_eq!(h.state().config.pen, pen);
}

#[test]
fn action_groups_open_without_closing_the_others() {
    // The on-screen shortcuts panel folds its groups; the device pages list them all.
    let mut h = harness(1600.);
    h.state_mut().tab = 1;
    h.state_mut().shortcuts.page = 1;
    h.state_mut().shortcuts.actions.selected = Some(0);
    h.run_steps(3);
    h.get_by_label("Clicks").click();
    h.run_steps(2);
    h.get_by_label("Keyboard").click();
    h.run_steps(2);
    let actions = &h.state().shortcuts.actions;
    assert!(actions.group_open(0) && actions.group_open(1) && actions.group_open(2));
    h.get_by_label("Clicks").click();
    h.run_steps(2);
    let actions = &h.state().shortcuts.actions;
    assert!(actions.group_open(0) && !actions.group_open(1) && actions.group_open(2));
}
#[test]
fn pen_action_menu_changes_the_pen_and_cancel_does_not_apply() {
    let mut h = harness(1600.);
    h.state_mut().devices.selected = Some(PEN_LOWER);
    h.run_steps(3);
    // Middle click is both recommended and under Clicks; either applies it.
    h.get_all_by_label("Middle click").next().unwrap().click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.pen.lower_button,
        tablet_config::ButtonAction::MiddleClick
    );
    h.get_all_by_label("Keyboard shortcut…")
        .next()
        .unwrap()
        .click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.pen.lower_button,
        tablet_config::ButtonAction::MiddleClick
    );
}
#[test]
fn app_picker_clones_selected_settings_and_remove_requires_confirmation() {
    let mut h = harness(720.);
    h.state_mut().devices.add_app = true;
    h.state_mut().devices.running_apps = vec![tablet_config::AppInfo {
        id: "org.kde.krita".into(),
        name: "Krita".into(),
        icon: None,
    }];
    h.state_mut().devices.app_choice = Some(0);
    h.run_steps(3);
    h.get_by_label("OK").click();
    h.run_steps(2);
    assert_eq!(h.state().config.apps[0].id, "org.kde.krita");
    assert_eq!(h.state().devices.app, Some(0));
    assert_eq!(h.state().config.apps[0].pen, h.state().config.pen);
    h.get_by_label("Remove application").click();
    h.run_steps(2);
    assert_eq!(h.state().config.apps.len(), 1);
    h.get_by_label("Delete").click();
    h.run_steps(1);
    assert!(h.state().config.apps.is_empty());
}
#[test]
fn add_application_click_keeps_the_picker_open() {
    let mut h = harness(720.);
    h.get_by_label("Add application").click();
    h.run_steps(3);
    assert!(h.state().devices.add_app);
    assert!(h.query_by_label("Cancel").is_some());
}
#[test]
fn wheel_over_the_app_picker_scrolls_the_list_not_the_page() {
    let mut h = harness(720.);
    h.get_by_label("Add application").click();
    h.state_mut().devices.running_apps = (0..30)
        .map(|i| tablet_config::AppInfo {
            id: format!("app.{i}"),
            name: format!("App {i}"),
            icon: None,
        })
        .collect();
    h.run_steps(3);
    h.event(egui::Event::PointerMoved(egui::pos2(550., 450.)));
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        delta: egui::vec2(0., -300.),
        modifiers: Default::default(),
    });
    h.run_steps(20);
    assert_eq!(h.state().pen.scroll, 0.);
    assert!(h.state().devices.app_list_scroll > 100.);
}
#[test]
fn active_app_follows_the_focused_window_like_app_js() {
    let mut c = tablet_config::Config::default();
    c.apps.push(tablet_config::AppProfile {
        id: "org.gnome.Calculator.desktop".into(),
        name: "Calculator".into(),
        icon: None,
        pen: c.pen.clone(),
        express_keys: c.express_keys.clone(),
    });
    let active = tablet_center::classic::devices::active_profile;
    assert_eq!(active(&c, Some("org.gnome.Calculator")), Some("Calculator"));
    assert_eq!(active(&c, Some("gnome-calculator.desktop")), None);
    assert_eq!(active(&c, Some("calculator")), Some("Calculator"));
    assert_eq!(active(&c, Some("firefox")), None);
    assert_eq!(active(&c, None), None);
}
#[test]
fn app_strip_scrolls_when_the_apps_do_not_fit() {
    let mut h = harness(720.);
    for i in 0..12 {
        h.state_mut().config.apps.push(tablet_config::AppProfile {
            id: format!("app.{i}"),
            name: format!("Application {i}"),
            icon: None,
            pen: Default::default(),
            express_keys: Default::default(),
        });
    }
    h.run_steps(2);
    h.get_by_label("Next").click();
    h.run_steps(2);
    assert!(h.state().devices.app_offset > 0.);
    h.get_by_label("Previous").click();
    h.run_steps(2);
    assert_eq!(h.state().devices.app_offset, 0.);
}
#[test]
fn desktop_entries_and_physical_keys_match_original_protocol() {
    assert!(tablet_center::system::parse(
        "hidden.desktop",
        "[Desktop Entry]\nType=Application\nName=Hidden\nNoDisplay=true"
    )
    .is_none());
    let app = tablet_center::system::parse(
        "krita.desktop",
        "[Desktop Entry]\nType=Application\nName=Krita\nIcon=krita\n[Other]\nName=Wrong",
    )
    .unwrap();
    assert_eq!(app.id, "krita");
    assert_eq!(app.name, "Krita");
    assert_eq!(
        tablet_center::classic::devices::evdev_key(egui::Key::PageDown),
        "KEY_PAGEDOWN"
    );
}
#[test]
fn installed_apps_follow_the_app_grid_rules() {
    use tablet_center::system::{parse_with, DesktopEnv};
    let env = DesktopEnv {
        desktops: vec!["GNOME".into()],
        langs: vec!["es_CO".into(), "es".into()],
        path: vec!["/usr/bin".into()],
    };
    let entry = |extra: &str| format!("[Desktop Entry]\nType=Application\nName=App\n{extra}");
    let shown = |extra: &str| parse_with("app.desktop", &entry(extra), &env).is_some();
    assert!(shown(""));
    assert!(!shown("NotShowIn=GNOME;Unity;"));
    assert!(!shown("OnlyShowIn=KDE;"));
    assert!(shown("OnlyShowIn=KDE;GNOME;"));
    assert!(!shown("TryExec=surely-not-installed-tablet-test"));
    assert!(!shown("Exec=\"/nonexistent/app.AppImage\" %U"));
    assert!(shown("Exec=/usr/bin/env foo"));
    let app = parse_with(
        "com.bitwarden.desktop.desktop",
        &entry("Name[es]=Aplicación"),
        &env,
    )
    .unwrap();
    assert_eq!(app.id, "com.bitwarden.desktop");
    assert_eq!(app.name, "Aplicación");
}
#[test]
fn partial_screen_area_moves_and_resizes_with_pointer() {
    let mut h = harness(720.);
    h.state_mut().page = 2;
    h.state_mut().config.mapping.screen_portion = Some(tablet_config::Rect {
        x: 0.1,
        y: 0.1,
        w: 0.8,
        h: 0.8,
    });
    h.run_steps(3);
    let drag = |h: &mut Harness<'_, tablet_center::App>, from: egui::Pos2, to: egui::Pos2| {
        h.event(egui::Event::PointerMoved(from));
        h.event(egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
        h.run_steps(1);
        h.event(egui::Event::PointerMoved(to));
        h.run_steps(1);
        h.event(egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        h.run_steps(1);
    };
    // Pages sit 72 px higher than the original 116 px header layout.
    drag(&mut h, egui::pos2(440., 178.), egui::pos2(460., 183.));
    let p = h.state().config.mapping.screen_portion.unwrap();
    assert!(p.x > 0.13);
    assert!(p.y > 0.12);
    assert_eq!(p.w, 0.8);
    let right = 253. + 452. * (p.x + p.w) as f32;
    let bottom = 109. + 145. * (p.y + p.h) as f32;
    drag(
        &mut h,
        egui::pos2(right, bottom),
        egui::pos2(right - 30., bottom - 15.),
    );
    let p = h.state().config.mapping.screen_portion.unwrap();
    assert!(p.w < 0.76);
    assert!(p.h < 0.74);
}
#[test]
fn orientation_requires_confirmation() {
    let mut h = harness(720.);
    h.state_mut().page = 6;
    h.run_steps(2);
    h.get_by_label("Portrait").click();
    h.run_steps(2);
    assert_eq!(
        h.state().config.mapping.orientation,
        tablet_config::Orientation::Landscape
    );
    h.get_by_label("OK").click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.mapping.orientation,
        tablet_config::Orientation::Portrait
    );
}
#[test]
fn parameter_dialogs_apply_modifiers_precision_and_application_command() {
    let mut h = harness(720.);
    h.state_mut().page = 1;
    h.state_mut().devices.selected = Some(1);
    h.state_mut().devices.action_dialog = Some("modifier".into());
    h.state_mut().devices.modifiers = [true, true, false, false];
    h.state_mut().devices.modifier_click = 2;
    h.run_steps(3);
    h.get_by_label("OK").click();
    h.run_steps(1);
    assert_eq!(
        h.state().config.express_keys[1],
        tablet_config::KeyAction::Modifier {
            shift: true,
            ctrl: true,
            alt: false,
            super_key: false,
            click: Some(tablet_config::Click::Right)
        }
    );
    h.state_mut().devices.action_dialog = Some("precision_mode".into());
    h.state_mut().devices.precision_scale = 6;
    h.state_mut().devices.precision_overlay = true;
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(1);
    assert_eq!(h.state().config.precision.scale, 6);
    assert!(h.state().config.precision.overlay);
    h.state_mut().devices.action_dialog = Some("run".into());
    h.state_mut().devices.command = "desktop:org.kde.krita".into();
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(1);
    assert!(
        matches!(&h.state().config.express_keys[1],tablet_config::KeyAction::Run{command,..} if command=="desktop:org.kde.krita")
    );
}

#[test]
fn shortcut_grid_edits_selected_entry_without_changing_device_actions() {
    let mut h = harness(720.);
    h.state_mut().tab = 1;
    h.state_mut().shortcuts.page = 3;
    h.run_steps(3);
    h.get_by_label("Undo").click();
    h.run_steps(3);
    assert_eq!(h.state().shortcuts.actions.selected, Some(0));
    h.get_by_label("Disabled").click();
    h.run_steps(2);
    assert_eq!(
        h.state().config.shortcuts.app_shortcuts.items[0].action,
        tablet_config::KeyAction::Disabled
    );
    assert_eq!(h.state().config.pen, tablet_config::PenSettings::default());
    h.state_mut().shortcuts.actions.selected = None;
    h.run_steps(3);
    h.get_by_label("Display at cursor").click();
    h.run_steps(2);
    assert!(!h.state().config.shortcuts.app_shortcuts.at_cursor);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    assert_eq!(
        h.state().config.shortcuts.app_shortcuts,
        tablet_config::ShortcutSettings::default().app_shortcuts
    );
}

#[test]
fn all_gesture_sliders_and_follow_toggle_update_their_own_settings() {
    let mut h = harness(1600.);
    h.state_mut().tab = 1;
    for page in 5..=10 {
        h.state_mut().shortcuts.page = page;
        h.run_steps(2);
        h.get_by_label("Speed").click();
        h.run_steps(2);
        h.event(egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: Some(egui::Key::ArrowRight),
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        });
        h.event(egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: Some(egui::Key::ArrowRight),
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        });
        h.run_steps(2);
    }
    let c = &h.state().config;
    assert_eq!(
        [
            c.pan_scroll.speed,
            c.gestures.legacy_speed,
            c.gestures.pan_zoom_speed,
            c.gestures.scroll_speed,
            c.gestures.wheel_speed,
            c.gestures.zoom_speed
        ],
        [4; 6]
    );
    h.state_mut().shortcuts.page = 5;
    h.run_steps(2);
    h.get_by_label("Content follows the pen").click();
    h.run_steps(2);
    assert!(!h.state().config.pan_scroll.content_follows_pen);
}

#[test]
fn shortcut_actions_escape_closes_and_size_keyboard_changes_preview() {
    let mut h = harness(720.);
    h.state_mut().tab = 1;
    h.state_mut().shortcuts.page = 3;
    h.run_steps(3);
    h.get_by_label("Size").click();
    h.run_steps(2);
    for pressed in [true, false] {
        h.event(egui::Event::Key {
            key: egui::Key::ArrowRight,
            physical_key: Some(egui::Key::ArrowRight),
            pressed,
            repeat: false,
            modifiers: Default::default(),
        });
    }
    h.run_steps(2);
    assert_eq!(h.state().config.shortcuts.app_shortcuts.size, 4);
    h.state_mut().shortcuts.page = 1;
    h.state_mut().shortcuts.actions.selected = Some(0);
    h.run_steps(2);
    for pressed in [true, false] {
        h.event(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: Some(egui::Key::Escape),
            pressed,
            repeat: false,
            modifiers: Default::default(),
        });
    }
    h.run_steps(2);
    assert!(h.state().shortcuts.actions.selected.is_none());
}

#[test]
fn shortcut_custom_label_and_keyboard_capture_are_saved_to_the_selected_menu() {
    let mut h = harness(1600.);
    h.state_mut().tab = 1;
    h.state_mut().shortcuts.page = 3;
    h.run_steps(3);
    h.get_by_label("Undo").click();
    h.run_steps(3);
    h.get_by_label("Name").click();
    h.run_steps(2);
    h.get_by_label("Name").type_text("Drawing undo");
    h.run_steps(2);
    assert_eq!(
        h.state().config.shortcuts.app_shortcuts.items[0].label,
        "Drawing undo"
    );
    h.get_by_label("Keyboard shortcut…").click();
    h.run_steps(2);
    h.event(egui::Event::Key {
        key: egui::Key::B,
        physical_key: Some(egui::Key::B),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    });
    h.event(egui::Event::Key {
        key: egui::Key::B,
        physical_key: Some(egui::Key::B),
        pressed: false,
        repeat: false,
        modifiers: Default::default(),
    });
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(2);
    assert!(
        matches!(&h.state().config.shortcuts.app_shortcuts.items[0].action,
        tablet_config::KeyAction::Keystroke{keys,..} if keys==&["KEY_LEFTCTRL","KEY_B"])
    );
    assert_eq!(
        h.state().config.shortcuts.express_menu,
        tablet_config::ShortcutSettings::default().express_menu
    );
}

#[test]
fn settings_language_changes_all_pages_and_cjk_without_restarting() {
    let mut h = harness(1600.);
    h.state_mut().tab = 2;
    h.run_steps(2);
    h.get_by_label("English").click();
    h.run_steps(2);
    h.get_by_label("Español").click();
    h.run_steps(2);
    assert_eq!(h.state().settings.language, "es");
    assert_eq!(h.state().locale.t("Settings"), "Ajustes");
    h.get_by_label("Español").click();
    h.run_steps(2);
    // End/Up selects Japanese through the same list used by the mouse.
    h.key_press(egui::Key::End);
    h.run_steps(1);
    for _ in 0..3 {
        h.key_press(egui::Key::ArrowUp);
        h.run_steps(1);
    }
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().settings.language, "ja");
    assert_eq!(h.state().locale.t("Settings"), "設定");
    assert!(h.state().settings.select.is_none());
    h.state_mut().tab = 0;
    h.state_mut().page = 0;
    h.run_steps(2);
    assert_ne!(h.state().locale.t("Mapping"), "Mapping");
}
#[test]
fn settings_toggles_and_confirmation_cancel_are_local_in_simulation() {
    let mut h = harness(1600.);
    h.state_mut().tab = 2;
    h.run_steps(2);
    h.get_by_label("Show shortcuts without switching apps")
        .click();
    h.run_steps(2);
    assert!(h.state().config.shortcuts.in_shell);
    h.get_by_label("Open when you sign in").click();
    h.run_steps(2);
    assert!(h.state().settings.autostart);
    // Backups share the single settings page with the general section.
    h.get_by_label("Reset all settings").click();
    h.run_steps(2);
    assert!(h.state().settings.dialog.is_some());
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    assert!(h.state().settings.dialog.is_none());
    assert!(h.state().config.shortcuts.in_shell);
    h.get_by_label("Create backup").click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.state().settings.dialog.is_none());
}

#[test]
fn compact_settings_scroll_reaches_shell_switch_and_reset_clears_config() {
    let mut h = harness(720.);
    h.state_mut().tab = 2;
    h.run_steps(2);
    h.event(egui::Event::PointerMoved(egui::pos2(900., 650.)));
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        delta: egui::vec2(0., -650.),
        modifiers: Default::default(),
    });
    h.run_steps(20);
    h.get_by_label("Show shortcuts without switching apps")
        .click();
    h.run_steps(2);
    assert!(h.state().config.shortcuts.in_shell);
    h.get_by_label("Reset all settings").click();
    h.run_steps(2);
    h.get_all_by_label("Reset all settings")
        .last()
        .unwrap()
        .click();
    h.run_steps(2);
    assert_eq!(h.state().config, tablet_config::Config::default());
    assert!(h.state().settings.dialog.is_none());
}

#[test]
fn css_hover_changes_button_sidebar_toolbar_and_window_pixels() {
    let mut app = tablet_center::App::simulated();
    app.page = 2;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 720.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(8);
    for label in ["Restore defaults", "ExpressKeys", "Settings", "Close"] {
        h.event(egui::Event::PointerGone);
        h.run_steps(12);
        let region = h.get_by_label(label).rect();
        let before = h.render().unwrap();
        h.get_by_label(label).hover();
        h.run_steps(12);
        let after = h.render().unwrap();
        let changed = (region.top().max(0.) as u32..region.bottom() as u32)
            .flat_map(|y| {
                (region.left().max(0.) as u32..region.right() as u32).map(move |x| (x, y))
            })
            .filter(|&(x, y)| before.get_pixel(x, y) != after.get_pixel(x, y))
            .count();
        assert!(changed > 10, "{label}: hover must change painted pixels");
        h.event(egui::Event::PointerGone);
        h.run_steps(12);
        let restored = h.render().unwrap();
        assert_eq!(
            before, restored,
            "{label}: pointer exit restores resting colors"
        );
    }
}

#[test]
fn pen_help_opens_on_hover_without_clicking() {
    let mut h = harness(720.);
    h.get_all_by_label("More information")
        .next()
        .unwrap()
        .hover();
    h.run_steps(12);
    assert!(h.query_by_value("Adjust the force needed to reach full pressure. Soft needs less force; Firm needs more.").is_some());
    h.event(egui::Event::PointerGone);
    h.run_steps(12);
    assert!(h.query_by_value("Adjust the force needed to reach full pressure. Soft needs less force; Firm needs more.").is_none());
}

#[test]
fn running_apps_come_from_systemd_scopes() {
    use tablet_center::system::scope_app_id;
    assert_eq!(
        scope_app_id("app-gnome-org.gnome.Calculator-774506.scope").as_deref(),
        Some("org.gnome.calculator")
    );
    assert_eq!(
        scope_app_id("app-gnome-google\\x2dchrome-686424.scope").as_deref(),
        Some("google-chrome")
    );
    assert_eq!(
        scope_app_id("app-flatpak-com.bitwarden.desktop-944310231.scope").as_deref(),
        Some("com.bitwarden.desktop")
    );
    assert_eq!(
        scope_app_id("app-com.mitchellh.ghostty.service").as_deref(),
        Some("com.mitchellh.ghostty")
    );
    assert_eq!(scope_app_id("session-2.scope"), None);
}

#[test]
fn pen_actions_panel_scrolls_to_its_last_item() {
    let mut h = harness(720.);
    h.state_mut().devices.selected = Some(PEN_LOWER);
    h.run_steps(30);
    h.event(egui::Event::PointerMoved(egui::pos2(920., 500.)));
    for _ in 0..10 {
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            phase: egui::TouchPhase::Move,
            delta: egui::vec2(0., -400.),
            modifiers: Default::default(),
        });
        h.run_steps(10);
    }
    let scroll = h.state().devices.action_scroll;
    assert!(scroll > 1000., "panel scrolled only {scroll}");
}

#[test]
fn touch_ring_changes_only_the_selected_mode() {
    use tablet_config::RingAction;
    let mut h = harness(720.);
    h.state_mut().status.device.as_mut().unwrap().product = 0x0314;
    h.state_mut().page = 8;
    let before = h.state().config.touch_ring.clone();
    h.run_steps(3);
    h.get_by_label("Mode 2").click();
    h.run_steps(3);
    h.get_by_label("Undo / redo").click();
    h.run_steps(2);
    let mut expected = before;
    expected[1] = RingAction::UndoRedo;
    assert_eq!(h.state().config.touch_ring, expected);
    assert_eq!(h.state().devices.ring_open, None);
}
