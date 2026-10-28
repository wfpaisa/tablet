//! Deterministic wgpu captures, using the browser preview fixtures.
use egui_kittest::Harness;
// Some Vulkan drivers cannot initialize several headless devices concurrently.
static SCREENSHOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
/// RMSE between a reference page and a capture over the page area. The
/// references have the original 116 px tab header; the app now shows only a
/// 44 px title bar with the tabs in the sidebar, so pages sit 72 px higher.
fn page_rmse(reference: &image::RgbImage, actual: &image::RgbImage) -> f64 {
    assert_eq!(reference.dimensions(), actual.dimensions());
    let (w, h) = reference.dimensions();
    let mut sum = 0.;
    let mut n = 0.;
    for y in 116..h {
        for x in 201..w {
            let a = reference.get_pixel(x, y).0;
            let b = actual.get_pixel(x, y - 72).0;
            for c in 0..3 {
                sum += (f64::from(a[c]) - f64::from(b[c])).powi(2);
            }
            n += 3.;
        }
    }
    (sum / n).sqrt() / 255.
}
#[test]
#[ignore = "requires a GPU; run explicitly to update screenshots"]
fn devices_screenshots() {
    let _guard = SCREENSHOT_LOCK.lock().unwrap();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    for (page, suffix) in [
        (0, "pen"),
        (1, "expresskeys"),
        (2, "mapping"),
        (3, "display-toggle"),
        (4, "info"),
        (5, "touch"),
        (6, ""),
    ] {
        for height in [720., 1600.] {
            let name = format!(
                "classic-dark-devices{}{}{}",
                if suffix.is_empty() { "" } else { "-" },
                suffix,
                if height > 720. { "-tall" } else { "" }
            );
            let mut app = tablet_center::App::simulated();
            app.page = page;
            let mut h = Harness::builder()
                .with_size(egui::vec2(1100., height))
                .wgpu()
                .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
            h.run_steps(8);
            h.render()
                .unwrap()
                .save(out.join(format!("{name}.png")))
                .unwrap();
            let reference =
                image::open(out.join("../../docs/reference").join(format!("{name}.png")))
                    .unwrap()
                    .to_rgb8();
            let actual = image::open(out.join(format!("{name}.png")))
                .unwrap()
                .to_rgb8();
            let rmse = page_rmse(&reference, &actual);
            println!("{name}: RMSE {rmse:.7}");
            // Lucide icons and restyled sliders differ from the reference on purpose;
            // the pen and ExpressKeys pages use callouts instead.
            let limit = if matches!(suffix, "pen" | "expresskeys" | "display-toggle" | "info") {
                0.15
            } else {
                // Responsive pages use the height freed by the smaller header.
                0.06
            };
            assert!(rmse <= limit, "{name}: RMSE {rmse} exceeds {limit}");
        }
    }
    for (name, scenario) in [
        ("classic-dark-actions-open", 0),
        ("classic-dark-add-application", 1),
        ("classic-dark-keystroke-capture", 2),
        ("classic-dark-add-application-list", 3),
        ("classic-dark-pen-actions-open", 4),
    ] {
        let mut app = tablet_center::App::simulated();
        app.page = 1;
        match scenario {
            0 => app.devices.selected = Some(0),
            4 => {
                // Upper pen button, with its actions panel on the right.
                app.page = 0;
                app.devices.selected = Some(4);
            }
            2 => {
                app.devices.selected = Some(0);
                app.devices.capture = true;
            }
            _ => {
                app.devices.add_app = true;
                if scenario == 3 {
                    app.devices.running_apps = vec![
                        tablet_config::AppInfo {
                            id: "org.kde.krita".into(),
                            name: "Krita".into(),
                            icon: None,
                        },
                        tablet_config::AppInfo {
                            id: "org.gimp.GIMP".into(),
                            name: "GNU Image Manipulation Program".into(),
                            icon: None,
                        },
                    ];
                    app.devices.app_choice = Some(0);
                }
            }
        }
        let mut h = Harness::builder()
            .with_size(egui::vec2(1100., 720.))
            .wgpu()
            .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
        h.run_steps(8);
        h.render()
            .unwrap()
            .save(out.join(format!("{name}.png")))
            .unwrap();
    }
    // Intuos Pro 2013 (no reference capture): ExpressKeys panel art.
    for (name, product) in [
        ("classic-dark-devices-expresskeys-pth451", 0x0314),
        ("classic-dark-devices-expresskeys-pth651", 0x0315),
    ] {
        let mut app = tablet_center::App::simulated();
        app.page = 1;
        if let Some(device) = app.status.device.as_mut() {
            device.product = product;
        }
        let mut h = Harness::builder()
            .with_size(egui::vec2(1100., 720.))
            .wgpu()
            .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
        h.run_steps(8);
        h.render()
            .unwrap()
            .save(out.join(format!("{name}.png")))
            .unwrap();
    }
}

#[test]
#[ignore = "requires a GPU; run explicitly to update screenshots"]
fn shortcuts_screenshots() {
    let _guard = SCREENSHOT_LOCK.lock().unwrap();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    let mut failures = Vec::new();
    for (page, (suffix, _)) in tablet_center::classic::shortcuts::PAGES.iter().enumerate() {
        for height in [720., 1600.] {
            let name = format!(
                "classic-dark-shortcuts{}{}{}",
                if suffix.is_empty() { "" } else { "-" },
                suffix,
                if height > 720. { "-tall" } else { "" }
            );
            let mut app = tablet_center::App::simulated();
            app.tab = 1;
            app.shortcuts.page = page;
            let mut h = Harness::builder()
                .with_size(egui::vec2(1100., height))
                .wgpu()
                .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
            h.run_steps(8);
            h.render()
                .unwrap()
                .save(out.join(format!("{name}.png")))
                .unwrap();
            let reference =
                image::open(out.join("../../docs/reference").join(format!("{name}.png")))
                    .unwrap()
                    .to_rgb8();
            let actual = image::open(out.join(format!("{name}.png")))
                .unwrap()
                .to_rgb8();
            let rmse = page_rmse(&reference, &actual);
            println!("{name}: RMSE {rmse:.7}");
            // Lucide icons and restyled sliders differ from the reference on purpose.
            if rmse > 0.07 {
                failures.push((name, rmse));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
#[ignore = "requires a GPU; run explicitly to update screenshots"]
fn overlay_screenshots() {
    let _guard = SCREENSHOT_LOCK.lock().unwrap();
    use tablet_center::classic::shortcuts;
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    for key in [
        tablet_config::Shortcut::ExpressMenu,
        tablet_config::Shortcut::Keypad,
        tablet_config::Shortcut::AppShortcuts,
    ] {
        let config = tablet_config::Config::default();
        let locale = tablet_center::i18n::Locale::new("en");
        let mut installed = false;
        let mut h = Harness::builder()
            .with_size(egui::vec2(1100., 720.))
            .wgpu()
            .build_ui(move |ui| {
                if !installed {
                    tablet_center::assets::install(ui.ctx());
                    installed = true;
                    ui.ctx().request_repaint();
                    return;
                }
                let size = shortcuts::menu_size(shortcuts::menu(&config.shortcuts, key), key);
                let request = tablet_center::overlay::ShowRequest {
                    name: key,
                    at: None,
                    centered: false,
                };
                let origin = tablet_center::overlay::place(
                    &request,
                    true,
                    None,
                    &[],
                    ui.ctx().content_rect().size(),
                    size,
                );
                shortcuts::draw_menu(ui, &locale, &config.shortcuts, key, origin, None, false);
            });
        h.run_steps(8);
        let name = format!("overlay-{}", key.name().replace('_', "-"));
        h.render()
            .unwrap()
            .save(out.join(format!("{name}.png")))
            .unwrap();
    }
    let mut app = tablet_center::App::simulated();
    app.tab = 1;
    app.shortcuts.page = 3;
    app.shortcuts.actions.selected = Some(0);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 1600.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(8);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-shortcuts-actions-open.png"))
        .unwrap();
}

#[test]
#[ignore = "requires a GPU; run explicitly to update screenshots"]
fn settings_screenshots() {
    let _guard = SCREENSHOT_LOCK.lock().unwrap();
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    // One sectioned page: the tabbed HTML references no longer apply, so
    // these captures are reviewed by eye.
    for height in [720., 1600.] {
        let mut app = tablet_center::App::simulated();
        app.tab = 2;
        let mut h = Harness::builder()
            .with_size(egui::vec2(1100., height))
            .wgpu()
            .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
        h.run_steps(8);
        h.render()
            .unwrap()
            .save(out.join(format!(
                "classic-dark-settings{}.png",
                if height > 720. { "-tall" } else { "" }
            )))
            .unwrap();
    }
    for suffix in [
        "language-open",
        "toast",
        "reset-confirmation",
        "delete-confirmation",
        "create-dialog",
        "general-es",
        "general-ja",
    ] {
        for height in if suffix.starts_with("general-") {
            vec![720., 1600.]
        } else {
            vec![720.]
        } {
            let mut app = tablet_center::App::simulated();
            app.tab = 2;
            match suffix {
                "language-open" => app.settings.select = Some(0),
                "create-dialog" => {
                    app.settings.dialog = Some(tablet_center::classic::settings::Dialog::Create);
                }
                "toast" => app.settings.notify("Driver restarted".into()),
                "reset-confirmation" => {
                    app.settings.dialog = Some(tablet_center::classic::settings::Dialog::Reset);
                }
                "delete-confirmation" => {
                    app.settings.dialog = Some(tablet_center::classic::settings::Dialog::Delete(
                        "test.json".into(),
                    ));
                }
                _ => {
                    let language = if suffix.ends_with("es") { "es" } else { "ja" };
                    app.locale = tablet_center::i18n::Locale::new(language);
                    app.settings.language = language.into();
                    // A row checks that longer action labels stay in the card.
                    app.settings.backups = vec![tablet_center::system::BackupInfo {
                        filename: "test.json".into(),
                        title: "Before the update".into(),
                        created: 1_790_000_000,
                    }];
                }
            }
            let mut h = Harness::builder()
                .with_size(egui::vec2(1100., height))
                .wgpu()
                .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
            h.run_steps(8);
            h.render()
                .unwrap()
                .save(out.join(format!(
                    "classic-dark-settings-{suffix}{}.png",
                    if height > 720. { "-tall" } else { "" }
                )))
                .unwrap();
        }
    }
}

#[test]
#[ignore = "requires a GPU; run explicitly to update screenshots"]
fn spanish_screenshots() {
    let _guard = SCREENSHOT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/screens");
    std::fs::create_dir_all(&out).unwrap();
    let mut pages = vec![
        (0, 0, 0, "devices-pen"),
        (0, 1, 0, "devices-expresskeys"),
        (0, 2, 0, "devices-mapping"),
        (0, 3, 0, "devices-display-toggle"),
        (0, 4, 0, "devices-info"),
        (0, 6, 0, "devices"),
        (2, 0, 0, "settings"),
    ];
    for (page, (suffix, _)) in tablet_center::classic::shortcuts::PAGES.iter().enumerate() {
        pages.push((
            1,
            0,
            page,
            if suffix.is_empty() {
                "shortcuts"
            } else {
                suffix
            },
        ));
    }
    for (tab, page, shortcut, suffix) in pages {
        let mut app = tablet_center::App::simulated();
        app.locale = tablet_center::i18n::Locale::new("es");
        app.settings.language = "es".into();
        app.tab = tab;
        app.page = page;
        app.shortcuts.page = shortcut;
        let mut h = Harness::builder()
            .with_size(egui::vec2(1100., 720.))
            .wgpu()
            .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
        h.run_steps(8);
        h.render()
            .unwrap()
            .save(out.join(format!("classic-dark-es-{suffix}.png")))
            .unwrap();
    }
    // Mouse mode hint uses a translated key, including below the fold.
    let mut app = tablet_center::App::simulated();
    app.locale = tablet_center::i18n::Locale::new("es");
    app.settings.language = "es".into();
    app.config.pen.mode = tablet_config::PenMode::Mouse;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 1600.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(8);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-es-devices-pen-mouse-tall.png"))
        .unwrap();
    // Pen actions panel with names long enough to wrap to two lines.
    let mut app = tablet_center::App::simulated();
    app.locale = tablet_center::i18n::Locale::new("es");
    app.settings.language = "es".into();
    app.config.pen.lower_button = tablet_config::ButtonAction::RightOrHoldMiddle;
    app.devices.selected = Some(5);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 720.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(30);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-es-pen-actions-open.png"))
        .unwrap();
    // Open/Run dialog with an installed app chosen.
    let mut app = tablet_center::App::simulated();
    app.locale = tablet_center::i18n::Locale::new("es");
    app.settings.language = "es".into();
    app.devices.action_dialog = Some("run".into());
    app.devices.installed_apps = [
        "Blender",
        "Calculadora",
        "GIMP",
        "Inkscape",
        "Krita",
        "Xournal++",
    ]
    .iter()
    .map(|name| tablet_config::AppInfo {
        id: format!("org.example.{name}"),
        name: (*name).into(),
        icon: None,
    })
    .collect();
    app.devices.command = "desktop:org.example.GIMP".into();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 720.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(8);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-es-run-dialog.png"))
        .unwrap();
    // Display toggle help, over the pen page with its actions panel open.
    let mut app = tablet_center::App::simulated();
    app.locale = tablet_center::i18n::Locale::new("es");
    app.settings.language = "es".into();
    app.devices.selected = Some(5);
    app.devices.toggle_help = true;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 720.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(30);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-es-toggle-help.png"))
        .unwrap();
    // App strip that overflows: arrows, the active badge and a hovered remove button.
    let mut app = tablet_center::App::simulated();
    app.locale = tablet_center::i18n::Locale::new("es");
    for name in [
        "Calculator",
        "Krita",
        "GIMP",
        "Inkscape",
        "Blender",
        "Xournal++",
    ] {
        app.config.apps.push(tablet_config::AppProfile {
            id: format!("org.example.{name}"),
            name: name.into(),
            icon: None,
            pen: Default::default(),
            express_keys: Default::default(),
        });
    }
    app.devices.app = Some(1);
    app.status.focused_app = Some("org.example.Calculator".into());
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 720.))
        .wgpu()
        .build_ui_state(|ui, app: &mut tablet_center::App| app.show(ui), app);
    h.run_steps(4);
    h.event(egui::Event::PointerMoved(egui::pos2(720., 165.)));
    h.run_steps(8);
    h.render()
        .unwrap()
        .save(out.join("classic-dark-es-app-strip.png"))
        .unwrap();
}
