//! Settings pages, asynchronous desktop commands and transient feedback.
use super::devices::button;
use crate::theme::settings_layout as s;
use crate::{
    i18n::Locale,
    system, theme as t,
    widgets::{click, rect, text},
};
use egui::{pos2, vec2, Align2, Stroke, StrokeKind, Ui};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use tablet_config::{Config, Response};
pub const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Español"),
    ("pt", "Português"),
    ("fr", "Français"),
    ("de", "Deutsch"),
    ("it", "Italiano"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("ru", "Русский"),
    ("tr", "Türkçe"),
    ("ja", "日本語"),
    ("ko", "한국어"),
    ("zh-CN", "简体中文"),
    ("zh-TW", "繁體中文"),
];
pub const SHELL_HINT: &str = "Radial menus and grid panels are drawn by the Tablet Center GNOME Shell extension: they open at once and never take the focus. Without the extension they open in a window as before.";
#[derive(Clone)]
pub enum Dialog {
    Create,
    Reset,
    Restore(String),
    Delete(String),
}
enum Update {
    Ready(bool, String, Vec<system::BackupInfo>),
    Done(String),
    Backups(Vec<system::BackupInfo>),
    Autostart(bool),
    Config(Box<Config>, bool),
}
#[derive(Default)]
pub struct State {
    pub language: String,
    pub select: Option<usize>,
    pub dialog: Option<Dialog>,
    pub title: String,
    pub toast: Option<(String, Instant)>,
    pub backups: Vec<system::BackupInfo>,
    pub autostart: bool,
    pub native: bool,
    pub busy: bool,
    active_option: usize,
    dialog_focused: bool,
    previous_focus: Option<egui::Id>,
    scroll: f32,
    field_y: f32,
    initialized: bool,
    receiver: Option<mpsc::Receiver<Result<Update, String>>>,
}
impl State {
    fn launch(
        &mut self,
        ctx: &egui::Context,
        job: impl FnOnce() -> Result<Update, String> + Send + 'static,
    ) {
        if !self.native || self.busy {
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        self.busy = true;
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let result = job();
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub fn confirm_dialog(&mut self, ctx: &egui::Context, c: &mut Config, d: Dialog) {
        self.dialog = None;
        if !self.native && matches!(d, Dialog::Reset) {
            *c = Config::default();
        }
        let config = c.clone();
        let title = self.title.clone();
        self.launch(ctx, move || match d {
            Dialog::Create => {
                system::save_config(config)?;
                system::create_backup(title)?;
                Ok(Update::Backups(system::list_backups()?))
            }
            Dialog::Delete(name) => {
                system::delete_backup(name)?;
                Ok(Update::Backups(system::list_backups()?))
            }
            Dialog::Restore(name) => {
                let applied = system::restore_backup(name)?;
                Ok(Update::Config(Box::new(tablet_config::load()), applied))
            }
            Dialog::Reset => {
                let applied = system::reset_config()?;
                Ok(Update::Config(Box::default(), applied))
            }
        });
    }
    pub fn notify(&mut self, message: String) {
        self.toast = Some((message, Instant::now()));
    }
    pub fn initialize(&mut self, ctx: &egui::Context) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        if self.language.is_empty() {
            self.language = "en".into();
        }
        self.launch(ctx, || {
            Ok(Update::Ready(
                system::autostart_enabled()?,
                std::fs::read_to_string(tablet_config::config_path().with_file_name("language"))
                    .unwrap_or_else(|_| "en".into()),
                system::list_backups()?,
            ))
        });
    }
    pub fn poll(&mut self, c: &mut Config, l: &mut Locale) {
        let update = self.receiver.as_ref().and_then(|rx| rx.try_recv().ok());
        if let Some(result) = update {
            self.receiver = None;
            self.busy = false;
            match result {
                Err(e) => self.notify(e),
                Ok(Update::Ready(a, lang, b)) => {
                    self.autostart = a;
                    self.language = lang.trim().into();
                    *l = Locale::new(&self.language);
                    self.backups = b;
                }
                Ok(Update::Done(s)) => {
                    if !s.is_empty() {
                        self.notify(l.t(&s));
                    }
                }
                Ok(Update::Backups(b)) => self.backups = b,
                Ok(Update::Autostart(value)) => self.autostart = value,
                Ok(Update::Config(config, applied)) => {
                    *c = *config;
                    if !applied {
                        self.notify(l.t("Settings saved. Start tabletd to apply them."));
                    }
                }
            }
        }
    }
}
fn width(ui: &Ui, s: &str, size: f32) -> f32 {
    ui.fonts_mut(|f| f.layout_no_wrap(s.into(), t::font(size), t::TEXT).size().x)
}
fn label(ui: &Ui, l: &Locale, x: f32, y: f32, s: &str, size: f32) {
    let offset = if size == s::PX_18 && l.is_cjk() {
        s::PX_1
    } else {
        -s::PX_1
    };
    text(ui, x, y + offset, &l.t(s), size, t::TEXT);
}
fn rule(ui: &Ui, y: f32) {
    ui.painter()
        .hline(s::PX_48..=s::PX_808, y, Stroke::new(s::PX_1, t::LINE));
}
fn toggle(ui: &mut Ui, l: &Locale, y: f32, s: &str, value: &mut bool) -> bool {
    let s = l.t(s);
    let x = s::PX_48 + width(ui, &s, s::PX_15) + s::PX_20;
    text(ui, s::PX_48, y + s::PX_5, &s, s::PX_15, t::TEXT);
    let progress =
        crate::widgets::switch_progress(ui, (&s, y.to_bits()), *value, t::interaction::SWITCH_TIME);
    ui.painter().rect_filled(
        rect(x, y, s::PX_52, s::PX_30),
        s::SWITCH_RADIUS,
        egui::Color32::from(egui::lerp(
            egui::Rgba::from(t::TRACK)..=egui::Rgba::from(t::ACCENT),
            progress,
        )),
    );
    ui.painter().circle_filled(
        pos2(
            x + s::PX_14 + (s::PX_38 - s::PX_14) * progress,
            y + s::PX_15,
        ),
        s::PX_12,
        t::THUMB,
    );
    let hit = click(
        ui,
        rect(s::PX_48, y, x + s::PX_52 - s::PX_48, s::PX_30),
        s.clone(),
        &s,
    )
    .clicked();
    if hit {
        *value = !*value;
    }
    hit
}
fn field(ui: &mut Ui, y: f32, value: &str, state: &mut State) {
    state.field_y = y;
    let r = rect(s::PX_48, y, s::PX_200, s::PX_36);
    let response = click(ui, r, "settings-select", value);
    ui.painter().rect_filled(
        r,
        t::RADIUS,
        crate::widgets::hover_color(&response, t::RAISED, t::interaction::RAISED_HOVER),
    );
    if state.select.is_some() || crate::widgets::focus_visible(ui, &response) {
        ui.painter().rect_stroke(
            r,
            t::RADIUS,
            Stroke::new(s::PX_3, t::settings_layout::FOCUS),
            StrokeKind::Outside,
        );
        ui.painter().rect_stroke(
            r,
            t::RADIUS,
            Stroke::new(s::PX_1, t::ACCENT),
            StrokeKind::Inside,
        );
    }
    text(ui, s::PX_61, y + s::PX_8, value, s::PX_15, t::TEXT);
    ui.painter().line_segment(
        [pos2(s::PX_225, y + s::PX_15), pos2(s::PX_230, y + s::PX_20)],
        Stroke::new(s::PX_1_6, t::MUTED),
    );
    ui.painter().line_segment(
        [pos2(s::PX_230, y + s::PX_20), pos2(s::PX_235, y + s::PX_15)],
        Stroke::new(s::PX_1_6, t::MUTED),
    );

    if !state.busy
        && (response.clicked()
            || (response.has_focus()
                && ui.input(|i| {
                    i.key_pressed(egui::Key::Enter)
                        || i.key_pressed(egui::Key::Space)
                        || i.key_pressed(egui::Key::ArrowDown)
                })))
    {
        response.request_focus();
        ui.input_mut(|i| {
            for key in [egui::Key::Enter, egui::Key::Space, egui::Key::ArrowDown] {
                i.consume_key(egui::Modifiers::NONE, key);
            }
        });
        state.active_option = LANGUAGES
            .iter()
            .position(|(id, _)| *id == state.language)
            .unwrap_or(0);
        state.select = if state.select.is_some() {
            None
        } else {
            Some(0)
        };
    }
}
/// Heading over a section card; returns the card's top edge.
fn section(ui: &Ui, l: &Locale, y: f32, key: &str, width: f32, height: f32) -> f32 {
    ui.painter().text(
        pos2(s::PX_29, y),
        Align2::LEFT_TOP,
        l.t(key),
        t::semibold(s::SECTION_FONT),
        t::TEXT,
    );
    let top = y + s::SECTION_CARD;
    ui.painter().rect(
        rect(s::PX_29, top, width - s::PX_58, height),
        t::CARD_RADIUS,
        t::PANEL,
        Stroke::new(s::PX_1, t::LINE),
        StrokeKind::Inside,
    );
    top + height + s::SECTION_GAP
}
pub fn show(ui: &mut Ui, l: &mut Locale, c: &mut Config, status: &Response, state: &mut State) {
    let bounds = crate::widgets::view(ui);
    if state.dialog.is_none() {
        state.dialog_focused = false;
    }
    let device_height = if status.device.is_some() {
        s::DEVICE_HEIGHT
    } else {
        s::NO_DEVICE_HEIGHT
    };
    let backup_height = if state.backups.is_empty() {
        s::BACKUP_HEIGHT
    } else {
        s::BACKUP_ROWS_HEIGHT
    } + state
        .backups
        .iter()
        .map(|b| backup_row_height(ui, &b.title))
        .sum::<f32>();
    let total = s::SECTION_TOP
        + s::PX_3 * (s::SECTION_CARD + s::SECTION_GAP)
        + s::GENERAL_HEIGHT
        + device_height
        + backup_height
        - s::SECTION_GAP
        + s::BOTTOM_MARGIN;
    let max_scroll = (total - bounds.bottom()).max(s::PX_0);
    if max_scroll > s::PX_0 {
        state.scroll = (state.scroll - crate::widgets::page_wheel(ui)).clamp(s::PX_0, max_scroll);
    } else {
        state.scroll = s::PX_0;
    }
    let viewport = rect(
        s::PX_1,
        t::HEADER,
        bounds.width() - s::PX_2,
        bounds.bottom() - t::HEADER - s::PX_1,
    );
    crate::widgets::scroll_indicator(
        ui,
        egui::Id::new("settings-scroll"),
        crate::widgets::scroll_track(viewport),
        &mut state.scroll,
        max_scroll,
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(viewport), |ui| {
        ui.set_clip_rect(viewport);
        // Each section keeps the coordinates of its former tab, whose card
        // started at CONTENT_TOP; `o` moves them to the section's card.
        let mut y = s::SECTION_TOP - state.scroll;
        let next = section(ui, l, y, "Devices manager", bounds.width(), device_height);
        let o = s::CONTENT_TOP - y - s::SECTION_CARD;
        if let Some(device) = &status.device {
            label(
                ui,
                l,
                s::PX_48,
                s::PX_206 - o,
                "Your connected devices",
                s::PX_18,
            );
            connected_device(ui, device, o);
        } else {
            label(
                ui,
                l,
                s::PX_48,
                s::PX_206 - o,
                "Connect a device to see its settings here.",
                s::PX_15,
            );
        }
        y = next;
        let next = section(
            ui,
            l,
            y,
            "General settings",
            bounds.width(),
            s::GENERAL_HEIGHT,
        );
        general(
            ui,
            l,
            c,
            status,
            state,
            s::CONTENT_TOP - y - s::SECTION_CARD,
        );
        y = next;
        section(ui, l, y, "Backup & restore", bounds.width(), backup_height);
        backups(ui, l, state, bounds, s::CONTENT_TOP - y - s::SECTION_CARD);
    });
    if state.select.is_some() {
        select_overlay(ui, l, state);
    }
    if state.dialog.is_some() {
        dialog(ui, l, c, state);
    }
}
fn general(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response, state: &mut State, o: f32) {
    let bounds = crate::widgets::view(ui);
    label(ui, l, s::PX_48, s::PX_206 - o, "Driver service", s::PX_18);
    ui.painter().circle_filled(
        pos2(s::PX_52_5, s::PX_254_5 - o),
        s::PX_4_5,
        if status.ok { s::RUNNING } else { s::STOPPED },
    );
    label(
        ui,
        l,
        s::PX_65,
        s::PX_246 - o,
        if status.ok { "Running" } else { "Not running" },
        s::PX_15,
    );
    rule(ui, s::PX_295_5 - o);
    let restart = l.t("Restart Driver");
    let restart_width = width(ui, &restart, s::PX_15) + s::PX_36;
    if button(
        ui,
        rect(s::PX_48, s::PX_326 - o, restart_width, s::PX_33),
        &restart,
    ) && !state.busy
    {
        state.launch(ui.ctx(), || {
            system::restart_daemon()?;
            Ok(Update::Done("Driver restarted".into()))
        });
    }
    rule(ui, s::PX_389_5 - o);
    label(ui, l, s::PX_48, s::PX_422 - o, "Language", s::PX_18);
    let value = LANGUAGES
        .iter()
        .find(|(id, _)| *id == state.language)
        .map(|(_, v)| *v)
        .unwrap_or("English");
    field(ui, s::PX_460 - o, value, state);
    rule(ui, s::PX_525_5 - o);
    let old = state.autostart;
    if toggle(
        ui,
        l,
        s::AUTOSTART_Y - o,
        "Tablet Center autostart",
        &mut state.autostart,
    ) && !state.busy
    {
        let enabled = state.autostart;
        if state.native {
            state.autostart = old;
        }
        state.launch(ui.ctx(), move || {
            system::set_autostart(enabled)?;
            Ok(Update::Autostart(enabled))
        });
    }
    rule(ui, s::AUTOSTART_RULE - o);
    toggle(
        ui,
        l,
        s::SHELL_Y - o,
        "On-screen shortcuts in GNOME Shell",
        &mut c.shortcuts.in_shell,
    );
    paragraph(
        ui,
        rect(
            s::PX_48,
            s::HINT_Y - o,
            s::HINT_WIDTH.min(bounds.width() - s::PX_96),
            s::PX_100,
        ),
        &l.t(SHELL_HINT),
        s::PX_13,
        t::MUTED,
    );
}
fn backups(ui: &mut Ui, l: &Locale, state: &mut State, bounds: egui::Rect, o: f32) {
    // Row actions end at the card's inner edge, whatever the window width.
    let actions_width = ["Restore", "Delete"]
        .iter()
        .map(|key| width(ui, &l.t(key), s::PX_15) + s::PX_36)
        .sum::<f32>()
        + s::PX_8;
    let actions = bounds.width() - s::PX_48 - actions_width;
    let mut x = s::PX_48;
    for key in ["New backup", "Open backup folder", "Reset all settings"] {
        let w = width(ui, &l.t(key), s::PX_15) + s::PX_36;
        if button(ui, rect(x, s::PX_204 - o, w, s::PX_33), &l.t(key)) && !state.busy {
            match key {
                "New backup" => {
                    state.dialog = Some(Dialog::Create);
                    state.title.clear();
                }
                "Reset all settings" => state.dialog = Some(Dialog::Reset),
                _ => state.launch(ui.ctx(), || {
                    system::open_backups_folder()?;
                    Ok(Update::Done(String::new()))
                }),
            }
        }
        x += w + s::PX_10;
    }
    for (x, key) in [
        (s::PX_56, "Title"),
        (s::PX_272, "Date created"),
        (actions, "Actions"),
    ] {
        ui.painter().text(
            pos2(x, s::PX_274 - o),
            Align2::LEFT_TOP,
            l.t(key),
            t::bold(s::PX_15),
            t::TEXT,
        );
    }
    ui.painter().hline(
        s::PX_48..=bounds.width() - s::PX_48,
        s::PX_306_5 - o,
        Stroke::new(s::PX_1, t::LINE),
    );
    if state.backups.is_empty() {
        label(ui, l, s::PX_56, s::PX_321 - o, "No backups yet.", s::PX_15);
        ui.painter().hline(
            s::PX_48..=bounds.width() - s::PX_48,
            s::PX_352_5 - o,
            Stroke::new(s::PX_1, t::LINE),
        );
    }
    let mut y = s::PX_321 - o;
    for b in &state.backups {
        let row_height = backup_row_height(ui, &b.title);
        paragraph(
            ui,
            rect(s::PX_56, y, s::BACKUP_TITLE_WIDTH, row_height),
            &b.title,
            s::PX_15,
            t::TEXT,
        );
        text(
            ui,
            s::PX_272,
            y,
            &system::backup_date(b.created, &state.language),
            s::PX_15,
            t::TEXT,
        );
        let mut x = actions;
        for key in ["Restore", "Delete"] {
            let value = l.t(key);
            let w = width(ui, &value, s::PX_15) + s::PX_36;
            if button(ui, rect(x, y - s::PX_4, w, s::PX_33), &value) {
                state.dialog = Some(if key == "Restore" {
                    Dialog::Restore(b.filename.clone())
                } else {
                    Dialog::Delete(b.filename.clone())
                });
            }
            x += w + s::PX_8;
        }
        ui.painter().hline(
            s::PX_48..=bounds.width() - s::PX_48,
            y + row_height - s::PX_12,
            Stroke::new(s::PX_1, t::LINE),
        );
        y += row_height;
    }
}
fn select_overlay(ui: &mut Ui, l: &mut Locale, state: &mut State) {
    let entries = LANGUAGES;
    let count = entries.len();
    let delta = ui.input(|i| {
        i.key_pressed(egui::Key::ArrowDown) as isize - i.key_pressed(egui::Key::ArrowUp) as isize
    });
    if delta != 0 {
        state.active_option =
            (state.active_option as isize + delta).rem_euclid(count as isize) as usize;
    }
    if ui.input(|i| i.key_pressed(egui::Key::Home)) {
        state.active_option = 0;
    }
    if ui.input(|i| i.key_pressed(egui::Key::End)) {
        state.active_option = count - 1;
    }
    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space));
    let field_y = state.field_y;
    let h = (entries.len() as f32 * s::PX_33_6 + s::PX_10).min(s::PX_320);
    let below = crate::widgets::view(ui).bottom() - field_y - s::PX_36 - s::PX_8;
    let y = if below >= h || below >= field_y - s::PX_8 {
        field_y + s::PX_40
    } else {
        field_y - h - s::PX_4
    };
    let window = crate::widgets::window(ui);
    let layer = crate::widgets::layer(
        ui,
        egui::Order::Foreground,
        egui::Id::new("settings-options"),
    );
    egui::Area::new(layer.id)
        .order(layer.order)
        .constrain(false)
        .fixed_pos(pos2(s::PX_48, y))
        .show(ui.ctx(), |ui| {
            ui.set_clip_rect(window);
            crate::widgets::shifted_access(ui);
            let r = rect(s::PX_48, y, s::PX_200, h);
            shadow(
                ui,
                r,
                s::MENU_RADIUS,
                s::SELECT_SHADOW_Y,
                s::SELECT_SHADOW_BLUR,
                t::settings_layout::SELECT_SHADOW,
            );
            ui.painter().rect(
                r,
                s::MENU_RADIUS,
                t::PANEL,
                Stroke::new(s::PX_1, t::LINE),
                StrokeKind::Inside,
            );
            ui.spacing_mut().item_spacing.y = s::PX_0;
            ui.scope_builder(egui::UiBuilder::new().max_rect(r.shrink(s::PX_5)), |ui| {
                let output = egui::ScrollArea::vertical()
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .max_height(h - s::PX_10)
                    .show(ui, |ui| {
                        for (index, &(key, value)) in entries.iter().enumerate() {
                            let (r, response) = ui.allocate_exact_size(
                                vec2(s::PX_190, s::PX_33_6),
                                egui::Sense::click(),
                            );
                            response
                                .clone()
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            let selected = state.language == key;
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, value)
                            });
                            if response.hovered()
                                && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                            {
                                state.active_option = index;
                            }
                            if index == state.active_option {
                                ui.painter().rect_filled(
                                    r,
                                    t::CONTROL_RADIUS,
                                    t::settings_layout::ACTIVE,
                                );
                            }
                            text(
                                ui,
                                r.left() + s::PX_12,
                                r.top() + s::PX_7,
                                value,
                                s::PX_14,
                                t::TEXT,
                            );
                            if selected {
                                egui::Image::new(egui::include_image!(
                                    "../../../../assets/icons/select-check.svg"
                                ))
                                .paint_at(
                                    ui,
                                    rect(
                                        r.right() - s::PX_24,
                                        r.center().y - s::PX_7,
                                        s::PX_14,
                                        s::PX_14,
                                    ),
                                );
                            }
                            if delta != 0 && index == state.active_option {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.clicked() || (enter && state.active_option == index) {
                                state.select = None;
                                state.language = key.into();
                                *l = Locale::new(key);
                                let language = key.to_owned();
                                state.launch(ui.ctx(), move || {
                                    std::fs::write(
                                        tablet_config::config_path().with_file_name("language"),
                                        language,
                                    )
                                    .map_err(|e| e.to_string())?;
                                    Ok(Update::Done(String::new()))
                                });
                            }
                        }
                    });
                crate::widgets::area_indicator(ui, &output);
            });
        });
    if ui.input(|i| i.key_pressed(egui::Key::Escape))
        || ui.input(|i| i.pointer.any_pressed())
            && crate::widgets::press(ui).is_some_and(|p| {
                !rect(s::PX_48, y, s::PX_200, h).contains(p)
                    && !rect(s::PX_48, field_y, s::PX_200, s::PX_36).contains(p)
            })
    {
        state.select = None;
    }
}
pub fn dialog(ui: &mut Ui, l: &Locale, c: &mut Config, state: &mut State) {
    let Some(d) = state.dialog.clone() else {
        return;
    };
    let (title, action) = match &d {
        Dialog::Create => ("New backup", "Create"),
        Dialog::Reset => (
            "Reset all settings to their defaults?",
            "Reset all settings",
        ),
        Dialog::Restore(_) => (
            "Restore this backup? Current settings will be replaced.",
            "Restore",
        ),
        Dialog::Delete(_) => ("Delete this backup? This cannot be undone.", "Delete"),
    };
    if !state.dialog_focused {
        state.previous_focus = ui.memory(|m| m.focused());
    }
    let bounds = crate::widgets::window(ui);
    let layer = crate::widgets::layer(
        ui,
        egui::Order::Foreground,
        egui::Id::new("settings-confirmation"),
    );
    egui::Area::new(layer.id)
        .order(layer.order)
        .constrain(false)
        .fixed_pos(bounds.min)
        .show(ui.ctx(), |ui| {
            ui.set_clip_rect(bounds);
            crate::widgets::shifted_access(ui);
            ui.allocate_rect(bounds, egui::Sense::click());
            ui.painter()
                .rect_filled(bounds, t::SQUARE, t::settings_layout::BACKDROP);
            let title_galley =
                ui.fonts_mut(|f| f.layout(l.t(title), t::medium(s::PX_17_55), t::TEXT, s::PX_332));
            let h = if matches!(d, Dialog::Create) {
                s::PX_190
            } else {
                s::PX_95 + title_galley.rows.len() as f32 * s::PX_24_57
            };
            let r = egui::Rect::from_center_size(bounds.center(), vec2(s::PX_380, h));
            shadow(
                ui,
                r,
                t::CARD_RADIUS,
                s::DIALOG_SHADOW_Y,
                s::DIALOG_SHADOW_BLUR,
                t::settings_layout::DIALOG_SHADOW,
            );
            ui.painter().rect_filled(r, t::CARD_RADIUS, t::PANEL);
            let mut job = egui::text::LayoutJob::simple(
                l.t(title),
                t::medium(s::PX_17_55),
                t::TEXT,
                s::PX_332,
            );
            job.wrap.max_width = s::PX_332;
            for section in &mut job.sections {
                section.format.line_height = Some(s::PX_24_57);
            }
            let g = ui.fonts_mut(|f| f.layout_job(job));
            ui.painter()
                .galley(r.min + vec2(s::PX_24, s::PX_22), g, t::TEXT);
            let mut focus_ids = Vec::new();
            if matches!(d, Dialog::Create) {
                label(
                    ui,
                    l,
                    r.left() + s::PX_24,
                    r.top() + s::PX_55,
                    "Backup title",
                    s::PX_15,
                );
                let input = crate::widgets::text_field(
                    ui,
                    rect(r.left() + s::PX_24, r.top() + s::PX_82, s::PX_332, s::PX_36),
                    &mut state.title,
                    "",
                    120,
                );
                if !state.dialog_focused {
                    input.request_focus();
                }
                focus_ids.push(input.id);
            }
            let action = l.t(action);
            let w = width(ui, &action, s::PX_15) + s::PX_36;
            let cancel_text = l.t("Cancel");
            let cancel_w = width(ui, &cancel_text, s::PX_15) + s::PX_36;
            let cancel = modal_button(
                ui,
                rect(
                    r.right() - w - cancel_w - s::PX_32,
                    r.bottom() - s::PX_55,
                    cancel_w,
                    s::PX_33,
                ),
                &cancel_text,
                false,
            );
            let accept = modal_button(
                ui,
                rect(r.right() - w - s::PX_24, r.bottom() - s::PX_55, w, s::PX_33),
                &action,
                true,
            );
            if !state.dialog_focused && !matches!(d, Dialog::Create) {
                cancel.request_focus();
            }
            state.dialog_focused = true;
            focus_ids.extend([cancel.id, accept.id]);
            if ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                    || i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)
            }) {
                let current = focus_ids
                    .iter()
                    .position(|id| ui.memory(|m| m.has_focus(*id)))
                    .unwrap_or(0);
                let step = if ui.input(|i| i.modifiers.shift) {
                    focus_ids.len() - 1
                } else {
                    1
                };
                ui.memory_mut(|m| m.request_focus(focus_ids[(current + step) % focus_ids.len()]));
            }
            if cancel.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.dialog = None;
            }
            if (accept.clicked()
                || (matches!(d, Dialog::Create) && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && (!matches!(d, Dialog::Create) || !state.title.trim().is_empty())
                && !state.busy
            {
                state.confirm_dialog(ui.ctx(), c, d);
            }
        });
    if state.dialog.is_none() {
        if let Some(id) = state.previous_focus.take() {
            ui.memory_mut(|m| m.request_focus(id));
        }
    }
}
pub fn toast(ui: &mut Ui, state: &mut State) {
    if let Some((message, time)) = &state.toast {
        if time.elapsed() > Duration::from_millis(s::TOAST_MS) {
            state.toast = None;
            return;
        }
        ui.ctx().request_repaint_after(
            Duration::from_millis(s::TOAST_MS).saturating_sub(time.elapsed()),
        );
        let w = width(ui, message, s::PX_15) + s::PX_36;
        let bounds = crate::widgets::view(ui);
        let r = rect(
            (bounds.width() - w) / s::PX_2,
            bounds.bottom() - s::PX_63,
            w,
            s::PX_39,
        );
        shadow(
            ui,
            r,
            t::RADIUS,
            s::TOAST_SHADOW_Y,
            s::TOAST_SHADOW_BLUR,
            t::settings_layout::TOAST_SHADOW,
        );
        ui.painter().rect_filled(r, t::RADIUS, t::RAISED);
        text(
            ui,
            r.left() + s::PX_18,
            r.top() + s::PX_9,
            message,
            s::PX_15,
            t::TEXT,
        );
    }
}

fn paragraph(ui: &Ui, r: egui::Rect, value: &str, size: f32, color: egui::Color32) {
    let mut job = egui::text::LayoutJob::simple(value.into(), t::font(size), color, r.width());
    for section in &mut job.sections {
        section.format.line_height = Some(size * s::PX_1_45);
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(r.min, galley, color);
}
fn shadow(ui: &Ui, r: egui::Rect, radius: u8, y: i8, blur: u8, color: egui::Color32) {
    ui.painter().add(
        egui::epaint::Shadow {
            offset: [0, y],
            blur,
            spread: 0,
            color,
        }
        .as_shape(r, radius),
    );
}
fn modal_button(ui: &mut Ui, r: egui::Rect, label: &str, primary: bool) -> egui::Response {
    let response = click(ui, r, ("modal", label), label);
    ui.painter().rect_filled(
        r,
        t::CONTROL_RADIUS,
        if primary {
            t::ACCENT
        } else {
            crate::widgets::hover_color(&response, t::RAISED, t::interaction::RAISED_HOVER)
        },
    );
    ui.painter().text(
        r.center() + vec2(s::PX_0, -s::PX_1),
        Align2::CENTER_CENTER,
        label,
        t::font(s::PX_15),
        t::TEXT,
    );
    response
}

fn connected_device(ui: &Ui, device: &tablet_config::DeviceInfo, offset: f32) {
    let (name, model) = crate::shared::model_name(device.product).unwrap_or(("Tablet", ""));
    let height = if crate::shared::intuos_art(device.product) {
        s::PX_77
    } else {
        s::PX_96 * s::PX_360 / s::PX_560
    };
    let area = rect(s::PX_48, s::PX_244 - offset, s::PX_96, height);
    if !crate::shared::thumbnail(ui, device.product, area) {
        crate::shared::tablet_art(ui, device.product, "tablet", area);
    }
    let y = s::PX_244 - offset + (height - s::PX_19) / s::PX_2;
    text(ui, s::PX_160, y, name, s::PX_15, t::TEXT);
    text(
        ui,
        s::PX_160 + width(ui, name, s::PX_15) + s::PX_24,
        y,
        model,
        s::PX_15,
        t::MUTED,
    );
}

fn backup_row_height(ui: &Ui, title: &str) -> f32 {
    let galley = ui.fonts_mut(|f| {
        f.layout(
            title.into(),
            t::font(s::PX_15),
            t::TEXT,
            s::BACKUP_TITLE_WIDTH,
        )
    });
    (galley.rows.len() as f32 * s::BACKUP_LINE_HEIGHT).max(s::PX_33) + s::PX_24
}
