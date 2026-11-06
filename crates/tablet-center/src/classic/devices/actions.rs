//! Action names and icons, the actions panel and its dialogs.
use super::*;

pub fn action_label(a: &tablet_config::KeyAction) -> String {
    match a {
        tablet_config::KeyAction::Keystroke { label, .. }
        | tablet_config::KeyAction::Run { label, .. } => label.clone(),
        _ => {
            let value = serde_json::to_value(a).unwrap_or_default();
            action_name(value["type"].as_str().unwrap_or("disabled"))
        }
    }
}
pub(super) fn action_name(id: &str) -> String {
    GROUPS
        .iter()
        .flat_map(|(_, items)| items.iter())
        .find(|(v, _)| *v == id)
        .map(|(_, label)| label.to_string())
        .unwrap_or_else(|| match id {
            "default" => "Default".into(),
            "right_or_hold_middle" => "Right · hold Middle".into(),
            _ => id.replace('_', " "),
        })
}

pub(crate) const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "Recommended",
        &[
            ("disabled", "Disabled"),
            ("undo", "Undo"),
            ("redo", "Redo"),
            ("pan_scroll", "Pan/Scroll"),
            ("display_toggle", "Display toggle"),
            ("record", "Keyboard shortcut…"),
        ],
    ),
    (
        "Clicks",
        &[
            ("left_click", "Click"),
            ("right_click", "Right click"),
            ("middle_click", "Middle click"),
            ("double_click", "Double click"),
            ("back", "Back"),
            ("forward", "Forward"),
            ("click_lock", "Click lock"),
            ("middle_click_lock", "Middle click lock"),
            ("right_click_lock", "Right click lock"),
        ],
    ),
    (
        "Keyboard",
        &[
            ("undo", "Undo"),
            ("redo", "Redo"),
            ("copy", "Copy"),
            ("paste", "Paste"),
            ("save", "Save"),
            ("record", "Keyboard shortcut…"),
            ("modifier", "Modifier…"),
        ],
    ),
    (
        "Tablet",
        &[
            ("display_toggle", "Display toggle"),
            ("precision_mode", "Precision mode"),
            ("pressure_hold", "Pressure hold"),
            ("mode_toggle", "Mode toggle"),
            ("pen_mouse_toggle", "Pen/Mouse toggle"),
        ],
    ),
    ("Applications", &[("run", "Run application…")]),
    (
        "Other",
        &[
            ("show_desktop", "Show desktop"),
            ("overview", "Overview"),
            ("switch_application", "Switch application"),
            ("notification_center", "Notification center"),
            ("on_screen_keyboard", "On-screen keyboard"),
        ],
    ),
    (
        "Radial menus",
        &[
            ("express_menu", "Express menu"),
            ("radial_menu", "Radial menu"),
        ],
    ),
    (
        "Grid panels",
        &[("app_shortcuts", "App shortcuts"), ("keypad", "Keypad")],
    ),
    (
        "Pen gestures",
        &[
            ("pan_scroll", "Pan/Scroll"),
            ("pan_scroll_legacy", "Pan/Scroll (Legacy)"),
            ("pan_zoom", "Pan/Zoom"),
            ("scroll", "Scroll"),
            ("wheel", "Wheel"),
            ("zoom", "Zoom"),
        ],
    ),
];
/// Icon id of a pen action: its type, with the hold variant and key-like
/// actions mapped to their family.
pub fn pen_action_id(a: &tablet_config::ButtonAction) -> String {
    let value = serde_json::to_value(a).unwrap_or_default();
    match &value {
        serde_json::Value::String(id) if id == "right_or_hold_middle" => "right_click".into(),
        serde_json::Value::String(id) => id.clone(),
        serde_json::Value::Object(o) => match o.keys().next().map(String::as_str) {
            Some("keystroke" | "run" | "modifier") | None => "keyboard".into(),
            Some(other) => other.into(),
        },
        _ => "default".into(),
    }
}
pub fn pen_label(a: &tablet_config::ButtonAction) -> String {
    let value = serde_json::to_value(a).unwrap_or_default();
    if let Some(id) = value.as_str() {
        action_name(id)
    } else {
        let Some((id, fields)) = value.as_object().and_then(|v| v.iter().next()) else {
            return String::new();
        };
        fields["label"]
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| action_name(id))
    }
}

pub fn overlays(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response, s: &mut State) -> bool {
    let before = c.clone();
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        s.selected = None;
        s.add_app = false;
        s.capture = false;
        s.action_dialog = None;
        s.toggle_help = false;
        s.pending_orientation = None;
        s.remove_app = false;
    }
    if s.panel_right {
        // The pen's panel slides in from the right, and back out on close
        // showing the last button, inert, until it is gone.
        let progress = crate::classic::pen::panel_progress(ui.ctx(), s.selected.is_some());
        let slide = (1. - progress) * (p::PANEL_WIDTH + p::CONTENT_RIGHT);
        if s.selected.is_some() {
            s.panel_last = s.selected;
            actions(ui, l, c, s, slide);
        } else if progress > 0. {
            if let Some(last) = s.panel_last {
                let mut closing = ui.new_child(egui::UiBuilder::new());
                closing.disable();
                s.selected = Some(last);
                actions(&mut closing, l, c, s, slide);
                s.selected = None;
            }
        }
    } else if s.selected.is_some() {
        actions(ui, l, c, s, 0.);
    }
    if s.add_app {
        app_dialog(ui, l, c, s);
    }
    if s.capture {
        capture(ui, l, c, s);
    }
    if s.action_dialog.is_some() {
        configure(ui, l, c, s);
    }
    if s.pending_orientation.is_some() || s.remove_app {
        confirmation(ui, l, c, s);
    }
    if s.toggle_help {
        toggle_dialog(ui, l, c, status, s);
    }
    *c != before
}
pub(super) fn apply(c: &mut Config, s: &State, a: tablet_config::KeyAction) {
    if let Some(i) = s.selected {
        if let Some(menu) = s.menu {
            let menu = crate::classic::shortcuts::menu_mut(&mut c.shortcuts, menu);
            let count = if i < 12 { i + 1 } else { 12 };
            if menu.items.len() < count {
                menu.items.resize(count, Default::default());
            }
            if let Some(item) = menu.items.get_mut(i) {
                item.action = a;
            }
            return;
        }
        if is_pen(i) {
            let pen = if let Some(i) = s.app {
                &mut c.apps[i].pen
            } else {
                &mut c.pen
            };
            let mut value = serde_json::to_value(&a).unwrap_or_default();
            let name = value["type"].as_str().unwrap_or("disabled").to_string();
            if let Some(obj) = value.as_object_mut() {
                obj.remove("type");
            }
            let v = if value.as_object().is_none_or(|o| o.is_empty()) {
                serde_json::Value::String(name)
            } else {
                serde_json::json!({name:value})
            };
            if let Ok(action) = serde_json::from_value(v) {
                match i {
                    PEN_UPPER => pen.upper_button = action,
                    PEN_LOWER => pen.lower_button = action,
                    _ => pen.tip = action,
                }
            }
        } else {
            let keys = if let Some(i) = s.app {
                &mut c.apps[i].express_keys
            } else {
                &mut c.express_keys
            };
            if let Some(k) = keys.get_mut(i) {
                *k = a;
            }
        }
    }
}

/// `slide` moves the right-hand panel out by that many points (pen page).
pub(super) fn actions(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State, slide: f32) {
    let h = crate::widgets::view(ui).bottom();
    let mut overlay = ui.new_child(egui::UiBuilder::new().layer_id(crate::widgets::layer(
        ui,
        egui::Order::Foreground,
        egui::Id::new("actions"),
    )));
    let ui = &mut overlay;
    let w = crate::widgets::view(ui).right();
    // Left: the classic column beside the sidebar. Right (pen page): a card
    // like an inspector, titled with the button and its action.
    let flat = s.panel_right;
    let (left, width, list_top, clip) = if flat {
        // Shortcut pages have no app strip, so the panel starts higher.
        let top = if s.menu.is_some() {
            t::HEADER + t::CONTENT_PAD
        } else {
            p::PAGE_TOP + t::SPACE_14
        };
        // Keep the sliding panel inside the window frame.
        ui.set_clip_rect(rect(0., 0., w - d::PX_1, h - d::PX_1));
        let r = rect(
            w - p::CONTENT_RIGHT - p::PANEL_WIDTH + slide,
            top,
            p::PANEL_WIDTH,
            h - top - t::CONTENT_PAD,
        );
        card(ui, r);
        let mut header = p::PANEL_HEADER;
        if let Some(index) = s.selected {
            // The button (or menu entry) and what it does now.
            let (name, current) = if let Some(menu) = s.menu {
                let entry = crate::classic::shortcuts::items(&c.shortcuts, menu)
                    .get(index)
                    .map(|item| crate::classic::shortcuts::item_label(l, item))
                    .unwrap_or_default();
                (l.t(crate::classic::shortcuts::title(menu)), entry)
            } else if s.panel_keys {
                let keys = s.app.map_or(&c.express_keys, |i| &c.apps[i].express_keys);
                (
                    format!("{} {}", l.t("ExpressKey"), index + 1),
                    keys.get(index).map(action_label).unwrap_or_default(),
                )
            } else {
                let pen = s.app.map_or(&c.pen, |i| &c.apps[i].pen);
                let action = match index {
                    PEN_UPPER => &pen.upper_button,
                    PEN_LOWER => &pen.lower_button,
                    _ => &pen.tip,
                };
                let name =
                    ["Upper button", "Lower button", "Tip"][index.saturating_sub(PEN_UPPER).min(2)];
                (l.t(name), pen_label(action))
            };
            text(
                ui,
                r.left() + p::PANEL_INSET,
                r.top() + p::PANEL_TITLE_TOP,
                &name,
                d::PX_17,
                t::TEXT,
            );
            let subtitle = crate::widgets::two_lines(
                ui,
                &l.t(&current),
                d::PX_13,
                r.width() - 2. * p::PANEL_INSET,
            );
            header = header.max(p::PANEL_SUBTITLE_TOP + subtitle.size().y + p::PANEL_INSET);
            ui.painter().galley(
                pos2(r.left() + p::PANEL_INSET, r.top() + p::PANEL_SUBTITLE_TOP),
                subtitle,
                t::MUTED,
            );
        }
        let close = rect(
            r.right() - p::PANEL_INSET - p::PANEL_CLOSE,
            r.top() + p::PANEL_TITLE_TOP,
            p::PANEL_CLOSE,
            p::PANEL_CLOSE,
        );
        let response = click(ui, close, "close-actions", &l.t("Close"));
        if response.hovered() {
            ui.painter()
                .rect_filled(close, t::CONTROL_RADIUS, t::RAISED);
        }
        egui::Image::new(crate::icons::source("x"))
            .tint(crate::widgets::hover_color(&response, t::MUTED, t::TEXT))
            .paint_at(ui, close.shrink(t::SPACE_6));
        if response.clicked() {
            s.selected = None;
        }
        let divider = r.top() + header;
        ui.painter().hline(
            r.left() + p::PANEL_INSET..=r.right() - p::PANEL_INSET,
            divider,
            Stroke::new(t::BORDER, t::LINE),
        );
        let clip = rect(
            r.left(),
            divider + d::PX_1,
            r.width(),
            r.bottom() - divider - p::PANEL_INSET,
        );
        (r.left(), r.width(), clip.top() + t::SPACE_10, clip)
    } else {
        ui.painter().rect_filled(
            rect(d::PX_201, d::PX_116, d::PX_230, h - d::PX_117),
            0,
            t::PANEL,
        );
        text(ui, d::PX_217, d::PX_130, &l.t("Actions"), d::PX_20, t::TEXT);
        (
            d::PX_201,
            d::PX_230,
            d::PX_168,
            rect(d::PX_201, d::PX_166, d::PX_230, h - d::PX_167),
        )
    };
    let dx = left - d::PX_201;
    ui.set_clip_rect(clip);
    // The overlay layer is not hit-tested, so check the pointer by position.
    if crate::widgets::pointer(ui).is_some_and(|p| clip.contains(p)) {
        let limit = ui
            .ctx()
            .data(|d| d.get_temp::<f32>(egui::Id::new("action-scroll-limit")))
            .unwrap_or(d::PX_360);
        s.action_scroll =
            (s.action_scroll - ui.input(|i| i.smooth_scroll_delta.y)).clamp(0., limit);
    }
    let mut y = list_top - s.action_scroll;
    let pen = if let Some(i) = s.app {
        &mut c.apps[i].pen
    } else {
        &mut c.pen
    };
    let action = match s.selected.filter(|_| s.menu.is_none()) {
        Some(PEN_UPPER) => Some(&pen.upper_button),
        Some(PEN_LOWER) => Some(&pen.lower_button),
        Some(PEN_TIP) => Some(&pen.tip),
        _ => None,
    };
    if action == Some(&tablet_config::ButtonAction::RightOrHoldMiddle) {
        let (card_left, card_width) = if flat {
            (left + p::PANEL_INSET, width - 2. * p::PANEL_INSET)
        } else {
            (d::PX_217 + dx, d::PX_198)
        };
        let inner = card_width - 2. * d::PX_14;
        let note = l.t("How long to hold the button before it becomes a middle click. Moving the pen while holding it starts the middle click at once.");
        // The card ends a padding below its description, measured as drawn.
        let mut job =
            egui::text::LayoutJob::simple(note.clone(), t::font(d::PX_12), t::MUTED, inner);
        job.sections[0].format.line_height = Some(d::PX_12 * d::LINE_HEIGHT);
        let note_height = ui.fonts_mut(|f| f.layout_job(job)).size().y;
        let card_height = p::HOLD_NOTE_TOP + note_height + d::PX_14;
        let r = rect(card_left, y, card_width, card_height);
        card(ui, r);
        let mut level = pen.hold_time.clamp(1, 7);
        if crate::widgets::slider(
            ui,
            l,
            rect(
                r.left() + d::PX_14,
                y + d::PX_14,
                inner,
                p::HOLD_SLIDER_HEIGHT,
            ),
            "Hold time",
            "",
            &mut level,
            1..=7,
            ("Short", "Long"),
        ) {
            pen.hold_time = level;
        }
        wrapped(
            ui,
            rect(
                r.left() + d::PX_14,
                y + p::HOLD_NOTE_TOP,
                inner,
                note_height,
            ),
            &note,
            d::PX_12,
            t::MUTED,
        );
        y += card_height + d::PX_18;
    }

    let current = current_action(c, s);
    let mut groups = GROUPS.to_vec();
    if s.menu.is_none() && s.selected.is_some_and(is_pen) {
        groups[0] = (
            "Recommended",
            &[
                ("default", "Default"),
                ("right_click", "Right click"),
                ("middle_click", "Middle click"),
                ("right_or_hold_middle", "Right · hold Middle"),
                ("pan_scroll", "Pan/Scroll"),
                ("display_toggle", "Display toggle"),
                ("disabled", "Disabled"),
            ],
        );
    }
    if s.menu.is_some() {
        groups[0] = (
            "Recommended",
            &[
                ("disabled", "Disabled"),
                ("undo", "Undo"),
                ("redo", "Redo"),
                ("display_toggle", "Display toggle"),
                ("record", "Keyboard shortcut…"),
            ],
        );
        groups[3] = (
            "Tablet",
            &[
                ("display_toggle", "Display toggle"),
                ("precision_mode", "Precision mode"),
            ],
        );
        groups.pop();
    }
    for (i, (label, items)) in groups.iter().enumerate() {
        if flat {
            // Every group open under a small caps heading.
            if i > 0 {
                y += p::PANEL_GROUP_GAP;
            }
            crate::widgets::heading(ui, left + p::PANEL_INSET, y + t::SPACE_6, &l.t(label));
            y += p::PANEL_HEADING_STEP;
        } else {
            text(ui, d::PX_231, y, &l.t(label), d::PX_16, t::TEXT);
            let expansion = ui.ctx().animate_bool_with_time(
                ui.id().with(("action-disclosure", i)),
                s.group_open(i),
                t::interaction::SWITCH_TIME,
            );
            if expansion > 0. && expansion < 1. {
                let center = pos2(d::PX_221, y + d::PX_7);
                let rotation =
                    egui::emath::Rot2::from_angle(expansion * std::f32::consts::FRAC_PI_2);
                let points = [
                    vec2(-d::PX_4, -d::PX_4),
                    vec2(0., 0.),
                    vec2(-d::PX_4, d::PX_4),
                ]
                .map(|v| center + rotation * v);
                ui.painter().add(egui::Shape::line(
                    points.to_vec(),
                    Stroke::new(d::THIN_STROKE, t::MUTED),
                ));
            } else if expansion == 0. {
                text(ui, d::PX_215, y, "›", d::PX_16, t::MUTED);
            }
            if expansion == 1. {
                ui.painter().add(egui::Shape::line(
                    vec![
                        pos2(d::PX_217, y + d::PX_7),
                        pos2(d::PX_221, y + d::PX_11),
                        pos2(d::PX_225, y + d::PX_7),
                    ],
                    Stroke::new(d::THIN_STROKE, t::MUTED),
                ));
            }
            if click(
                ui,
                rect(d::PX_211, y - d::PX_4, d::PX_220, d::PX_28),
                ("group", i),
                label,
            )
            .clicked()
            {
                if s.toggled_groups.contains(&i) {
                    s.toggled_groups.remove(&i);
                } else {
                    s.toggled_groups.insert(i);
                }
            }
            y += crate::theme::shortcuts_layout::ACTION_GROUP_HEIGHT;
        }
        if flat || s.group_open(i) {
            for (id, label) in *items {
                let (row_left, row_width) = if flat {
                    (left + p::ROW_INSET, width - 2. * p::ROW_INSET)
                } else {
                    (d::PX_201, d::PX_230)
                };
                // Long names wrap to a second line and the row grows.
                let name = crate::widgets::two_lines(
                    ui,
                    &l.t(label),
                    d::PX_15,
                    row_left + row_width - d::PX_8 - (d::PX_263 + dx),
                );
                let row_height = (name.size().y + 2. * d::PX_8).max(d::PX_38);
                let r = rect(row_left, y, row_width, row_height);
                let response = click(ui, r, ("action", i, id), label);
                // Display toggle explains itself in a dialog; the mark sits
                // over the row so it takes the click.
                let help = (*id == "display_toggle").then(|| {
                    let center = pos2(r.right() - tg::MARK_RIGHT, r.center().y);
                    click(
                        ui,
                        egui::Rect::from_center_size(center, vec2(tg::CLOSE, tg::CLOSE)),
                        ("toggle-help", i),
                        &l.t("More information"),
                    )
                });
                if response.hovered() {
                    ui.painter()
                        .rect_filled(r, t::CONTROL_RADIUS, t::interaction::WHITE_6);
                }
                if current["type"].as_str() == Some(id)
                    || current["label"].as_str() == Some(l.t(label).as_str())
                {
                    ui.painter().rect_filled(
                        r,
                        if flat { t::CONTROL_RADIUS } else { t::SQUARE },
                        t::ACCENT,
                    );
                }
                ui.painter()
                    .galley(pos2(d::PX_263 + dx, y + d::PX_8), name, t::TEXT);
                let icon = action_icon(id);
                egui::Image::new(icon)
                    .tint(t::TEXT)
                    .paint_at(ui, rect(d::PX_231 + dx, y + d::PX_7, d::PX_22, d::PX_22));
                if let Some(help) = &help {
                    crate::widgets::help_mark(
                        ui,
                        help.rect.center(),
                        help.hovered() || help.is_pointer_button_down_on(),
                    );
                    if help.clicked() {
                        s.toggle_help = true;
                    }
                }
                if response.clicked() {
                    if *id == "record" {
                        s.capture = true;
                        s.keys.clear();
                        s.captured.clear();
                    } else if *id == "right_or_hold_middle" || *id == "default" {
                        let pen = if let Some(i) = s.app {
                            &mut c.apps[i].pen
                        } else {
                            &mut c.pen
                        };
                        let a = if *id == "default" {
                            tablet_config::ButtonAction::Default
                        } else {
                            tablet_config::ButtonAction::RightOrHoldMiddle
                        };
                        match s.selected {
                            Some(PEN_UPPER) => pen.upper_button = a,
                            Some(PEN_LOWER) => pen.lower_button = a,
                            _ => pen.tip = a,
                        }
                    } else if ["run", "modifier", "mode_toggle", "precision_mode"].contains(id) {
                        s.action_dialog = Some(id.to_string());
                        s.run_list_scroll = 0.;
                        let pen = if let Some(i) = s.app {
                            &c.apps[i].pen
                        } else {
                            &c.pen
                        };
                        s.speed = pen.relative.speed;
                        s.acceleration = pen.relative.acceleration;
                        s.precision_scale = c.precision.scale;
                        s.precision_overlay = c.precision.overlay;
                        let value = current_action(c, s);
                        s.command = value["command"].as_str().unwrap_or_default().into();
                        for (i, key) in ["shift", "ctrl", "alt", "super"].iter().enumerate() {
                            s.modifiers[i] = value[key].as_bool().unwrap_or(false);
                        }
                        s.modifier_click = match value["click"].as_str() {
                            Some("left") => 1,
                            Some("right") => 2,
                            Some("middle") => 3,
                            _ => 0,
                        };
                    } else {
                        let a = match *id {
                            "undo" | "redo" | "copy" | "paste" | "save" => {
                                let key = match *id {
                                    "undo" => "Z",
                                    "redo" => "Z",
                                    "copy" => "C",
                                    "paste" => "V",
                                    _ => "S",
                                };
                                tablet_config::KeyAction::Keystroke {
                                    keys: if *id == "redo" {
                                        vec![
                                            "KEY_LEFTCTRL".into(),
                                            "KEY_LEFTSHIFT".into(),
                                            "KEY_Z".into(),
                                        ]
                                    } else {
                                        vec!["KEY_LEFTCTRL".into(), format!("KEY_{key}")]
                                    },
                                    label: l.t(label),
                                }
                            }
                            _ => serde_json::from_value(serde_json::json!({"type":id}))
                                .unwrap_or_default(),
                        };
                        apply(c, s, a);
                    }
                }
                y += row_height;
            }
        }
    }
    // With several groups open the list grows, so the scroll range follows it.
    let content = y + s.action_scroll - list_top;
    let limit = (content - clip.height() + d::PX_24).max(0.);
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new("action-scroll-limit"), limit));
    let total_height = y + s.action_scroll - clip.top();
    let overflow = (total_height - clip.height()).max(0.);
    s.action_scroll = s.action_scroll.clamp(0., overflow);
    crate::widgets::scroll_indicator(
        ui,
        ui.id().with("actions-scrollbar"),
        rect(
            left + width - d::PX_6,
            clip.top(),
            t::scroll::WIDTH,
            clip.height() - d::PX_4,
        ),
        &mut s.action_scroll,
        overflow,
    );
}
pub(super) fn capture(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
    let r = egui::Rect::from_center_size(
        crate::widgets::window(ui).center(),
        vec2(d::PX_380, d::PX_221),
    );
    let mut child = backdrop(ui, "capture", r);
    let ui = &mut child;
    text(
        ui,
        r.left() + d::PX_24,
        r.top() + d::PX_24,
        &l.t("Keystroke"),
        d::PX_20,
        t::TEXT,
    );
    text(
        ui,
        r.left() + d::PX_24,
        r.top() + d::PX_53,
        &l.t("Press the key combination on your keyboard."),
        d::PX_14,
        t::MUTED,
    );
    let events = ui.input(|i| i.events.clone());
    for e in events {
        if let egui::Event::Key {
            key,
            physical_key,
            pressed: true,
            modifiers,
            ..
        } = e
        {
            let key = physical_key.unwrap_or(key);
            s.keys.clear();
            let mut labels = Vec::new();
            for (on, name, label) in [
                (modifiers.ctrl, "KEY_LEFTCTRL", "Ctrl"),
                (modifiers.shift, "KEY_LEFTSHIFT", "Shift"),
                (modifiers.alt, "KEY_LEFTALT", "Alt"),
                (
                    modifiers.mac_cmd || modifiers.command && !modifiers.ctrl,
                    "KEY_LEFTMETA",
                    "Super",
                ),
            ] {
                if on {
                    s.keys.push(name.into());
                    labels.push(label.to_string());
                }
            }
            s.keys.push(evdev_key(key));
            labels.push(key.name().into());
            s.captured = labels.join("+");
        }
    }
    let cap = rect(r.left() + d::PX_24, r.top() + d::PX_88, d::PX_332, d::PX_60);
    ui.painter().rect(
        cap,
        8,
        t::VIEW,
        Stroke::new(d::PX_2, t::ACCENT),
        StrokeKind::Inside,
    );
    ui.painter().text(
        cap.center(),
        Align2::CENTER_CENTER,
        if s.captured.is_empty() {
            "—"
        } else {
            &s.captured
        },
        t::font(d::PX_20),
        t::TEXT,
    );
    if button(
        ui,
        rect(
            r.right() - d::PX_178,
            r.bottom() - d::PX_55,
            d::PX_80,
            d::PX_33,
        ),
        &l.t("Cancel"),
    ) {
        s.capture = false;
    }
    if primary_button(
        ui,
        rect(
            r.right() - d::PX_88,
            r.bottom() - d::PX_55,
            d::PX_64,
            d::PX_33,
        ),
        &l.t("OK"),
        !s.keys.is_empty(),
    ) && !s.keys.is_empty()
    {
        apply(
            c,
            s,
            tablet_config::KeyAction::Keystroke {
                keys: s.keys.clone(),
                label: s.captured.clone(),
            },
        );
        s.capture = false;
    }
}
pub fn evdev_key(key: egui::Key) -> String {
    use egui::Key::*;
    let name = match key {
        ArrowUp => "UP",
        ArrowDown => "DOWN",
        ArrowLeft => "LEFT",
        ArrowRight => "RIGHT",
        Escape => "ESC",
        Enter => "ENTER",
        Space => "SPACE",
        Tab => "TAB",
        Backspace => "BACKSPACE",
        Delete => "DELETE",
        Insert => "INSERT",
        Home => "HOME",
        End => "END",
        PageUp => "PAGEUP",
        PageDown => "PAGEDOWN",
        Minus => "MINUS",
        Equals => "EQUAL",
        OpenBracket => "LEFTBRACE",
        CloseBracket => "RIGHTBRACE",
        Semicolon => "SEMICOLON",
        Quote => "APOSTROPHE",
        Backtick => "GRAVE",
        Backslash => "BACKSLASH",
        Comma => "COMMA",
        Period => "DOT",
        Slash => "SLASH",
        _ => return format!("KEY_{}", key.name().to_uppercase()),
    };
    format!("KEY_{name}")
}
pub(super) fn confirmation(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
    let label = if let Some(o) = s.pending_orientation {
        let name = match o {
            Orientation::Landscape => "Landscape",
            Orientation::Portrait => "Portrait",
            Orientation::LandscapeFlipped => "Landscape flipped",
            Orientation::PortraitFlipped => "Portrait flipped",
        };
        l.t(&format!(
            "Would you like to apply the orientation \"{name}\"?"
        ))
    } else {
        s.app
            .and_then(|i| c.apps.get(i))
            .map(|a| {
                l.t("Are you sure you want to remove the settings for %s?")
                    .replace("%s", &a.name)
            })
            .unwrap_or_default()
    };
    let r = egui::Rect::from_center_size(
        crate::widgets::window(ui).center(),
        vec2(d::PX_428, d::PX_180),
    );
    let mut child = backdrop(ui, "confirmation", r);
    wrapped(
        &child,
        rect(r.left() + d::PX_24, r.top() + d::PX_24, d::PX_380, d::PX_80),
        &label,
        d::PX_16,
        t::TEXT,
    );
    if button(
        &mut child,
        rect(
            r.right() - d::PX_190,
            r.bottom() - d::PX_55,
            d::PX_85,
            d::PX_32,
        ),
        &l.t("Cancel"),
    ) {
        s.pending_orientation = None;
        s.remove_app = false;
    }
    let action = l.t(if s.remove_app { "Delete" } else { "OK" });
    if button(
        &mut child,
        rect(
            r.right() - d::PX_95,
            r.bottom() - d::PX_55,
            d::PX_71,
            d::PX_32,
        ),
        &action,
    ) {
        if let Some(o) = s.pending_orientation.take() {
            c.mapping.orientation = o;
        } else if let Some(i) = s.app.take() {
            c.apps.remove(i);
            s.selected = None;
            s.app_offset = 0.;
        }
        s.remove_app = false;
    }
}
pub(super) fn configure(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
    poll_apps(ui, s);
    let Some(kind) = s.action_dialog.clone() else {
        return;
    };
    let run = !["modifier", "mode_toggle", "precision_mode"].contains(&kind.as_str());
    let size = if run {
        vec2(rd::WIDTH, rd::HEIGHT)
    } else {
        vec2(d::PX_428, d::PX_330)
    };
    let r = egui::Rect::from_center_size(crate::widgets::window(ui).center(), size);
    let mut child = backdrop(ui, "configure", r);
    let x = r.left() + rd::PAD;
    let y = r.top() + rd::PAD;
    child.painter().text(
        pos2(x, y),
        Align2::LEFT_TOP,
        l.t(match kind.as_str() {
            "modifier" => "Modifier",
            "mode_toggle" => "Mode toggle",
            "precision_mode" => "Precision mode",
            _ => "Open/Run",
        }),
        t::medium(rd::TITLE),
        t::TEXT,
    );
    if run {
        run_fields(&mut child, l, s, r);
    }
    child.scope_builder(
        egui::UiBuilder::new().max_rect(rect(x, y + d::PX_45, d::PX_380, d::PX_200)),
        |ui| match kind.as_str() {
            "modifier" => {
                for (i, label) in ["Shift", "Ctrl", "Alt", "Super"].iter().enumerate() {
                    ui.checkbox(&mut s.modifiers[i], l.t(label));
                }
                egui::ComboBox::from_id_salt("click")
                    .selected_text(
                        l.t(["Disabled", "Left click", "Right click", "Middle click"]
                            [s.modifier_click]),
                    )
                    .show_ui(ui, |ui| {
                        for (i, label) in ["Disabled", "Left click", "Right click", "Middle click"]
                            .iter()
                            .enumerate()
                        {
                            ui.selectable_value(&mut s.modifier_click, i, l.t(label));
                        }
                    });
            }
            "mode_toggle" => {
                ui.add(egui::Slider::new(&mut s.speed, 1..=7).text(l.t("Speed")));
                ui.add(egui::Slider::new(&mut s.acceleration, 1..=7).text(l.t("Acceleration")));
            }
            "precision_mode" => {
                egui::ComboBox::from_id_salt("scale")
                    .selected_text(s.precision_scale.to_string())
                    .show_ui(ui, |ui| {
                        for scale in [2, 3, 4, 6, 8] {
                            ui.selectable_value(&mut s.precision_scale, scale, scale.to_string());
                        }
                    });
                ui.checkbox(&mut s.precision_overlay, l.t("Show overlay"));
            }
            _ => {}
        },
    );
    if button(
        &mut child,
        rect(
            r.right() - d::PX_178,
            r.bottom() - d::PX_57,
            d::PX_80,
            d::PX_33,
        ),
        &l.t("Cancel"),
    ) {
        s.action_dialog = None;
    }
    if button(
        &mut child,
        rect(
            r.right() - d::PX_88,
            r.bottom() - d::PX_57,
            d::PX_64,
            d::PX_33,
        ),
        &l.t("OK"),
    ) {
        let action = match kind.as_str() {
            "modifier" => tablet_config::KeyAction::Modifier {
                shift: s.modifiers[0],
                ctrl: s.modifiers[1],
                alt: s.modifiers[2],
                super_key: s.modifiers[3],
                click: match s.modifier_click {
                    1 => Some(tablet_config::Click::Left),
                    2 => Some(tablet_config::Click::Right),
                    3 => Some(tablet_config::Click::Middle),
                    _ => None,
                },
            },
            "mode_toggle" => {
                let pen = if let Some(i) = s.app {
                    &mut c.apps[i].pen
                } else {
                    &mut c.pen
                };
                pen.relative.speed = s.speed;
                pen.relative.acceleration = s.acceleration;
                tablet_config::KeyAction::ModeToggle
            }
            "precision_mode" => {
                c.precision.scale = s.precision_scale;
                c.precision.overlay = s.precision_overlay;
                tablet_config::KeyAction::PrecisionMode
            }
            _ => tablet_config::KeyAction::Run {
                command: s.command.trim().into(),
                label: s
                    .installed_apps
                    .iter()
                    .find(|a| format!("desktop:{}", a.id) == s.command)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| s.command.clone()),
            },
        };
        apply(c, s, action);
        s.action_dialog = None;
    }
}
/// Display toggle explanation, opened from the help mark of its action.
pub(super) fn toggle_dialog(ui: &mut Ui, l: &Locale, c: &Config, status: &Response, s: &mut State) {
    let r = egui::Rect::from_center_size(
        crate::widgets::window(ui).center(),
        vec2(tg::WIDTH, tg::HEIGHT),
    );
    let mut child = backdrop(ui, "toggle-help", r);
    if crate::widgets::clicked_outside(&child, "toggle-help", r) {
        s.toggle_help = false;
    }
    child.painter().text(
        pos2(r.left() + rd::PAD, r.top() + rd::PAD),
        Align2::LEFT_TOP,
        l.t("Display toggle"),
        t::medium(rd::TITLE),
        t::TEXT,
    );
    toggle(
        &mut child,
        l,
        r.center().x,
        r.top() + tg::ART_TOP,
        tg::ART_SCALE,
        c,
        status,
        s.native,
    );
    if button(
        &mut child,
        rect(
            r.right() - d::PX_88,
            r.bottom() - d::PX_57,
            d::PX_64,
            d::PX_33,
        ),
        &l.t("OK"),
    ) {
        s.toggle_help = false;
    }
}
/// Command field and installed-apps list of the Open/Run dialog in `r`.
pub(super) fn run_fields(ui: &mut Ui, l: &Locale, s: &mut State, r: egui::Rect) {
    let x = r.left() + rd::PAD;
    let width = r.width() - rd::PAD * 2.;
    text(
        ui,
        x,
        r.top() + rd::PAD + rd::FIELD_LABEL_TOP,
        &l.t("Command or file path"),
        rd::LABEL,
        t::MUTED,
    );
    // An app's desktop id is shown as its name until the user types.
    let chosen = s
        .installed_apps
        .iter()
        .position(|a| format!("desktop:{}", a.id) == s.command);
    let mut value = chosen
        .and_then(|i| s.installed_apps.get(i))
        .map_or_else(|| s.command.clone(), |a| a.name.clone());
    let field = crate::widgets::text_field(
        ui,
        rect(
            x,
            r.top() + rd::PAD + rd::FIELD_TOP,
            width,
            rd::FIELD_HEIGHT,
        ),
        &mut value,
        "",
        usize::MAX,
    );
    if field.changed() {
        s.command = value;
    }
    text(
        ui,
        x,
        r.top() + rd::PAD + rd::LIST_LABEL_TOP,
        &l.t("Applications"),
        rd::LABEL,
        t::MUTED,
    );
    let top = r.top() + rd::PAD + rd::LIST_TOP;
    let list =
        egui::Rect::from_min_max(pos2(x, top), pos2(x + width, r.bottom() - rd::LIST_BOTTOM));
    ui.painter().rect(
        list,
        t::control_layout::FIELD_RADIUS,
        t::VIEW,
        Stroke::new(t::BORDER, t::LINE),
        StrokeKind::Inside,
    );
    let apps = s.installed_apps.clone();
    want_icons(ui, s, apps.iter().filter_map(|a| a.icon.as_deref()));
    let inner = list.shrink(rd::LIST_INSET);
    let max_scroll = (apps.len() as f32 * rd::ROW_HEIGHT - inner.height()).max(0.);
    // The dialog layer is not hit-tested, so the wheel is read here.
    if crate::widgets::pointer(ui).is_some_and(|p| list.contains(p)) {
        s.run_list_scroll -= crate::widgets::page_wheel(ui);
    }
    s.run_list_scroll = s.run_list_scroll.clamp(0., max_scroll);
    let mut list_ui = ui.new_child(egui::UiBuilder::new().max_rect(inner));
    list_ui.set_clip_rect(inner);
    if apps.is_empty() {
        list_ui.painter().text(
            pos2(list.center().x, list.top() + d::PX_40),
            Align2::CENTER_TOP,
            l.t(if s.loading.is_some() {
                "Loading…"
            } else {
                "No applications found"
            }),
            t::font(d::PX_15),
            t::MUTED,
        );
    }
    for (i, app) in apps.iter().enumerate() {
        let row = rect(
            inner.left(),
            inner.top() + i as f32 * rd::ROW_HEIGHT - s.run_list_scroll,
            inner.width() - rd::LIST_INSET * 2.,
            rd::ROW_HEIGHT,
        );
        if !row.intersects(inner) {
            continue;
        }
        let response = click(&mut list_ui, row, ("run-app", i), &app.name);
        if chosen == Some(i) {
            list_ui
                .painter()
                .rect_filled(row, t::CONTROL_RADIUS, t::interaction::ACCENT_TINT);
        } else if response.hovered() {
            list_ui
                .painter()
                .rect_filled(row, t::CONTROL_RADIUS, t::RAISED);
        }
        paint_app_icon(
            &list_ui,
            &s.app_icons,
            app.icon.as_deref(),
            &app.name,
            rect(
                row.left() + rd::ICON_LEFT,
                row.center().y - rd::ICON / 2.,
                rd::ICON,
                rd::ICON,
            ),
        );
        crate::widgets::fit_text(
            &list_ui,
            pos2(row.left() + rd::NAME_LEFT, row.center().y - d::PX_10),
            &app.name,
            d::PX_15,
            row.width() - rd::NAME_LEFT - rd::ICON_LEFT,
            row,
            t::TEXT,
        );
        if response.clicked() {
            s.command = format!("desktop:{}", app.id);
        }
    }
    let mut offset = s.run_list_scroll;
    crate::widgets::scroll_indicator(
        ui,
        egui::Id::new("run-list-scroll"),
        crate::widgets::scroll_track(inner),
        &mut offset,
        max_scroll,
    );
    s.run_list_scroll = offset;
}
pub fn action_icon(id: &str) -> egui::ImageSource<'static> {
    crate::icons::source(crate::icons::action(id))
}

pub(super) fn current_action(c: &Config, s: &State) -> serde_json::Value {
    let Some(index) = s.selected else {
        return serde_json::Value::Null;
    };
    if let Some(menu) = s.menu {
        return crate::classic::shortcuts::menu(&c.shortcuts, menu)
            .items
            .get(index)
            .and_then(|item| serde_json::to_value(&item.action).ok())
            .unwrap_or_default();
    }
    if !is_pen(index) {
        let keys = if let Some(i) = s.app {
            &c.apps[i].express_keys
        } else {
            &c.express_keys
        };
        keys.get(index)
            .and_then(|a| serde_json::to_value(a).ok())
            .unwrap_or_default()
    } else {
        let pen = if let Some(i) = s.app {
            &c.apps[i].pen
        } else {
            &c.pen
        };
        let a = match index {
            PEN_UPPER => &pen.upper_button,
            PEN_LOWER => &pen.lower_button,
            _ => &pen.tip,
        };
        let v = serde_json::to_value(a).unwrap_or_default();
        if let Some(id) = v.as_str() {
            serde_json::json!({"type":id})
        } else if let Some((id, fields)) = v.as_object().and_then(|o| o.iter().next()) {
            let mut fields = fields.clone();
            if let Some(fields) = fields.as_object_mut() {
                fields.insert("type".into(), serde_json::Value::String(id.clone()));
            }
            fields
        } else {
            serde_json::Value::Null
        }
    }
}
