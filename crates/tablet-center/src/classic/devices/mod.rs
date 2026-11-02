//! Device overview, ExpressKeys, mapping and device diagnostics.
use crate::theme::about_layout as ab;
use crate::theme::app_strip as a;
use crate::theme::devices_layout as d;
use crate::theme::overview_layout as ov;
use crate::theme::pen_layout as p;
use crate::theme::run_dialog as rd;
use crate::theme::toggle_dialog as tg;
use crate::{
    assets,
    i18n::Locale,
    theme as t,
    widgets::{card, click, heading, rect, text},
};
use egui::{pos2, vec2, Align2, Stroke, StrokeKind, Ui};
use tablet_config::{Config, Orientation, Response};

mod about;
mod actions;
mod apps;
mod mapping;
mod overview;

use about::*;
pub use actions::*;
pub(crate) use apps::*;
use mapping::*;
use overview::*;
/// `State::selected` for pen controls; ExpressKeys use 0..count (up to 8).
pub const PEN_UPPER: usize = 100;
pub const PEN_LOWER: usize = 101;
pub const PEN_TIP: usize = 102;
pub fn is_pen(index: usize) -> bool {
    index >= PEN_UPPER
}
/// Icon art loaded in the background, by `Icon=` value.
type LoadedIcons = Vec<(String, Option<crate::system::IconData>)>;
#[derive(Default)]
pub struct State {
    pub selected: Option<usize>,
    pub ring_open: Option<usize>,
    pub menu: Option<tablet_config::Shortcut>,
    pub app: Option<usize>,
    pub add_app: bool,
    /// Action groups whose open state differs from the default (only the first open).
    pub toggled_groups: std::collections::BTreeSet<usize>,
    pub action_scroll: f32,
    pub action_dialog: Option<String>,
    pub command: String,
    pub modifiers: [bool; 4],
    pub modifier_click: usize,
    pub speed: u8,
    pub acceleration: u8,
    pub precision_scale: u8,
    pub precision_overlay: bool,
    pub pending_orientation: Option<Orientation>,
    pub remove_app: bool,
    /// Horizontal scroll of the application strip, in points.
    pub app_offset: f32,
    pub capture: bool,
    pub captured: String,
    pub keys: Vec<String>,
    pub search: String,
    pub native: bool,
    pub installed: bool,
    pub app_choice: Option<usize>,
    /// Scroll offset of the app picker list.
    pub app_list_scroll: f32,
    /// The Display toggle help dialog is open.
    pub toggle_help: bool,
    /// Scroll offset of the Open/Run apps list.
    pub run_list_scroll: f32,
    /// The actions panel opens on the right (pen page) instead of the left.
    pub panel_right: bool,
    /// The right-hand panel edits ExpressKeys rather than pen buttons.
    pub panel_keys: bool,
    /// Pen button the panel showed last, drawn while it slides out.
    pub panel_last: Option<usize>,
    pub app_source: bool,
    pub installed_apps: Vec<tablet_config::AppInfo>,
    pub running_apps: Vec<tablet_config::AppInfo>,
    /// App icon art by `Icon=` value; None while loading or when there is none.
    pub app_icons: std::collections::HashMap<String, Option<crate::system::IconData>>,
    pub icon_loading: Option<std::sync::mpsc::Receiver<LoadedIcons>>,
    pub loading: Option<
        std::sync::mpsc::Receiver<(Vec<tablet_config::AppInfo>, Vec<tablet_config::AppInfo>)>,
    >,
}
impl State {
    /// Groups open independently; opening one leaves the others as they are.
    pub fn group_open(&self, index: usize) -> bool {
        (index == 0) != self.toggled_groups.contains(&index)
    }
}
pub fn button(ui: &mut Ui, r: egui::Rect, label: &str) -> bool {
    let response = click(ui, r, (label, r.min.x.to_bits(), r.min.y.to_bits()), label);
    ui.painter().rect_filled(
        r,
        t::CONTROL_RADIUS,
        crate::widgets::hover_color(&response, t::RAISED, t::interaction::RAISED_HOVER),
    );
    ui.painter().text(
        r.center() + vec2(0., -1.),
        Align2::CENTER_CENTER,
        label,
        t::font(d::PX_15),
        t::TEXT,
    );
    response.clicked()
}
fn title(ui: &Ui, l: &Locale, s: &str) {
    ui.painter().text(
        pos2(d::PX_289, d::PX_171),
        Align2::LEFT_TOP,
        l.t(s),
        t::bold(d::PX_30),
        t::TEXT,
    );
}
/// Index of the app profile the daemon applies to the focused window, as app.js
/// matches it: the full id or its last dotted part, without `.desktop`.
pub fn active_app(c: &Config, focused: Option<&str>) -> Option<usize> {
    let bare = |id: &str| {
        let lower = id.to_lowercase();
        lower
            .strip_suffix(".desktop")
            .map(String::from)
            .unwrap_or(lower)
    };
    let focused = bare(focused?);
    c.apps.iter().position(|app| {
        let id = bare(&app.id);
        id == focused || id.rsplit('.').next() == Some(focused.as_str())
    })
}
/// Name of the app profile in use for the focused window, if it has one.
pub fn active_profile<'a>(c: &'a Config, focused: Option<&str>) -> Option<&'a str> {
    active_app(c, focused).map(|i| c.apps[i].name.as_str())
}
/// Scroll arrow at `r`; returns true when clicked.
fn strip_arrow(ui: &mut Ui, r: egui::Rect, icon: &str, label: &str, enabled: bool) -> bool {
    let response = ui
        .add_enabled_ui(enabled, |ui| click(ui, r, label, label))
        .inner;
    if response.hovered() {
        ui.painter().rect_filled(r, t::RADIUS, t::RAISED);
    }
    let color = if enabled {
        crate::widgets::hover_color(&response, t::MUTED, t::TEXT)
    } else {
        t::FAINT
    };
    egui::Image::new(crate::icons::source(icon))
        .tint(color)
        .paint_at(
            ui,
            egui::Rect::from_center_size(r.center(), vec2(a::ARROW_ICON, a::ARROW_ICON)),
        );
    enabled && response.clicked()
}
pub fn strip(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State, focused: Option<&str>) {
    let w = crate::widgets::view(ui).right();
    let shift = if s.selected.is_some() && !s.panel_right {
        d::PX_230
    } else {
        0.
    };
    let r = rect(
        d::PX_229 + shift,
        d::PX_138 + a::TOP_SHIFT,
        w - d::PX_257 - shift,
        d::PX_54,
    );
    card(ui, r);
    heading(
        ui,
        d::PX_242 + shift,
        d::PX_158 + a::TOP_SHIFT,
        &l.t("Applications"),
    );
    let active = active_app(c, focused).map_or(0, |i| i + 1);
    let add = l.t("Add");
    // Pill widths: "All other", each app (with room for its remove button), Add.
    let labels: Vec<String> = std::iter::once(l.t("All other"))
        .chain(c.apps.iter().map(|app| app.name.clone()))
        .collect();
    let widths: Vec<f32> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let text = crate::widgets::text_width(ui, label, d::PX_14);
            if i == 0 {
                (d::PX_35 + text + d::PX_12).max(d::PX_118)
            } else {
                (d::PX_35 + text + a::CLOSE_ROOM).max(d::PX_118)
            }
        })
        .collect();
    let add_width = a::ADD_PAD_LEFT
        + a::ADD_ICON
        + a::ADD_GAP
        + crate::widgets::text_width(ui, &add, d::PX_14)
        + a::ADD_PAD_RIGHT;
    let total = widths.iter().map(|w| w + a::GAP).sum::<f32>() + add_width;
    // Pills scroll between two arrows once they no longer fit.
    let left = d::PX_348 + shift;
    let right = r.right() - a::PAD;
    let overflow = total > right - left;
    let view = if overflow {
        egui::Rect::from_x_y_ranges(
            left + a::ARROW + a::GAP..=right - a::ARROW - a::GAP,
            r.y_range(),
        )
    } else {
        egui::Rect::from_x_y_ranges(left..=right, r.y_range())
    };
    let max_offset = (total - view.width()).max(0.);
    if crate::widgets::pointer(ui).is_some_and(|p| view.contains(p)) {
        let delta = ui.input(|i| i.smooth_scroll_delta);
        s.app_offset -= delta.x + delta.y;
    }
    if overflow {
        let arrow = |x: f32| rect(x, d::PX_150 + a::TOP_SHIFT, a::ARROW, d::PX_30);
        if strip_arrow(
            ui,
            arrow(left),
            "chevron-left",
            "Previous",
            s.app_offset > 0.,
        ) {
            s.app_offset -= a::STEP;
        }
        if strip_arrow(
            ui,
            arrow(right - a::ARROW),
            "chevron-right",
            "Next",
            s.app_offset < max_offset,
        ) {
            s.app_offset += a::STEP;
        }
    }
    s.app_offset = s.app_offset.clamp(0., max_offset);
    let offset =
        ui.ctx()
            .animate_value_with_time(ui.id().with("app-offset"), s.app_offset, a::SCROLL_TIME);
    want_icons(ui, s, c.apps.iter().filter_map(|app| app.icon.as_deref()));
    let mut row = ui.new_child(egui::UiBuilder::new().max_rect(view));
    row.set_clip_rect(view);
    let ui = &mut row;
    let mut x = view.left() - offset;
    for (i, (label, chip)) in labels.iter().zip(&widths).enumerate() {
        let rr = rect(x, d::PX_147 + a::TOP_SHIFT, *chip, d::PX_36);
        x += chip + a::GAP;
        let response = click(ui, rr, ("app", i), label);
        let close = (i > 0).then(|| {
            click(
                ui,
                egui::Rect::from_center_size(
                    pos2(rr.right() - a::CLOSE_INSET, rr.center().y),
                    vec2(a::CLOSE, a::CLOSE),
                ),
                ("remove-app", i),
                "Remove application",
            )
            .on_hover_text(l.t("Remove application"))
        });
        let close_hovered = close.as_ref().is_some_and(|c| c.hovered());
        let selected = s.app == i.checked_sub(1);
        ui.painter().rect(
            rr,
            t::PILL_RADIUS,
            if selected {
                t::CHIP
            } else if response.hovered() || close_hovered {
                t::RAISED
            } else {
                t::PANEL
            },
            Stroke::new(1., if selected { t::ACCENT } else { t::LINE }),
            StrokeKind::Inside,
        );
        if i == 0 {
            for xx in [rr.left() + d::PX_12, rr.left() + d::PX_21] {
                for y in [d::PX_158 + a::TOP_SHIFT, d::PX_167 + a::TOP_SHIFT] {
                    ui.painter()
                        .rect_filled(rect(xx, y, d::PX_6, d::PX_6), 0, t::TEXT);
                }
            }
        }
        if i > 0 {
            paint_app_icon(
                ui,
                &s.app_icons,
                c.apps[i - 1].icon.as_deref(),
                label,
                egui::Rect::from_center_size(
                    pos2(rr.left() + d::PX_12 + a::PILL_ICON / 2., rr.center().y),
                    vec2(a::PILL_ICON, a::PILL_ICON),
                ),
            );
        }
        text(
            ui,
            rr.left() + d::PX_35,
            d::PX_156 + a::TOP_SHIFT,
            label,
            d::PX_14,
            t::TEXT,
        );
        if let Some(close) = &close {
            if response.hovered() || close.hovered() {
                if close.hovered() {
                    ui.painter()
                        .circle_filled(close.rect.center(), a::CLOSE / 2., t::TRACK);
                }
                egui::Image::new(crate::icons::source("x"))
                    .tint(crate::widgets::hover_color(close, t::MUTED, t::TEXT))
                    .paint_at(
                        ui,
                        egui::Rect::from_center_size(
                            close.rect.center(),
                            vec2(a::CLOSE_ICON, a::CLOSE_ICON),
                        ),
                    );
            }
            if close.clicked() {
                s.app = i.checked_sub(1);
                s.selected = None;
                s.remove_app = true;
            }
        }
        if i == active {
            let at = pos2(rr.right() - a::BADGE_X, rr.top() + a::BADGE_Y);
            ui.painter()
                .circle_filled(at, a::BADGE + a::BADGE_RING, t::PANEL);
            ui.painter().circle_filled(at, a::BADGE, t::GREEN);
        }
        let response = if i == active {
            response.on_hover_text(format!(
                "{}: {}",
                l.t("Application in use now"),
                focused
                    .map(String::from)
                    .unwrap_or_else(|| l.t("All other"))
            ))
        } else {
            response
        };
        if response.clicked() {
            s.app = i.checked_sub(1);
            s.selected = None;
        }
    }
    let rr = rect(x, d::PX_147 + a::TOP_SHIFT, add_width, d::PX_36);
    let response = click(ui, rr, "add", "Add application").on_hover_text(l.t("Add application"));
    ui.painter().rect(
        rr,
        t::PILL_RADIUS,
        crate::widgets::hover_color(&response, t::PANEL, t::RAISED),
        Stroke::new(1., t::LINE),
        StrokeKind::Inside,
    );
    let color = crate::widgets::hover_color(&response, t::MUTED, t::TEXT);
    egui::Image::new(crate::icons::source("plus"))
        .tint(color)
        .paint_at(
            ui,
            egui::Rect::from_center_size(
                pos2(
                    rr.left() + a::ADD_PAD_LEFT + a::ADD_ICON / 2.,
                    rr.center().y,
                ),
                vec2(a::ADD_ICON, a::ADD_ICON),
            ),
        );
    text(
        ui,
        rr.left() + a::ADD_PAD_LEFT + a::ADD_ICON + a::ADD_GAP,
        rr.top() + a::ADD_TEXT_Y,
        &add,
        d::PX_14,
        color,
    );
    if response.clicked() {
        s.add_app = true;
        s.installed = false;
        s.app_list_scroll = 0.;
    }
}
fn radio(ui: &mut Ui, x: f32, y: f32, label: &str, on: bool) -> bool {
    ui.painter().circle_filled(
        pos2(x + d::PX_10, y + d::PX_10),
        d::PX_10,
        if on { t::ACCENT } else { t::TRACK },
    );
    if on {
        ui.painter()
            .circle_filled(pos2(x + d::PX_10, y + d::PX_10), d::PX_4, t::TEXT);
    }
    text(ui, x + d::PX_30, y + 1., label, d::PX_15, t::TEXT);
    click(
        ui,
        rect(x, y, d::PX_300, d::PX_23),
        (label, y.to_bits()),
        label,
    )
    .clicked()
}
pub fn switch(ui: &mut Ui, x: f32, y: f32, on: &mut bool, label: &str) -> bool {
    let progress =
        crate::widgets::switch_progress(ui, (label, y.to_bits()), *on, t::interaction::SWITCH_TIME);
    ui.painter().rect_filled(
        rect(x, y, d::PX_52, d::PX_30),
        15,
        egui::Color32::from(egui::lerp(
            egui::Rgba::from(t::TRACK)..=egui::Rgba::from(t::ACCENT),
            progress,
        )),
    );
    ui.painter().circle_filled(
        pos2(
            x + d::PX_14 + (d::PX_38 - d::PX_14) * progress,
            y + d::PX_15,
        ),
        d::PX_12,
        t::THUMB,
    );
    text(ui, x + d::PX_64, y + d::PX_5, label, d::PX_15, t::MUTED);
    let changed = click(ui, rect(x, y, d::PX_210, d::PX_30), label, label).clicked();
    if changed {
        *on = !*on;
    }
    changed
}
pub fn show(
    ui: &mut Ui,
    l: &Locale,
    c: &mut Config,
    status: &Response,
    page: usize,
    s: &mut State,
) -> bool {
    let before = c.clone();
    let h = crate::widgets::view(ui).bottom();
    let w = crate::widgets::view(ui).right();
    // ExpressKeys edits its keys in the right-hand panel, like the pen page.
    s.panel_right = page == 1;
    s.panel_keys = page == 1;
    match page {
        1 => {
            strip(ui, l, c, s, status.focused_app.as_deref());
            // The tablet with a callout card per key on its right;
            // the actions panel slides in on the right and pushes them left.
            let slide = super::pen::panel_progress(ui.ctx(), s.selected.is_some());
            let product = status.device.as_ref().map_or(0x0374, |dev| dev.product);
            let count = tablet_config::express_key_count(product).max(1);
            let art = keys_art_rect(product, w, h, slide);
            let top = art.top();
            crate::shared::tablet_art(ui, product, "tablet", art);
            if let Some((x, y)) = crate::shared::ring_spot(product) {
                let center = pos2(art.left() + x * art.width(), art.top() + y * art.height());
                let hit = egui::Rect::from_center_size(
                    center,
                    vec2(art.width(), art.width()) * t::device_art::RING_HIT_RATIO,
                );
                ui.interact(hit, ui.id().with("ring-center"), egui::Sense::hover())
                    .on_hover_text(l.t("Press the centre button to switch modes. The lit LED shows the active mode."));
            }
            let keys = if let Some(i) = s.app {
                &mut c.apps[i].express_keys
            } else {
                &mut c.express_keys
            };
            keys.resize(keys.len().max(count), tablet_config::KeyAction::Disabled);
            let spots: Vec<egui::Pos2> = (0..count)
                .map(|i| {
                    let (sx, sy) = crate::shared::key_spot(product, i, count);
                    pos2(art.left() + sx * art.width(), art.top() + sy * art.height())
                })
                .collect();
            let labels: Vec<_> = (0..count)
                .map(|i| super::callout::label(ui, &l.t(&action_label(&keys[i]))))
                .collect();
            let heights: Vec<f32> = labels.iter().map(|g| super::callout::height(g)).collect();
            let mut tops = if count > 6 && crate::shared::ring_spot(product).is_some() {
                super::callout::stack_with_gap(
                    &spots,
                    &heights,
                    top,
                    h - t::CONTENT_PAD,
                    t::SPACE_4,
                )
            } else {
                super::callout::stack(&spots, &heights, top, h - t::CONTENT_PAD)
            };
            if count > 6 && crate::shared::ring_spot(product).is_some() {
                let mut bottom = h - t::CONTENT_PAD;
                for i in (0..count).rev() {
                    tops[i] = tops[i].min(bottom - heights[i]);
                    bottom = tops[i] - t::SPACE_4;
                }
            }
            for i in 0..count {
                let r = rect(
                    art.right() + p::CALLOUT_REACH,
                    tops[i],
                    p::CALLOUT_WIDTH,
                    heights[i],
                );
                let icon = super::shortcuts::item_icon(&tablet_config::MenuItem {
                    label: String::new(),
                    action: keys[i].clone(),
                });
                // Keys share one row, so their legs leave level with the card.
                if super::callout::card(
                    ui,
                    r,
                    spots[i],
                    pos2(art.right() + p::CALLOUT_BEND, r.center().y),
                    &format!("{} {}", l.t("ExpressKey"), i + 1),
                    &icon,
                    labels[i].clone(),
                    s.selected == Some(i),
                    status.express_keys.get(i).copied().unwrap_or(false),
                ) {
                    s.selected = if s.selected == Some(i) { None } else { Some(i) };
                }
            }
            if link_button(
                ui,
                rect(
                    art.left(),
                    art.bottom() + p::KEYS_DEFAULT_GAP,
                    d::PX_145,
                    d::PX_32,
                ),
                &l.t("Default"),
            ) {
                *keys = vec![tablet_config::KeyAction::Disabled; keys.len()];
            }
        }
        8 => touch_ring(ui, l, c, status, s),
        2 => mapping(ui, l, c, status),
        3 => {
            let scale = (h * d::TOGGLE_HEIGHT_VH / d::PX_300).min(1.);
            toggle(ui, l, d::PX_650, d::PX_168, scale, c, status, s.native);
        }
        4 => {
            title(ui, l, "About this device");
            about(ui, l, w, status);
        }
        5 => {
            title(ui, l, "Touch");
            switch(
                ui,
                d::PX_289,
                d::PX_242,
                &mut c.touch.enabled,
                &l.t("Enable touch"),
            );
            wrapped(ui,rect(d::PX_289,d::PX_288,d::PX_480,d::PX_70),&l.t("Use your fingers on the screen. Turn touch off to rest your hand on the screen while you draw."),d::PX_15,t::TEXT);
        }
        _ => overview(ui, l, c, status, s),
    }
    overlays(ui, l, c, status, s);
    s.panel_right = false;
    s.panel_keys = false;
    *c != before
}
fn backdrop(ui: &mut Ui, id: &str, r: egui::Rect) -> Ui {
    let mut child = ui.new_child(egui::UiBuilder::new().layer_id(crate::widgets::layer(
        ui,
        egui::Order::Tooltip,
        egui::Id::new(id),
    )));
    let window = crate::widgets::window(ui);
    child.set_clip_rect(window);
    child
        .painter()
        .rect_filled(window, t::WINDOW_RADIUS, t::DIM);
    child.interact(window, child.id().with("modal"), egui::Sense::click());
    child.painter().add(
        egui::epaint::Shadow {
            offset: [0, 20],
            blur: 50,
            spread: 0,
            color: t::DIALOG_SHADOW,
        }
        .as_shape(r, t::CARD_RADIUS),
    );
    child.painter().rect_filled(r, t::CARD_RADIUS, t::PANEL);
    crate::widgets::mark_modal(ui.ctx());
    child
}
pub fn welcome(ui: &mut Ui, l: &Locale, error: bool) {
    let r = crate::widgets::view(ui);
    let h = if error { d::PX_245 } else { d::PX_218 };
    let y = d::PX_116 + (r.bottom() - d::PX_116 - h) / d::PX_2;
    ui.painter()
        .rect_filled(rect(1., y, r.width() - d::PX_2, h), 0, t::RAISED);
    for (dy, label, size) in [
        (d::PX_40, l.t("Welcome to"), d::PX_26),
        (d::PX_80, "tablet Center".into(), d::PX_52),
        (
            d::PX_151,
            l.t("Connect a device to see its settings here."),
            d::PX_17,
        ),
    ] {
        ui.painter().text(
            pos2(r.center().x, y + dy),
            Align2::CENTER_TOP,
            label,
            t::font(size),
            t::TEXT,
        );
    }
    if error {
        ui.painter().text(
            pos2(r.center().x, y + d::PX_187),
            Align2::CENTER_TOP,
            l.t("The tablet driver is not responding."),
            t::font(d::PX_15),
            t::MUTED,
        );
    }
}
fn segment(ui: &mut Ui, r: egui::Rect, label: &str, on: bool) -> bool {
    let response = click(ui, r, label, label);
    if on {
        ui.painter().rect_filled(r, 6, t::RAISED);
    }
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        label,
        t::font(d::PX_14),
        if on {
            t::TEXT
        } else {
            crate::widgets::hover_color(&response, t::MUTED, t::TEXT)
        },
    );
    response.clicked()
}
fn primary_button(ui: &mut Ui, r: egui::Rect, label: &str, enabled: bool) -> bool {
    ui.painter().rect_filled(
        r,
        t::CONTROL_RADIUS,
        if enabled {
            t::ACCENT
        } else {
            t::DISABLED_ACCENT
        },
    );
    ui.painter().text(
        r.center() - vec2(0., 1.),
        Align2::CENTER_CENTER,
        label,
        t::font(d::PX_15),
        if enabled { t::TEXT } else { t::MUTED },
    );
    click(ui, r, ("primary", label), label).clicked() && enabled
}
fn link_button(ui: &mut Ui, r: egui::Rect, label: &str) -> bool {
    let response = click(ui, r, label, label);
    if response.hovered() {
        ui.painter().rect_filled(r, t::CONTROL_RADIUS, t::RAISED);
    }
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        label,
        t::font(d::PX_14),
        crate::widgets::hover_color(&response, t::MUTED, t::TEXT),
    );
    response.clicked()
}
