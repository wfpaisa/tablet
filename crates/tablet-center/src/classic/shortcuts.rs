//! Shortcut settings, shared radial/grid rendering and gesture illustrations.
use super::devices;
use crate::theme::{devices_layout as d, pen_layout as p, shortcuts_layout as s};
use crate::{
    i18n::Locale,
    theme as t,
    widgets::{click, rect, text},
};
use egui::{pos2, vec2, Align2, Rect, Stroke, StrokeKind, Ui};
use tablet_config::{Config, KeyAction, MenuItem, OnScreen, Shortcut, ShortcutSettings};
pub const PAGES: [(&str, &str); 11] = [
    ("", "Start"),
    ("express-menu", "Express menu"),
    ("radial-menu", "Radial menu"),
    ("app-shortcuts", "App shortcuts"),
    ("keypad", "Keypad"),
    ("pan-scroll", "Pan/Scroll"),
    ("pan-scroll-legacy", "Pan/Scroll (Legacy)"),
    ("pan-zoom", "Pan/Zoom"),
    ("scroll", "Scroll"),
    ("wheel", "Wheel"),
    ("zoom", "Zoom"),
];
#[derive(Default)]
pub struct State {
    pub page: usize,
    pub actions: devices::State,
    pub scroll: f32,
    pub sidebar_scroll: f32,
}
pub fn menu(settings: &ShortcutSettings, key: Shortcut) -> &OnScreen {
    match key {
        Shortcut::ExpressMenu => &settings.express_menu,
        Shortcut::RadialMenu => &settings.radial_menu,
        Shortcut::AppShortcuts => &settings.app_shortcuts,
        Shortcut::Keypad => &settings.keypad,
    }
}
pub fn menu_mut(settings: &mut ShortcutSettings, key: Shortcut) -> &mut OnScreen {
    match key {
        Shortcut::ExpressMenu => &mut settings.express_menu,
        Shortcut::RadialMenu => &mut settings.radial_menu,
        Shortcut::AppShortcuts => &mut settings.app_shortcuts,
        Shortcut::Keypad => &mut settings.keypad,
    }
}
pub fn key(page: usize) -> Option<Shortcut> {
    [
        Shortcut::ExpressMenu,
        Shortcut::RadialMenu,
        Shortcut::AppShortcuts,
        Shortcut::Keypad,
    ]
    .get(page.checked_sub(1)?)
    .copied()
}
pub fn title(key: Shortcut) -> &'static str {
    match key {
        Shortcut::ExpressMenu => "Express menu",
        Shortcut::RadialMenu => "Radial menu",
        Shortcut::AppShortcuts => "App shortcuts",
        Shortcut::Keypad => "Keypad",
    }
}
fn icon(ui: &Ui, name: &str, r: Rect, color: egui::Color32) {
    let source = match name {
        "home" => crate::icons::source("house"),
        "gesture" => crate::icons::source("move"),
        "close" => crate::icons::source("x"),
        _ => devices::action_icon(name),
    };
    egui::Image::new(source).tint(color).paint_at(ui, r);
}
/// Shortcut pages under the sidebar switch, scrolling above the footer.
pub fn sidebar(ui: &mut Ui, l: &Locale, state: &mut State) {
    let height = crate::widgets::view(ui).bottom() - crate::theme::sidebar_layout::FOOTER;
    let clip = rect(d::PX_1, t::HEADER, t::SIDEBAR - d::PX_1, height - t::HEADER);
    if crate::widgets::contains_pointer(ui, clip) {
        state.sidebar_scroll = (state.sidebar_scroll - crate::widgets::page_wheel(ui))
            .clamp(0., (s::SIDEBAR_CONTENT_HEIGHT - height).max(0.));
    }
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(clip));
    child.set_clip_rect(clip);
    let ui = &mut child;
    for (i, (_, label)) in PAGES.iter().enumerate() {
        if let Some(group) = match i {
            1 => Some("Radial menus"),
            3 => Some("Grid panels"),
            5 => Some("Pen gestures"),
            _ => None,
        } {
            let top = match i {
                1 => d::PX_180,
                3 => d::PX_301,
                _ => d::PX_422,
            } - state.sidebar_scroll;
            ui.painter().add(egui::Shape::line(
                vec![
                    pos2(d::PX_22, top + d::PX_10),
                    pos2(d::PX_24, top + d::PX_12),
                    pos2(d::PX_26, top + d::PX_10),
                ],
                Stroke::new(t::BORDER, t::MUTED),
            ));
            text(ui, d::PX_38, top, &l.t(group), d::PX_15, t::TEXT);
        }
        let y = [
            d::PX_126,
            d::PX_206,
            d::PX_248,
            d::PX_327,
            d::PX_369,
            d::PX_448,
            d::PX_490,
            d::PX_552,
            d::PX_594,
            d::PX_636,
            d::PX_678,
        ][i] - state.sidebar_scroll;
        let height = if i == 6 { d::PX_60 } else { d::PX_40 };
        let r = rect(d::PX_13, y, d::PX_175, height);
        let response = click(ui, r, ("shortcut-page", i), label);
        if i != state.page && response.hovered() {
            ui.painter()
                .rect_filled(r, t::RADIUS, t::interaction::WHITE_6);
        }
        if i == state.page {
            ui.painter().rect_filled(r, t::RADIUS, t::ACCENT);
        }
        let name = match i {
            0 => "home",
            1 | 2 => "express_menu",
            3 => "app_shortcuts",
            4 => "keypad",
            _ => "gesture",
        };
        icon(
            ui,
            name,
            rect(
                if i == 0 { d::PX_25 } else { d::PX_39 },
                y + (height - d::PX_22) / d::PX_2,
                d::PX_22,
                d::PX_22,
            ),
            t::TEXT,
        );
        let left = if i == 0 { d::PX_59 } else { d::PX_73 };
        let size = if i == 0 { d::PX_16 } else { d::PX_15 };
        let width = r.right() - left - t::sidebar_layout::LABEL_PAD;
        let label = l.t(label);
        let label = if i == 6 {
            label.replace(" (", "\n(")
        } else {
            label
        };
        if label.contains('\n')
            && label
                .lines()
                .all(|line| crate::widgets::text_width(ui, line, size) <= width)
        {
            for (line_index, line) in label.lines().enumerate() {
                text(
                    ui,
                    left,
                    y + d::PX_10 + line_index as f32 * d::PX_21,
                    line,
                    size,
                    t::TEXT,
                );
            }
        } else {
            crate::widgets::fit_text(
                ui,
                pos2(left, y + d::PX_10),
                &label,
                size,
                width,
                r,
                t::TEXT,
            );
        }
        if response.clicked() {
            state.page = i;
            state.actions.selected = None;
            state.scroll = 0.;
        }
    }
}
fn centered(ui: &Ui, center: egui::Pos2, label: &str, size: f32, color: egui::Color32) {
    ui.painter()
        .text(center, Align2::CENTER_TOP, label, t::font(size), color);
}
fn paragraph(ui: &Ui, r: Rect, label: &str, size: f32, color: egui::Color32) {
    let mut job = egui::text::LayoutJob::simple(label.into(), t::font(size), color, r.width());
    job.halign = egui::Align::Center;
    for section in &mut job.sections {
        section.format.line_height = Some(size * d::PX_1_45);
    }
    let g = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(pos2(r.center().x, r.top()), g, color);
}
pub fn items(settings: &ShortcutSettings, key: Shortcut) -> Vec<MenuItem> {
    if key == Shortcut::Keypad {
        return [
            ("Esc", "KEY_ESC"),
            ("Tab", "KEY_TAB"),
            ("⌫", "KEY_BACKSPACE"),
            ("/", "KEY_KPSLASH"),
            ("7", "KEY_7"),
            ("8", "KEY_8"),
            ("9", "KEY_9"),
            ("*", "KEY_KPASTERISK"),
            ("4", "KEY_4"),
            ("5", "KEY_5"),
            ("6", "KEY_6"),
            ("−", "KEY_KPMINUS"),
            ("1", "KEY_1"),
            ("2", "KEY_2"),
            ("3", "KEY_3"),
            ("+", "KEY_KPPLUS"),
            ("0", "KEY_0"),
            (".", "KEY_DOT"),
            ("Enter", "KEY_ENTER"),
        ]
        .into_iter()
        .map(|(label, key)| MenuItem {
            label: label.into(),
            action: KeyAction::Keystroke {
                label: label.into(),
                keys: vec![key.into()],
            },
        })
        .collect();
    }
    let m = menu(settings, key);
    (0..if key == Shortcut::AppShortcuts { 12 } else { 8 })
        .map(|i| m.items.get(i).cloned().unwrap_or_default())
        .collect()
}
pub fn item_label(l: &Locale, item: &MenuItem) -> String {
    if !item.label.is_empty() {
        return item.label.clone();
    }
    match &item.action {
        KeyAction::Keystroke { keys, label } => {
            if label.is_empty() {
                keys.join("+")
            } else {
                l.t(label)
            }
        }
        KeyAction::Run { .. } => l.t("Open/Run..."),
        KeyAction::Modifier { .. } => l.t("Keyboard modifier..."),
        KeyAction::PrecisionMode => l.t("Precision mode..."),
        KeyAction::ModeToggle => l.t("Mode toggle..."),
        _ => l.t(&devices::action_label(&item.action)),
    }
}
pub fn item_icon(item: &MenuItem) -> String {
    match &item.action {
        KeyAction::Keystroke { keys, .. } => {
            let presets = [
                ("undo", vec!["KEY_LEFTCTRL", "KEY_Z"]),
                ("redo", vec!["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_Z"]),
                ("copy", vec!["KEY_LEFTCTRL", "KEY_C"]),
                ("paste", vec!["KEY_LEFTCTRL", "KEY_V"]),
                ("save", vec!["KEY_LEFTCTRL", "KEY_S"]),
            ];
            presets
                .into_iter()
                .find(|(_, preset)| keys == preset)
                .map(|(name, _)| name.to_string())
                .unwrap_or("keyboard".into())
        }
        _ => serde_json::to_value(&item.action)
            .ok()
            .and_then(|v| v["type"].as_str().map(String::from))
            .unwrap_or("disabled".into()),
    }
}
fn step(value: u8) -> usize {
    usize::from(value.clamp(1, 5) - 1)
}
pub fn menu_size(m: &OnScreen, key: Shortcut) -> egui::Vec2 {
    if matches!(key, Shortcut::ExpressMenu | Shortcut::RadialMenu) {
        return vec2(s::RADIAL_SIZES[step(m.size)], s::RADIAL_SIZES[step(m.size)]);
    }
    let cell = s::CELL_SIZES[step(m.size)];
    vec2(
        cell * d::PX_4 + s::CELL_GAP * d::PX_3 + s::GRID_PAD * d::PX_2,
        cell * if key == Shortcut::Keypad {
            d::PX_5
        } else {
            d::PX_3
        } + s::CELL_GAP
            * if key == Shortcut::Keypad {
                d::PX_4
            } else {
                d::PX_2
            }
            + s::GRID_HEADER
            + s::GRID_BOTTOM,
    )
}
#[derive(Default)]
pub struct MenuResponse {
    pub item: Option<usize>,
    pub close: bool,
}
pub fn draw_menu(
    ui: &mut Ui,
    l: &Locale,
    settings: &ShortcutSettings,
    key: Shortcut,
    origin: egui::Pos2,
    selected: Option<usize>,
    preview: bool,
) -> MenuResponse {
    let m = menu(settings, key);
    let size = menu_size(m, key);
    let font = s::TEXT_SIZES[step(m.text_size)];
    let entries = items(settings, key);
    let mut response = MenuResponse::default();
    if matches!(key, Shortcut::ExpressMenu | Shortcut::RadialMenu) {
        let r = Rect::from_min_size(origin, size);
        let center = r.center();
        let hovered = crate::widgets::pointer(ui).and_then(|p| radial_item(p - center, size.x));
        // SVG paths preserve the browser's annular separators and drop shadow.
        let mut paths = String::new();
        for i in 0..8 {
            let angle = |j: f32| j / 8. * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let at =
                |radius: f32, a: f32| format!("{:.2} {:.2}", radius * a.cos(), radius * a.sin());
            let a0 = angle(i as f32 - 0.5);
            let a1 = angle(i as f32 + 0.5);
            let fill = if selected == Some(i) {
                t::ACCENT
            } else if hovered == Some(i) {
                s::HOVER
            } else {
                t::RAISED
            };
            paths.push_str(&format!(
                r#"<path fill="{}" d="M{} L{} A100 100 0 0 1 {} L{} A34 34 0 0 0 {} Z"/>"#,
                fill.to_hex(),
                at(34., a0),
                at(100., a0),
                at(100., a1),
                at(34., a1),
                at(34., a0)
            ));
        }
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-140 -140 280 280"><defs><filter id="s" x="-50%" y="-50%" width="200%" height="200%"><feDropShadow dx="0" dy="4.8" stdDeviation="14.4" flood-opacity=".45"/></filter></defs><g filter="url(#s)" stroke="{}" stroke-width="1.2">{}<circle r="30" fill="{}"/></g></svg>"#,
            t::BG.to_hex(),
            paths,
            t::PANEL.to_hex()
        );
        let expanded = size.x * 280. / s::RADIAL_UNITS;
        egui::Image::new(egui::ImageSource::Bytes {
            uri: format!(
                "bytes://radial-{}-{:?}-{:?}-{}.svg",
                key.name(),
                selected,
                hovered,
                m.size
            )
            .into(),
            bytes: svg.into_bytes().into(),
        })
        .paint_at(ui, Rect::from_center_size(center, vec2(expanded, expanded)));
        let hit = click(ui, r, ("menu", key.name()), title(key));
        let hub_hovered = hit.hovered()
            && crate::widgets::pointer(ui)
                .is_some_and(|p| p.distance(center) < size.x * 30. / s::RADIAL_UNITS);
        if hub_hovered {
            ui.painter().circle_filled(
                center,
                size.x * 30. / s::RADIAL_UNITS,
                t::interaction::WHITE_8,
            );
        }
        if let Some(p) = hit.interact_pointer_pos().filter(|_| hit.clicked()) {
            let delta = p - center;
            let radius = delta.length() / size.x * s::RADIAL_UNITS;
            if radius < 30. {
                response.close = true;
            } else {
                response.item = radial_item(delta, size.x);
            }
        }
        for (i, item) in entries.iter().enumerate() {
            let a = i as f32 / 8. * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
            let c = center
                + vec2(a.cos(), a.sin()) * size.x * s::LABEL_RADIUS / s::RADIAL_UNITS
                + if preview {
                    vec2(0., d::PX_2 + a.sin() * d::PX_1_3)
                } else {
                    egui::Vec2::ZERO
                };
            label(ui, l, item, c, size.x * d::PX_0_22, font, key);
        }
        icon(
            ui,
            "close",
            Rect::from_center_size(center, size * d::PX_0_075),
            if hub_hovered { t::TEXT } else { t::MUTED },
        );
    } else {
        let r = Rect::from_min_size(origin, size);
        ui.painter().add(
            egui::epaint::Shadow {
                offset: [0, 10],
                blur: 60,
                spread: 0,
                color: s::SHADOW,
            }
            .as_shape(r, t::CARD_RADIUS),
        );
        ui.painter().rect(
            r,
            t::CARD_RADIUS,
            t::PANEL,
            Stroke::new(t::BORDER, t::LINE),
            StrokeKind::Inside,
        );
        text(
            ui,
            origin.x + d::PX_15,
            origin.y + d::PX_16,
            &l.t(title(key)),
            font,
            t::MUTED,
        );
        let close = rect(
            r.right() - d::PX_41,
            origin.y + d::PX_11,
            d::PX_26,
            d::PX_26,
        );
        let close_response = click(ui, close, ("close-menu", key.name()), &l.t("Close"));
        if close_response.hovered() {
            ui.painter()
                .rect_filled(close, t::FULL_RADIUS, t::interaction::WHITE_8);
        }
        icon(
            ui,
            "close",
            Rect::from_center_size(close.center(), vec2(d::PX_14, d::PX_14)),
            crate::widgets::hover_color(&close_response, t::MUTED, t::TEXT),
        );
        response.close = close_response.clicked();
        let cell = s::CELL_SIZES[step(m.size)];
        for (i, item) in entries.iter().enumerate() {
            let (col, row, span) = if key == Shortcut::Keypad && i >= 16 {
                match i {
                    16 => (0, 4, 2),
                    17 => (2, 4, 1),
                    _ => (3, 4, 1),
                }
            } else {
                (i % 4, i / 4, 1)
            };
            let r = rect(
                origin.x + s::GRID_PAD + col as f32 * (cell + s::CELL_GAP),
                origin.y + s::GRID_HEADER + row as f32 * (cell + s::CELL_GAP),
                cell * span as f32 + s::CELL_GAP * (span - 1) as f32,
                cell,
            );
            let hit = click(ui, r, ("menu-item", key.name(), i), &item_label(l, item));
            ui.painter().rect_filled(
                r,
                t::RADIUS,
                if selected == Some(i) {
                    t::ACCENT
                } else if hit.hovered() {
                    s::HOVER
                } else {
                    t::RAISED
                },
            );
            label(ui, l, item, r.center(), r.width() - d::PX_8, font, key);
            if hit.clicked() {
                response.item = Some(i);
            }
        }
    }
    response
}
fn radial_item(delta: egui::Vec2, diameter: f32) -> Option<usize> {
    let radius = delta.length() / diameter * s::RADIAL_UNITS;
    if !(34. ..=100.).contains(&radius) {
        return None;
    }
    Some(
        ((delta.angle()
            + std::f32::consts::FRAC_PI_2
            + std::f32::consts::TAU
            + std::f32::consts::PI / 8.)
            / std::f32::consts::TAU
            * 8.) as usize
            % 8,
    )
}

fn label(
    ui: &Ui,
    l: &Locale,
    item: &MenuItem,
    c: egui::Pos2,
    width: f32,
    font: f32,
    key: Shortcut,
) {
    let keypad = key == Shortcut::Keypad;
    let radial = matches!(key, Shortcut::ExpressMenu | Shortcut::RadialMenu);
    let color = if item.action == KeyAction::Disabled {
        t::FAINT
    } else {
        t::TEXT
    };
    let label = item_label(l, item);
    if keypad {
        if label == "⌫" {
            let w = font * d::PX_0_9;
            let h = font * d::PX_0_5;
            ui.painter().add(egui::Shape::closed_line(
                vec![
                    c + vec2(-w, 0.),
                    c + vec2(-w + d::PX_6, -h),
                    c + vec2(w, -h),
                    c + vec2(w, h),
                    c + vec2(-w + d::PX_6, h),
                ],
                Stroke::new(t::BORDER, color),
            ));
            for sign in [-1., 1.] {
                ui.painter().line_segment(
                    [c + vec2(-h, sign * h), c + vec2(h, -sign * h)],
                    Stroke::new(t::BORDER, color),
                );
            }
        } else {
            ui.painter().text(
                c - vec2(0., d::PX_1),
                Align2::CENTER_CENTER,
                label,
                t::font(font * d::PX_1_5),
                color,
            );
        }
        return;
    }
    let mut job = egui::text::LayoutJob::simple(label, t::font(font), color, width);
    job.halign = egui::Align::Center;
    for section in &mut job.sections {
        section.format.line_height = Some(font * d::PX_1_15);
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let icon_size = font * d::PX_1_9;
    let top = c.y - (icon_size + d::PX_4 + galley.size().y) / d::PX_2;
    icon(
        ui,
        &item_icon(item),
        rect(
            c.x - icon_size / d::PX_2,
            top + if radial { -d::PX_2 } else { d::PX_1 },
            icon_size,
            icon_size,
        ),
        color,
    );
    ui.painter().galley(
        pos2(
            c.x,
            top + icon_size + d::PX_4 - if radial { d::PX_3 } else { d::PX_1 },
        ),
        galley,
        color,
    );
}
pub fn show(ui: &mut Ui, l: &Locale, c: &mut Config, state: &mut State) {
    if state.page == 0 {
        start(ui, l, state);
        return;
    }
    let bounds = crate::widgets::view(ui);
    // Entry actions open in the right panel, as on the pen and ExpressKeys
    // pages; the preview and its options move over to make room.
    state.actions.panel_right = true;
    state.actions.panel_keys = false;
    let slide = super::pen::panel_progress(ui.ctx(), state.actions.selected.is_some());
    let left = t::SIDEBAR + d::PX_1;
    let full = bounds.right() - d::PX_1 - left;
    let width = full - slide * (p::PANEL_WIDTH + p::CONTENT_RIGHT);
    let clip = rect(left, t::HEADER, full, bounds.bottom() - t::HEADER - d::PX_1);
    let mut content = ui.new_child(egui::UiBuilder::new().max_rect(clip));
    content.set_clip_rect(clip);
    let background = click(&mut content, clip, "dismiss-menu-actions", "");
    let ui_content = &mut content;
    if let Some(key) = key(state.page) {
        let preview_width = if key == Shortcut::Keypad {
            menu_size(menu(&c.shortcuts, key), key).x
        } else {
            s::PREVIEW_WIDTH
        };
        // Options always sit below the preview, so opening the actions panel
        // narrows the page without moving them.
        let block_width = preview_width.max(s::OPTIONS_WIDTH);
        let x = left + ((width - block_width) / d::PX_2).max(t::CONTENT_PAD);
        let m = menu(&c.shortcuts, key);
        let size = menu_size(m, key);
        let editing = state.actions.selected.is_some() && key != Shortcut::Keypad;
        let content_height = size.y + if editing { d::PX_600 } else { d::PX_500 };
        let max_scroll = (content_height - (bounds.bottom() - t::HEADER)).max(0.);
        if crate::widgets::contains_pointer(ui_content, clip) {
            state.scroll =
                (state.scroll - crate::widgets::page_wheel(ui_content)).clamp(0., max_scroll);
        }
        crate::widgets::scroll_indicator(
            ui_content,
            egui::Id::new("shortcuts-scroll"),
            crate::widgets::scroll_track(rect(left, clip.top(), width, clip.height())),
            &mut state.scroll,
            max_scroll,
        );
        let dy = state.scroll;
        centered(
            ui_content,
            pos2(x + preview_width / d::PX_2, d::PX_167 - dy),
            &l.t(title(key)),
            d::PX_22,
            t::MUTED,
        );
        let menu_origin = pos2(
            x + (preview_width - size.x) / d::PX_2,
            s::MENU_TOP - dy
                + if matches!(key, Shortcut::ExpressMenu | Shortcut::RadialMenu) {
                    d::PX_3
                } else {
                    0.
                },
        );
        let response = draw_menu(
            ui_content,
            l,
            &c.shortcuts,
            key,
            menu_origin,
            state.actions.selected,
            true,
        );
        paragraph(ui_content,rect(x,menu_origin.y+size.y+if matches!(key,Shortcut::ExpressMenu|Shortcut::RadialMenu){d::PX_26}else{d::PX_23},preview_width,d::PX_60),&l.t(if key==Shortcut::Keypad{"Each key types once and closes the keypad."}else{"Click an entry to choose its action. Assign this menu to a pen button or ExpressKey to open it."}),d::PX_15,t::MUTED);
        // Centered below the preview; with the panel open they lean left,
        // clear of the Default button.
        let ox = x + (block_width - s::OPTIONS_WIDTH) / d::PX_2 * (1. - slide);
        let oy = menu_origin.y + size.y + d::PX_170;
        let m = menu_mut(&mut c.shortcuts, key);
        crate::widgets::slider(
            ui_content,
            l,
            rect(ox, oy, s::OPTIONS_WIDTH, d::PX_92),
            "Size",
            "",
            &mut m.size,
            1..=5,
            ("Small", "Large"),
        );
        crate::widgets::slider(
            ui_content,
            l,
            rect(ox, oy + s::SLIDER_STEP, s::OPTIONS_WIDTH, d::PX_92),
            "Text size",
            "",
            &mut m.text_size,
            1..=5,
            ("Small", "Large"),
        );
        devices::switch(
            ui_content,
            ox,
            oy + s::TOGGLE_TOP - s::OPTIONS_TOP,
            &mut m.at_cursor,
            &l.t("Display at cursor"),
        );
        // The selected entry's name comes last, so nothing above it moves.
        if let Some(index) = state.actions.selected.filter(|_| editing) {
            if m.items.len() <= index {
                m.items.resize(index + 1, Default::default());
            }
            let ny = oy + s::NAME_TOP - s::OPTIONS_TOP;
            text(ui_content, ox, ny, &l.t("Label"), d::PX_16, t::TEXT);
            let field = rect(ox, ny + d::PX_28, s::OPTIONS_WIDTH, d::PX_36);
            ui_content.scope_builder(egui::UiBuilder::new().max_rect(field), |ui| {
                let response = crate::widgets::text_field(
                    ui,
                    field,
                    &mut m.items[index].label,
                    "",
                    usize::MAX,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, l.t("Label"))
                });
            });
        }
        let default = rect(
            left + width - d::PX_176 + d::PX_1,
            bounds.bottom() - d::PX_60,
            d::PX_145,
            d::PX_34,
        );
        if devices::button(ui_content, default, &l.t("Default")) {
            *menu_mut(&mut c.shortcuts, key) = menu(&ShortcutSettings::default(), key).clone();
            state.actions.selected = None;
        }
        state.actions.menu = Some(key);
        if background.clicked() && !state.actions.capture && state.actions.action_dialog.is_none() {
            state.actions.selected = None;
        }
        devices::overlays(ui, l, c, &Default::default(), &mut state.actions);
        if let Some(index) = response.item.filter(|_| key != Shortcut::Keypad) {
            state.actions.selected = if state.actions.selected == Some(index) {
                None
            } else {
                Some(index)
            };
        }
    } else {
        gesture(ui_content, l, c, state.page, left, width, state.scroll);
        let max_scroll = (d::PX_738 - bounds.bottom()).max(0.);
        if crate::widgets::contains_pointer(ui_content, clip) {
            state.scroll =
                (state.scroll - crate::widgets::page_wheel(ui_content)).clamp(0., max_scroll);
        }
        crate::widgets::scroll_indicator(
            ui_content,
            egui::Id::new("gestures-scroll"),
            crate::widgets::scroll_track(clip),
            &mut state.scroll,
            max_scroll,
        );
    }
}
fn start(ui: &mut Ui, l: &Locale, state: &mut State) {
    let bounds = crate::widgets::view(ui);
    let clip = rect(
        d::PX_201,
        t::HEADER,
        bounds.width() - d::PX_202,
        bounds.bottom() - t::HEADER - d::PX_1,
    );
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(clip));
    child.set_clip_rect(clip);
    let ui = &mut child;
    ui.painter().text(
        pos2(d::PX_207, d::PX_116),
        Align2::LEFT_TOP,
        l.t("On-screen shortcuts"),
        t::bold(d::PX_150),
        s::WATERMARK,
    );
    ui.painter().text(
        pos2(d::PX_289, d::PX_171),
        Align2::LEFT_TOP,
        l.t("On-screen shortcuts"),
        t::bold(d::PX_30),
        t::TEXT,
    );
    text(
        ui,
        d::PX_289,
        d::PX_215,
        &l.t("Assign an on-screen shortcut to a pen button or ExpressKey."),
        d::PX_15,
        t::MUTED,
    );
    let card_width = (bounds.width() - d::PX_358) / d::PX_3;
    for (i, (page, name, title, description)) in [
        (
            1,
            "express_menu",
            "Radial menus",
            "Radial menus open around the pen, so reaching an action takes a short flick.",
        ),
        (
            3,
            "app_shortcuts",
            "Grid panels",
            "Grid panels hold more actions, like your app's shortcuts or a numeric keypad.",
        ),
        (
            5,
            "gesture",
            "Pen gestures",
            "Pen gestures scroll, pan and zoom while you hold the button and move the pen.",
        ),
    ]
    .iter()
    .enumerate()
    {
        let x = d::PX_289 + i as f32 * (card_width + d::PX_20);
        let r = rect(x, d::PX_285, card_width, d::PX_170);
        let response = click(ui, r, ("start-card", i), &l.t(title));
        ui.painter().rect(
            r,
            t::CARD_RADIUS,
            crate::widgets::hover_color(&response, t::PANEL, t::RAISED),
            Stroke::new(t::BORDER, t::LINE),
            StrokeKind::Inside,
        );
        icon(
            ui,
            name,
            rect(x + d::PX_23, d::PX_308, d::PX_44, d::PX_44),
            t::ACCENT,
        );
        ui.painter().text(
            pos2(x + d::PX_23, d::PX_364),
            Align2::LEFT_TOP,
            l.t(title),
            t::bold(d::PX_15),
            t::TEXT,
        );
        let mut job = egui::text::LayoutJob::simple(
            l.t(description),
            t::font(d::PX_14),
            t::MUTED,
            card_width - d::PX_46,
        );
        for section in &mut job.sections {
            section.format.line_height = Some(d::PX_19_6);
        }
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        ui.painter()
            .galley(pos2(x + d::PX_23, d::PX_394), galley, t::MUTED);
        if response.clicked() {
            state.page = *page;
        }
    }
}
fn gesture(
    ui: &mut Ui,
    l: &Locale,
    c: &mut Config,
    page: usize,
    left: f32,
    width: f32,
    scroll: f32,
) {
    let cx = left + width / d::PX_2;
    centered(
        ui,
        pos2(cx, d::PX_165 - scroll),
        &l.t(PAGES[page].1),
        d::PX_26,
        t::MUTED,
    );
    let vertical = page == 8 || page == 10;
    let source = if vertical {
        egui::include_image!("../../../../assets/icons/gesture-illustration-vertical.svg")
    } else {
        egui::include_image!("../../../../assets/icons/gesture-illustration.svg")
    };
    egui::Image::new(source).paint_at(
        ui,
        rect(
            cx - s::GESTURE_SIZE / d::PX_2,
            s::GESTURE_TOP - scroll,
            s::GESTURE_SIZE,
            s::GESTURE_SIZE,
        ),
    );
    let speed = match page {
        5 => &mut c.pan_scroll.speed,
        6 => &mut c.gestures.legacy_speed,
        7 => &mut c.gestures.pan_zoom_speed,
        8 => &mut c.gestures.scroll_speed,
        9 => &mut c.gestures.wheel_speed,
        _ => &mut c.gestures.zoom_speed,
    };
    crate::widgets::slider(
        ui,
        l,
        rect(
            cx - s::SPEED_WIDTH / d::PX_2,
            s::SPEED_TOP - scroll,
            s::SPEED_WIDTH,
            d::PX_92,
        ),
        "Speed",
        "",
        speed,
        1..=5,
        ("Slow", "Fast"),
    );
    if page == 5 {
        devices::switch(
            ui,
            cx - s::SPEED_WIDTH / d::PX_2,
            s::FOLLOW_TOP - scroll,
            &mut c.pan_scroll.content_follows_pen,
            &l.t("Content follows the pen"),
        );
    }
    let hint=match page {
        5=>"Hold the button, touch the tablet and drag: the content moves with the pen.",
        6=>"Hold the button and touch the tablet: the farther you move the pen, the faster it scrolls, even when the pen rests.",
        7=>"Hold the button and drag the tip to pan. Hold it with the pen in the air and move up to zoom in or down to zoom out.",
        8=>"Hold the button, touch the tablet and drag up or down to scroll.",
        9=>"Works like a mouse wheel: the farther you move the pen, the faster it turns.",
        _=>"Hold the button, touch the tablet and move the pen up to zoom in or down to zoom out.",
    };
    paragraph(
        ui,
        rect(
            cx - s::PREVIEW_WIDTH / d::PX_2,
            if page == 5 {
                s::HINT_TOP
            } else {
                s::HINT_PLAIN_TOP
            } - scroll,
            s::PREVIEW_WIDTH,
            d::PX_90,
        ),
        &l.t(hint),
        d::PX_15,
        t::MUTED,
    );
}
