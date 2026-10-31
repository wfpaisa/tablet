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
fn touch_ring(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response, s: &mut State) {
    use tablet_config::RingAction;
    const ACTIONS: [(RingAction, &str); 6] = [
        (RingAction::Disabled, "Disabled"),
        (RingAction::Scroll, "Scroll"),
        (RingAction::Zoom, "Zoom"),
        (RingAction::BrushSize, "Brush size"),
        (RingAction::Rotate, "Rotate canvas"),
        (RingAction::UndoRedo, "Undo / redo"),
    ];
    title(ui, l, "Touch Ring");
    let view = crate::widgets::view(ui);
    let product = status.device.as_ref().map_or(0x0314, |dev| dev.product);
    let art = keys_art_rect(product, view.right(), view.bottom(), 0.);
    crate::shared::tablet_art(ui, product, "ring", art);
    let fractions = if product == 0x0314 {
        t::device_art::PRO_2013_S_RING_SPOTS
    } else {
        t::device_art::PRO_2013_ML_RING_SPOTS
    };
    let spots: Vec<_> = fractions
        .into_iter()
        .map(|(x, y)| pos2(art.left() + x * art.width(), art.top() + y * art.height()))
        .collect();
    let labels: Vec<_> = (0..4)
        .map(|i| {
            let action = c.touch_ring.get(i).copied().unwrap_or(RingAction::Disabled);
            let name = ACTIONS
                .iter()
                .find(|(a, _)| *a == action)
                .map_or("Disabled", |(_, name)| *name);
            super::callout::label(ui, &l.t(name))
        })
        .collect();
    let heights: Vec<_> = labels.iter().map(|g| super::callout::height(g)).collect();
    let block_height = heights.iter().sum::<f32>() + p::CALLOUT_GAP * 3.;
    let mut next =
        spots.iter().map(|spot| spot.y).sum::<f32>() / spots.len() as f32 - block_height / 2.;
    let tops: Vec<_> = heights
        .iter()
        .map(|height| {
            let top = next;
            next += height + p::CALLOUT_GAP;
            top
        })
        .collect();
    let mut open_card = None;
    for i in 0..4 {
        let r = rect(
            art.right() + p::CALLOUT_REACH,
            tops[i],
            p::CALLOUT_WIDTH,
            heights[i],
        );
        let name = l.t(["Mode 1", "Mode 2", "Mode 3", "Mode 4"][i]);
        let clicked = super::callout::card(
            ui,
            r,
            spots[i],
            pos2(art.right() + p::CALLOUT_BEND, r.center().y),
            &name,
            "scroll",
            labels[i].clone(),
            s.ring_open == Some(i),
            false,
        );
        if clicked {
            s.ring_open = if s.ring_open == Some(i) {
                None
            } else {
                Some(i)
            };
        }
        if s.ring_open == Some(i) {
            open_card = Some((i, r));
        }
    }
    if let Some((i, card)) = open_card {
        use t::settings_layout as menu;
        let height = ACTIONS.len() as f32 * menu::PX_33_6 + menu::PX_10;
        let y = if card.bottom() + menu::PX_4 + height <= view.bottom() {
            card.bottom() + menu::PX_4
        } else {
            card.top() - height - menu::PX_4
        };
        let r = rect(card.left(), y, p::CALLOUT_WIDTH, height);
        let window = crate::widgets::window(ui);
        let layer = crate::widgets::layer(
            ui,
            egui::Order::Foreground,
            egui::Id::new("touch-ring-options"),
        );
        egui::Area::new(layer.id)
            .order(layer.order)
            .constrain(false)
            .fixed_pos(r.min)
            .show(ui.ctx(), |ui| {
                ui.set_clip_rect(window);
                crate::widgets::shifted_access(ui);
                super::settings::shadow(
                    ui,
                    r,
                    menu::MENU_RADIUS,
                    menu::SELECT_SHADOW_Y,
                    menu::SELECT_SHADOW_BLUR,
                    menu::SELECT_SHADOW,
                );
                ui.painter().rect(
                    r,
                    menu::MENU_RADIUS,
                    t::PANEL,
                    Stroke::new(menu::PX_1, t::LINE),
                    StrokeKind::Inside,
                );
                for (index, (action, label)) in ACTIONS.into_iter().enumerate() {
                    let row = rect(
                        r.left() + menu::PX_5,
                        r.top() + menu::PX_5 + index as f32 * menu::PX_33_6,
                        r.width() - menu::PX_10,
                        menu::PX_33_6,
                    );
                    let label = l.t(label);
                    let response = click(ui, row, ("ring-action", index), &label);
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(row, t::CONTROL_RADIUS, menu::ACTIVE);
                    }
                    text(
                        ui,
                        row.left() + menu::PX_12,
                        row.top() + menu::PX_7,
                        &label,
                        menu::PX_14,
                        t::TEXT,
                    );
                    if c.touch_ring.get(i) == Some(&action) {
                        egui::Image::new(egui::include_image!(
                            "../../../../assets/icons/select-check.svg"
                        ))
                        .paint_at(
                            ui,
                            rect(
                                row.right() - menu::PX_24,
                                row.center().y - menu::PX_7,
                                menu::PX_14,
                                menu::PX_14,
                            ),
                        );
                    }
                    if response.clicked() {
                        c.touch_ring.resize(4, RingAction::Disabled);
                        c.touch_ring[i] = action;
                        s.ring_open = None;
                    }
                }
            });
        if ui.input(|input| input.key_pressed(egui::Key::Escape))
            || ui.input(|input| input.pointer.any_pressed())
                && crate::widgets::press(ui)
                    .is_some_and(|pos| !r.contains(pos) && !card.contains(pos))
        {
            s.ring_open = None;
        }
    }
    wrapped(
        ui,
        rect(
            art.left(),
            art.bottom() + p::KEYS_DEFAULT_GAP,
            view.right() - art.left() - t::CONTENT_PAD,
            d::PX_70,
        ),
        &l.t("Press the centre button to switch modes. The lit LED shows the active mode."),
        d::PX_15,
        t::MUTED,
    );
}

fn keys_art_rect(product: u16, w: f32, h: f32, slide: f32) -> egui::Rect {
    let left = t::SIDEBAR + d::PX_1 + t::CONTENT_PAD;
    let right = w - p::CONTENT_RIGHT - slide * (p::PANEL_WIDTH + p::COLUMN_GAP);
    // The tablet shrinks so its callouts stay clear of the open panel.
    let top = p::PAGE_TOP + t::SPACE_14;
    let mut art_w = (right - left - p::CALLOUT_REACH - p::CALLOUT_WIDTH)
        .clamp(p::KEYS_ART_MIN_WIDTH, p::KEYS_ART_WIDTH);
    let ratio =
        crate::shared::art_ratio(product).unwrap_or(if crate::shared::intuos_art(product) {
            p::INTUOS_ART_RATIO
        } else {
            p::KEYS_ART_RATIO
        });
    // Portrait panels shrink to leave room for the Default button.
    if crate::shared::art_ratio(product).is_some() {
        let room = h - top - p::KEYS_DEFAULT_GAP - d::PX_32 - t::CONTENT_PAD;
        art_w = art_w.min(room / ratio);
    }
    let art_h = art_w * ratio;
    let group = art_w + p::CALLOUT_REACH + p::CALLOUT_WIDTH;
    rect(
        left + ((right - left - group) / d::PX_2).max(0.),
        top,
        art_w,
        art_h,
    )
}

fn wrapped(ui: &Ui, r: egui::Rect, s: &str, size: f32, color: egui::Color32) {
    let mut job = egui::text::LayoutJob::simple(s.into(), t::font(size), color, r.width());
    for section in &mut job.sections {
        section.format.line_height = Some(
            size * if size == d::PX_19 {
                d::TOGGLE_LINE_HEIGHT
            } else {
                d::LINE_HEIGHT
            },
        );
    }
    let g = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(r.min, g, color);
}
fn overview(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response, s: &mut State) {
    let product = status.device.as_ref().map_or(0x0374, |d| d.product);
    let (name, model) = device_model(status);
    let h = crate::widgets::view(ui).bottom();
    let w = crate::widgets::view(ui).right();
    let mut clipped = ui.new_child(egui::UiBuilder::new());
    clipped.set_clip_rect(rect(d::PX_207, d::PX_116, w - d::PX_208, h - d::PX_117));
    clipped.painter().text(
        pos2(d::PX_207, d::PX_113),
        Align2::LEFT_TOP,
        name,
        t::bold(d::PX_150),
        t::WATERMARK,
    );
    title(ui, l, name);
    text(ui, d::PX_289, d::PX_215, model, d::PX_15, t::MUTED);
    // Connection: a green USB plug, or grey and "Disconnected".
    let connected = status.device.is_some();
    let model_width = crate::widgets::text_width(ui, model, d::PX_15);
    let x = d::PX_289 + model_width + ov::STATUS_GAP;
    let line = d::PX_215 + d::PX_10;
    egui::Image::new(crate::icons::source("usb"))
        .tint(if connected { t::GREEN } else { t::FAINT })
        .paint_at(
            ui,
            egui::Rect::from_center_size(
                pos2(x + ov::STATUS_ICON / 2., line),
                vec2(ov::STATUS_ICON, ov::STATUS_ICON),
            ),
        );
    ui.painter().text(
        pos2(x + ov::STATUS_ICON + ov::STATUS_TEXT_GAP, line),
        Align2::LEFT_CENTER,
        if connected {
            "USB".to_string()
        } else {
            l.t("Disconnected")
        },
        t::font(d::PX_14),
        t::MUTED,
    );
    // Device info: accent-tinted button with an info icon and a chevron.
    let info = l.t("Device info");
    let width = ov::BUTTON_PAD * 2.
        + ov::BUTTON_ICON
        + ov::BUTTON_GAP * 2.
        + crate::widgets::text_width(ui, &info, d::PX_14)
        + ov::CHEVRON;
    let r = rect(d::PX_289, ov::BUTTON_TOP, width, ov::BUTTON_HEIGHT);
    let response = click(ui, r, "info", &info);
    ui.painter().rect(
        r,
        t::CONTROL_RADIUS,
        if response.hovered() {
            t::interaction::ACCENT_TINT_HOVER
        } else {
            t::interaction::ACCENT_TINT
        },
        Stroke::new(t::BORDER, t::interaction::ACCENT_TINT_LINE),
        StrokeKind::Inside,
    );
    egui::Image::new(crate::icons::source("info"))
        .tint(t::interaction::ACCENT_SOFT)
        .paint_at(
            ui,
            rect(
                r.left() + ov::BUTTON_PAD,
                r.center().y - ov::BUTTON_ICON / 2.,
                ov::BUTTON_ICON,
                ov::BUTTON_ICON,
            ),
        );
    ui.painter().text(
        pos2(
            r.left() + ov::BUTTON_PAD + ov::BUTTON_ICON + ov::BUTTON_GAP,
            r.center().y,
        ),
        Align2::LEFT_CENTER,
        &info,
        t::font(d::PX_14),
        t::interaction::ACCENT_SOFT,
    );
    egui::Image::new(crate::icons::source("chevron-right"))
        .tint(t::interaction::ACCENT_SOFT)
        .paint_at(
            ui,
            rect(
                r.right() - ov::BUTTON_PAD - ov::CHEVRON,
                r.center().y - ov::CHEVRON / 2.,
                ov::CHEVRON,
                ov::CHEVRON,
            ),
        );
    if response.clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new("navigate"), 4usize));
    }
    let ar = rect(
        d::PX_372,
        d::PX_387,
        d::PX_400,
        d::PX_400 * d::PX_221 / d::PX_277,
    );
    let ar = tablet_art_rect(ar, product, c.mapping.orientation, true);
    let shadow = if crate::shared::device_art(product).name != "intuos-s"
        && c.mapping.orientation.is_portrait()
    {
        egui::Rect::from_center_size(ar.center(), vec2(ar.height(), ar.width()))
    } else {
        ar
    };
    ui.painter().add(
        egui::epaint::Shadow {
            offset: [0, 0],
            blur: 44,
            spread: 0,
            color: t::GLOW,
        }
        .as_shape(shadow, 18),
    );
    tablet_art(ui, ar, product, "overview", c.mapping.orientation);
    crate::shared::pen_glow(ui, product, rect(d::PX_892, d::PX_327, d::PX_36, d::PX_440));
    let (key, bytes) = crate::shared::pen_art(product);
    assets::artwork(
        ui,
        &format!("overview-{key}"),
        bytes,
        rect(d::PX_892, d::PX_327, d::PX_36, d::PX_440),
    );
    let has_keys = tablet_config::express_key_count(product) > 0;
    for (i, label) in ["ExpressKeys", "Mapping", "Display toggle", "Pen"]
        .iter()
        .enumerate()
    {
        if i == 0 && !has_keys {
            continue;
        }
        let r = if i == 3 {
            rect(d::PX_845, d::PX_669, d::PX_130, d::PX_31)
        } else {
            rect(
                if tablet_config::intuos_pro_2013(product) {
                    ar.left() + ar.width() * t::device_art::PRO_2013_ACTIVE_CENTER_X
                        - d::PX_170 / 2.
                } else {
                    d::PX_487
                },
                d::PX_492 + (i as f32 - if has_keys { 0. } else { 0.5 }) * d::PX_39,
                d::PX_170,
                d::PX_31,
            )
        };
        // Icon and label stay centered as a group; long translations widen the button.
        let caption = l.t(label);
        let group = d::PX_22 + crate::widgets::text_width(ui, &caption, d::PX_15);
        let r = egui::Rect::from_center_size(
            r.center(),
            vec2(r.width().max(group + d::PX_24), r.height()),
        );
        let icon_left = r.center().x - group / d::PX_2;
        ui.painter().rect_filled(r, 6, t::STROKE);
        egui::Image::new(assets::icon(if i == 3 { 3 } else { i + 4 }))
            .tint(t::VIEW)
            .paint_at(ui, rect(icon_left, r.top() + d::PX_8, d::PX_16, d::PX_16));
        ui.painter().text(
            pos2(icon_left + d::PX_22, r.center().y - 1.),
            Align2::LEFT_CENTER,
            caption,
            t::font(d::PX_15),
            t::VIEW,
        );
        if click(ui, r, label, label).clicked() {
            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("navigate"),
                    if i == 3 { 0usize } else { i + 1 },
                )
            });
        }
    }
    let y = d::PX_116 + (h - d::PX_116 - d::PX_234) / d::PX_2;
    ui.painter()
        .rect_filled(rect(w - d::PX_65, y, d::PX_64, d::PX_234), 14, t::PANEL);
    for (i, o) in [
        Orientation::Landscape,
        Orientation::Portrait,
        Orientation::LandscapeFlipped,
        Orientation::PortraitFlipped,
    ]
    .into_iter()
    .enumerate()
    {
        let rr = rect(
            w - d::PX_57,
            y + d::PX_12 + i as f32 * d::PX_54,
            d::PX_48,
            d::PX_48,
        );
        let response = click(
            ui,
            rr,
            ("orientation", i),
            [
                "Landscape",
                "Portrait",
                "Landscape flipped",
                "Portrait flipped",
            ][i],
        );
        if response.hovered() {
            ui.painter()
                .rect_filled(rr, t::CONTROL_RADIUS, t::interaction::RAISED_HOVER);
        }
        let col = if c.mapping.orientation == o {
            t::ACCENT
        } else {
            t::MUTED
        };
        let r = if i % 2 == 0 {
            rect(rr.left() + d::PX_7, rr.top() + d::PX_12, d::PX_34, d::PX_24)
        } else {
            rect(rr.left() + d::PX_12, rr.top() + d::PX_7, d::PX_24, d::PX_34)
        };
        ui.painter()
            .rect_stroke(r, 3, Stroke::new(d::PX_2, col), StrokeKind::Inside);
        let band = match i {
            0 => rect(r.left(), r.top(), r.width(), d::PX_8),
            1 => rect(r.right() - d::PX_8, r.top(), d::PX_8, r.height()),
            2 => rect(r.left(), r.bottom() - d::PX_8, r.width(), d::PX_8),
            _ => rect(r.left(), r.top(), d::PX_8, r.height()),
        };
        ui.painter().rect_filled(band, 2, col);
        if response.clicked() {
            s.pending_orientation = Some(o);
        }
    }
}
fn mapping(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response) {
    let w = crate::widgets::view(ui).right();
    let h = crate::widgets::view(ui).bottom();
    let fw = w - d::PX_201 - d::PX_56 - d::PX_340;
    let scale = (fw / d::PX_600).min((h - d::PX_170) / d::PX_670);
    let origin = pos2(d::PX_229 + (fw - d::PX_600 * scale) / d::PX_2, d::PX_168);
    let rr = |x, y, ww, hh| {
        rect(
            origin.x + x * scale,
            origin.y + y * scale,
            ww * scale,
            hh * scale,
        )
    };
    let ds = &status.displays;
    let x0 = ds.iter().map(|d| d.x).reduce(f64::min).unwrap_or_default();
    let y0 = ds.iter().map(|d| d.y).reduce(f64::min).unwrap_or_default();
    let dw = ds
        .iter()
        .map(|d| d.x + d.width)
        .reduce(f64::max)
        .unwrap_or(1.)
        - x0;
    let dh = ds
        .iter()
        .map(|d| d.y + d.height)
        .reduce(f64::max)
        .unwrap_or(1.)
        - y0;
    let sc = (f64::from(d::PX_540) / dw).min(f64::from(d::PX_240) / dh) as f32;
    let ox = (d::PX_600 - dw as f32 * sc) / d::PX_2;
    let mut target = rr(ox, d::PX_16, dw as f32 * sc, dh as f32 * sc);
    for d in ds {
        if c.mapping.display.as_ref() == Some(&d.id) {
            target = rr(
                ox + (d.x - x0) as f32 * sc,
                d::PX_16 + (d.y - y0) as f32 * sc,
                d.width as f32 * sc,
                d.height as f32 * sc,
            );
        }
    }
    let screen_bounds = target;
    target = subrect(target, c.mapping.screen_portion);
    let product = status.device.as_ref().map_or(0x0374, |d| d.product);
    let image = tablet_art_rect(
        rr(d::PX_160, d::PX_384, d::PX_277, d::PX_221),
        product,
        c.mapping.orientation,
        false,
    );
    let raw = if tablet_config::intuos_pro_2013(product) {
        rect(
            image.left() + image.width() * t::device_art::PRO_2013_ACTIVE_X,
            image.top() + image.height() * t::device_art::PRO_2013_ACTIVE_Y,
            image.width() * t::device_art::PRO_2013_ACTIVE_WIDTH,
            image.height() * t::device_art::PRO_2013_ACTIVE_HEIGHT,
        )
    } else if crate::shared::device_art(product).name != "intuos-s" {
        let [x, y, width, height] = crate::shared::active_surface(product);
        rect(
            image.left() + image.width() * x,
            image.top() + image.height() * y,
            image.width() * width,
            image.height() * height,
        )
    } else {
        rr(
            d::PX_160 + d::TABLET_ACTIVE_X * d::PX_277,
            d::PX_384 + d::TABLET_ACTIVE_Y * d::PX_221,
            d::TABLET_ACTIVE_WIDTH * d::PX_277,
            d::TABLET_ACTIVE_HEIGHT * d::PX_221,
        )
    };
    let angle = match c.mapping.orientation {
        Orientation::Landscape => 0.,
        Orientation::Portrait => std::f32::consts::FRAC_PI_2,
        Orientation::LandscapeFlipped => std::f32::consts::PI,
        Orientation::PortraitFlipped => d::PX_3 * std::f32::consts::FRAC_PI_2,
    };
    let rotation = egui::emath::Rot2::from_angle(angle);
    let a = image.center() + rotation * (raw.min - image.center());
    let b = image.center() + rotation * (raw.max - image.center());
    let full = egui::Rect::from_two_pos(a, b);
    let mut area = subrect(full, c.mapping.tablet_portion);
    if c.mapping.force_proportions {
        let physical = status
            .device
            .as_ref()
            .filter(|d| d.width_mm > 0. && d.height_mm > 0.)
            .map(|d| (d.width_mm / d.height_mm) as f32)
            .unwrap_or(raw.width() / raw.height());
        let physical = if c.mapping.orientation.is_portrait() {
            1. / physical
        } else {
            physical
        };
        let a = physical
            * c.mapping
                .tablet_portion
                .map(|p| (p.w / p.h) as f32)
                .unwrap_or(1.);
        let b = target.width() / target.height();
        if a > b {
            area.max.x = area.min.x + area.width() * b / a;
        } else {
            area.max.y = area.min.y + area.height() * a / b;
        }
    }
    let beam = vec![
        target.left_bottom(),
        target.right_bottom(),
        area.right_top(),
        area.left_top(),
    ];
    ui.painter()
        .add(egui::Shape::convex_polygon(beam, t::BEAM, Stroke::NONE));
    for (a, b) in [
        (target.left_bottom(), area.left_top()),
        (target.right_bottom(), area.right_top()),
    ] {
        ui.painter()
            .line_segment([a, b], Stroke::new(d::THIN_STROKE * scale, t::BEAM_EDGE));
    }
    for (i, d) in ds.iter().enumerate() {
        let r = rr(
            ox + (d.x - x0) as f32 * sc + d::PX_3,
            d::PX_16 + (d.y - y0) as f32 * sc + d::PX_3,
            d.width as f32 * sc - d::PX_6,
            d.height as f32 * sc - d::PX_6,
        );
        let on = c.mapping.display.is_none() || c.mapping.display.as_ref() == Some(&d.id);
        let response = click(ui, r, ("monitor", i), &d.name);
        let border = crate::widgets::animated_color(
            &response,
            if on { t::ACCENT } else { t::TRACK },
            if on {
                t::interaction::MONITOR_ON_HOVER
            } else {
                t::interaction::MONITOR_HOVER
            },
            t::interaction::MONITOR_TIME,
        );
        ui.painter().rect(
            r,
            6,
            egui::Color32::from(egui::lerp(
                egui::Rgba::from(t::if_screen(false))..=egui::Rgba::from(t::if_screen(true)),
                ui.ctx().animate_bool_with_time(
                    response.id.with("screen-on"),
                    on,
                    t::interaction::MONITOR_TIME,
                ),
            )),
            Stroke::new(d::PX_2 * scale, border),
            StrokeKind::Middle,
        );
        ui.painter().text(
            r.center() - vec2(0., d::PX_9 * scale),
            Align2::CENTER_CENTER,
            (i + 1).to_string(),
            t::semibold(d::PX_26 * scale),
            t::TEXT,
        );
        ui.painter().text(
            r.center() + vec2(0., d::PX_17 * scale),
            Align2::CENTER_CENTER,
            format!("{}×{}", d.width, d.height),
            t::font(d::PX_12 * scale),
            t::SCREEN_SIZE,
        );
        if response.clicked() {
            c.mapping.display = if c.mapping.display.as_ref() == Some(&d.id) {
                None
            } else {
                Some(d.id.clone())
            };
        }
    }
    if c.mapping.display.is_none() {
        let r = screen_bounds;
        for (a, b) in [
            (r.left_top(), r.right_top()),
            (r.right_top(), r.right_bottom()),
            (r.right_bottom(), r.left_bottom()),
            (r.left_bottom(), r.left_top()),
        ] {
            ui.painter().add(egui::Shape::dashed_line(
                &[a, b],
                Stroke::new(d::THIN_STROKE * scale, t::ACCENT),
                d::PX_5 * scale,
                d::PX_4 * scale,
            ));
        }
    }
    tablet_art(ui, image, product, "mapping", c.mapping.orientation);
    if area != full {
        ui.painter().rect(
            full,
            t::TRACK_RADIUS,
            t::UNUSED_AREA,
            Stroke::new(1., t::KEY_RING),
            StrokeKind::Middle,
        );
    }
    ui.painter().rect(
        area,
        3,
        t::AREA,
        Stroke::new(d::PX_2 * scale, t::ACCENT),
        StrokeKind::Middle,
    );
    portion(ui, "screen", screen_bounds, &mut c.mapping.screen_portion);
    portion(ui, "tablet", full, &mut c.mapping.tablet_portion);
    let x = w - d::PX_329;
    text(ui, x, d::PX_209, &l.t("Display mapping"), d::PX_16, t::TEXT);
    if radio(
        ui,
        x,
        d::PX_241,
        &l.t("Map to all displays"),
        c.mapping.display.is_none(),
    ) {
        c.mapping.display = None;
    }
    for (i, d) in ds.iter().enumerate() {
        let label = format!(
            "{} · {} ({}×{})",
            l.t("Display %s").replace("%s", &(i + 1).to_string()),
            d.name,
            d.width,
            d.height
        );
        if radio(
            ui,
            x,
            d::PX_270 + i as f32 * d::PX_29,
            &label,
            c.mapping.display.as_ref() == Some(&d.id),
        ) {
            c.mapping.display = Some(d.id.clone());
        }
    }
    for (y, label, full_label, p) in [
        (
            d::PX_356,
            "Screen area",
            "Full screen",
            &mut c.mapping.screen_portion,
        ),
        (
            d::PX_568,
            "Tablet area",
            "Use full area",
            &mut c.mapping.tablet_portion,
        ),
    ] {
        text(ui, x, y - 1., &l.t(label), d::PX_16, t::TEXT);
        if radio(
            ui,
            x,
            y + d::PX_30 + if y > d::PX_500 { 1. } else { 0. },
            &l.t(full_label),
            p.is_none(),
        ) {
            *p = None;
        }
        if radio(
            ui,
            x,
            y + d::PX_59 + if y > d::PX_500 { 1. } else { 0. },
            &l.t("Portion"),
            p.is_some(),
        ) {
            *p = Some(tablet_config::Rect {
                x: d::PORTION_INSET,
                y: d::PORTION_INSET,
                w: d::PORTION_SIZE,
                h: d::PORTION_SIZE,
            });
        }
    }
    let force = l.t("Force proportions");
    text(ui, x, d::PX_471, &force, d::PX_16, t::TEXT);
    // The help icon follows labels longer than the English one.
    let help_x = x
        + (crate::widgets::text_width(ui, &force, d::PX_16) + t::control_layout::LABEL_GAP
            - d::PX_134)
            .max(0.);
    let label = l.t(if c.mapping.force_proportions {
        "On"
    } else {
        "Off"
    });
    switch(ui, x, d::PX_502, &mut c.mapping.force_proportions, &label);
    let hp = click(
        ui,
        rect(help_x + d::PX_134, d::PX_472, d::PX_24, d::PX_24),
        "force-help",
        "More information",
    );
    hp.clone().on_hover_cursor(egui::CursorIcon::Help);
    let help_active = hp.hovered() || hp.has_focus();
    text(
        ui,
        help_x + d::PX_138,
        d::PX_473,
        "?",
        d::PX_14,
        if help_active { t::TEXT } else { t::MUTED },
    );
    ui.painter().circle_stroke(
        pos2(help_x + d::PX_143, d::PX_482),
        d::PX_9,
        Stroke::new(
            d::THIN_STROKE,
            if help_active { t::ACCENT } else { t::MUTED },
        ),
    );
    egui::Popup::from_toggle_button_response(&hp).open(help_active).width(t::control_layout::HELP_WIDTH).show(|ui| { ui.label(l.t("To prevent this, select \"Force proportions\" to use only a proportional area of the pen display. This may leave a portion of your pen display inactive, but ensures proportional pen movement.")); });
    if button(
        ui,
        rect(w - d::PX_176, h - d::PX_60, d::PX_145, d::PX_33),
        &l.t("Default"),
    ) {
        let o = c.mapping.orientation;
        c.mapping = Default::default();
        c.mapping.orientation = o;
    }
}
fn subrect(b: egui::Rect, p: Option<tablet_config::Rect>) -> egui::Rect {
    p.map(|p| {
        rect(
            b.left() + p.x as f32 * b.width(),
            b.top() + p.y as f32 * b.height(),
            p.w as f32 * b.width(),
            p.h as f32 * b.height(),
        )
    })
    .unwrap_or(b)
}
fn portion(ui: &mut Ui, id: &str, b: egui::Rect, p: &mut Option<tablet_config::Rect>) {
    let Some(v) = p else {
        return;
    };
    let r = subrect(b, Some(*v));
    ui.painter()
        .rect_stroke(r, 0, Stroke::new(d::PX_2, t::ACCENT), StrokeKind::Inside);
    let resp = ui.interact(r, ui.id().with((id, "body")), egui::Sense::drag());
    if resp.dragged() {
        let d = resp.drag_delta();
        v.x += (d.x / b.width()) as f64;
        v.y += (d.y / b.height()) as f64;
        *v = v.clamped();
    }
    for (i, (x, y)) in [
        (0., 0.),
        (0.5, 0.),
        (1., 0.),
        (1., 0.5),
        (1., 1.),
        (0.5, 1.),
        (0., 1.),
        (0., 0.5),
    ]
    .into_iter()
    .enumerate()
    {
        let h = egui::Rect::from_center_size(
            pos2(r.left() + x * r.width(), r.top() + y * r.height()),
            vec2(d::PX_8, d::PX_8),
        );
        ui.painter().rect(
            h,
            0,
            t::TEXT,
            Stroke::new(d::THIN_STROKE, t::ACCENT),
            StrokeKind::Inside,
        );
        let resp = ui.interact(
            h.expand(d::PX_3),
            ui.id().with((id, i)),
            egui::Sense::drag(),
        );
        if resp.dragged() {
            let d = resp.drag_delta();
            let dx = (d.x / b.width()) as f64;
            let dy = (d.y / b.height()) as f64;
            if x == 0. {
                let nx = (v.x + dx).clamp(0., v.x + v.w - d::PORTION_MIN);
                v.w += v.x - nx;
                v.x = nx;
            }
            if x == 1. {
                v.w = (v.w + dx).clamp(d::PORTION_MIN, 1. - v.x);
            }
            if y == 0. {
                let ny = (v.y + dy).clamp(0., v.y + v.h - d::PORTION_MIN);
                v.h += v.y - ny;
                v.y = ny;
            }
            if y == 1. {
                v.h = (v.h + dy).clamp(d::PORTION_MIN, 1. - v.y);
            }
        }
    }
}
/// Display toggle illustration, explanation and current display, centered
/// on `cx` from `y` down.
#[allow(clippy::too_many_arguments)]
fn toggle(
    ui: &mut Ui,
    l: &Locale,
    cx: f32,
    y: f32,
    scale: f32,
    c: &Config,
    status: &Response,
    native: bool,
) {
    let x = cx - d::PX_260 * scale;
    let r = |a, b, w, h| rect(x + a * scale, y + b * scale, w * scale, h * scale);
    for xx in [d::PX_40, d::PX_280] {
        ui.painter().rect(
            r(xx, d::PX_20, d::PX_200, d::PX_120),
            8,
            t::VIEW,
            Stroke::new(d::PX_2 * scale, t::TOGGLE_OUTLINE),
            StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [
                r(xx + d::PX_92, d::PX_140, 0., 0.).min,
                r(xx + d::PX_92, d::PX_154, 0., 0.).min,
            ],
            Stroke::new(d::PX_4 * scale, t::LINE),
        );
        ui.painter().line_segment(
            [
                r(xx + d::PX_68, d::PX_154, 0., 0.).min,
                r(xx + d::PX_116, d::PX_154, 0., 0.).min,
            ],
            Stroke::new(d::PX_4 * scale, t::LINE),
        );
    }
    ui.painter().rect(
        r(d::PX_180, d::PX_190, d::PX_160, d::PX_96),
        10,
        t::VIEW,
        Stroke::new(d::PX_2 * scale, t::LINE),
        StrokeKind::Inside,
    );
    if native {
        let time = ui.input(|i| i.time);
        let phase = ((time / 2.5) as usize) % 3;
        let (left, right) = match phase {
            1 => (d::PX_40, d::PX_240),
            2 => (d::PX_280, d::PX_480),
            _ => (d::PX_36, d::PX_484),
        };
        let target = r(left, d::PX_20, right - left, d::PX_120);
        let points = vec![
            target.left_bottom(),
            target.right_bottom(),
            r(d::PX_315, d::PX_222, 0., 0.).min,
            r(d::PX_205, d::PX_222, 0., 0.).min,
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            points,
            t::TOGGLE_BEAM_ACTIVE,
            Stroke::NONE,
        ));
        ui.painter().rect_stroke(
            target,
            t::RADIUS,
            Stroke::new(d::PX_3 * scale, t::ACCENT),
            StrokeKind::Middle,
        );
        ui.painter()
            .circle_filled(target.center(), d::PX_5 * scale, t::TEXT);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(40));
    }
    let ar = r(d::PX_205, d::PX_222, d::PX_110, d::PX_50);
    for (a, b) in [
        (ar.left_top(), ar.right_top()),
        (ar.right_top(), ar.right_bottom()),
        (ar.right_bottom(), ar.left_bottom()),
        (ar.left_bottom(), ar.left_top()),
    ] {
        ui.painter().add(egui::Shape::dashed_line(
            &[a, b],
            Stroke::new(1., t::MUTED),
            d::PX_4 * scale,
            d::PX_3 * scale,
        ));
    }
    for xx in [d::PX_260, d::PX_380] {
        ui.painter()
            .circle_filled(r(xx, d::PX_80, 0., 0.).min, d::PX_5 * scale, t::TOGGLE_DOT);
    }
    for i in 0..4 {
        ui.painter().rect_filled(
            r(
                d::PX_196 + i as f32 * d::PX_34,
                d::PX_198,
                d::PX_30,
                d::PX_14,
            ),
            7,
            if i == 0 { t::ACCENT } else { t::LINE },
        );
    }
    let size = d::PX_19 * scale;
    let mut job = egui::text::LayoutJob::simple(
        l.t("Display toggle cycles your cursor between multiple screens and is only available for multi-screen setups."),
        t::font(size),
        t::TEXT,
        d::PX_560 * scale,
    );
    job.halign = egui::Align::Center;
    for section in &mut job.sections {
        section.format.line_height = Some(size * d::TOGGLE_LINE_HEIGHT);
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter()
        .galley(pos2(cx, y + d::PX_300 * scale + d::PX_29), galley, t::TEXT);
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &format!("{}: ", l.t("Now")),
        0.,
        egui::TextFormat::simple(t::font(d::PX_14), t::MUTED),
    );
    let current = status
        .toggled_display
        .as_ref()
        .unwrap_or(&c.mapping.display);
    let label = current
        .as_ref()
        .and_then(|id| {
            status
                .displays
                .iter()
                .enumerate()
                .find(|(_, d)| &d.id == id)
        })
        .map(|(i, d)| {
            format!(
                "{} · {}",
                l.t("Display %s").replace("%s", &(i + 1).to_string()),
                d.name
            )
        })
        .unwrap_or_else(|| l.t("Map to all displays"));
    job.append(
        &label,
        0.,
        egui::TextFormat::simple(t::medium(d::PX_14), t::TEXT),
    );
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let width = galley.size().x;
    ui.painter().galley(
        pos2(cx - width / d::PX_2, y + d::PX_300 * scale + d::PX_125),
        galley,
        t::TEXT,
    );
}
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
fn action_name(id: &str) -> String {
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
        let progress = super::pen::panel_progress(ui.ctx(), s.selected.is_some());
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
fn apply(c: &mut Config, s: &State, a: tablet_config::KeyAction) {
    if let Some(i) = s.selected {
        if let Some(menu) = s.menu {
            let menu = super::shortcuts::menu_mut(&mut c.shortcuts, menu);
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
fn actions(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State, slide: f32) {
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
                let entry = super::shortcuts::items(&c.shortcuts, menu)
                    .get(index)
                    .map(|item| super::shortcuts::item_label(l, item))
                    .unwrap_or_default();
                (l.t(super::shortcuts::title(menu)), entry)
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
/// Loads missing icon art off the UI thread; results land in `s.app_icons`.
pub(crate) fn want_icons<'a>(ui: &Ui, s: &mut State, icons: impl Iterator<Item = &'a str>) {
    if let Some(rx) = &s.icon_loading {
        if let Ok(loaded) = rx.try_recv() {
            s.app_icons.extend(loaded);
            s.icon_loading = None;
        }
    }
    if s.icon_loading.is_some() {
        return;
    }
    let missing: Vec<String> = icons
        .filter(|icon| !s.app_icons.contains_key(*icon))
        .map(String::from)
        .collect();
    if missing.is_empty() {
        return;
    }
    for icon in &missing {
        s.app_icons.insert(icon.clone(), None);
    }
    let (tx, rx) = std::sync::mpsc::channel::<LoadedIcons>();
    s.icon_loading = Some(rx);
    let ctx = ui.ctx().clone();
    std::thread::spawn(move || {
        let loaded = missing
            .into_iter()
            .map(|icon| {
                let data = crate::system::icon_data(&icon);
                (icon, data)
            })
            .collect();
        let _ = tx.send(loaded);
        ctx.request_repaint();
    });
}
/// Paints an app's icon, or its initial on a tile when it has no art.
pub(crate) fn paint_app_icon(
    ui: &Ui,
    icons: &std::collections::HashMap<String, Option<crate::system::IconData>>,
    icon: Option<&str>,
    name: &str,
    r: egui::Rect,
) {
    if !ui.is_rect_visible(r) {
        return;
    }
    if let Some((uri, bytes)) = icon.and_then(|i| icons.get(i)).and_then(Option::as_ref) {
        egui::Image::new(egui::ImageSource::Bytes {
            uri: uri.clone().into(),
            bytes: egui::load::Bytes::Shared(bytes.clone()),
        })
        .paint_at(ui, r);
        return;
    }
    ui.painter().rect_filled(r, t::RADIUS, t::HIGHLIGHT);
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        name.chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string(),
        t::medium(r.height() / 2.),
        t::TEXT,
    );
}
pub(crate) fn poll_apps(ui: &Ui, s: &mut State) {
    if s.native && !s.installed && s.loading.is_none() {
        s.installed = true;
        let (tx, rx) = std::sync::mpsc::channel();
        s.loading = Some(rx);
        let ctx = ui.ctx().clone();
        std::thread::spawn(move || {
            let installed = crate::system::installed_apps();
            let mut running = crate::daemon::Client::default()
                .ask(&tablet_config::Request::RunningApps)
                .map(|r| r.apps)
                .unwrap_or_default();
            // An old or missing Shell extension lists nothing; ask systemd.
            if running.is_empty() {
                running = crate::system::running_apps(&installed);
            }
            let _ = tx.send((installed, running));
            ctx.request_repaint();
        });
    }
    if let Some(rx) = &s.loading {
        if let Ok((installed, running)) = rx.try_recv() {
            s.installed_apps = installed;
            s.running_apps = running;
            s.loading = None;
        }
    }
}
/// Add the chosen app as a profile, cloning the settings of the active one.
pub(crate) fn submit_app(c: &mut Config, s: &mut State, pool: &[tablet_config::AppInfo]) {
    if let Some(app) = s.app_choice.and_then(|i| pool.get(i)) {
        if let Some(i) = c
            .apps
            .iter()
            .position(|a| a.id.eq_ignore_ascii_case(&app.id))
        {
            s.app = Some(i);
        } else {
            let (pen, express_keys) = if let Some(i) = s.app {
                (c.apps[i].pen.clone(), c.apps[i].express_keys.clone())
            } else {
                (c.pen.clone(), c.express_keys.clone())
            };
            c.apps.push(tablet_config::AppProfile {
                id: app.id.clone(),
                name: app.name.clone(),
                icon: app.icon.clone(),
                pen,
                express_keys,
            });
            s.app = Some(c.apps.len() - 1);
        }
        s.add_app = false;
    }
}

fn app_dialog(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
    poll_apps(ui, s);
    let center = crate::widgets::window(ui).center();
    let r = egui::Rect::from_center_size(center, vec2(d::PX_520, d::PX_515));
    let mut child = backdrop(ui, "apps", r);
    if crate::widgets::clicked_outside(&child, "apps", r) {
        s.add_app = false;
    }
    let ui = &mut child;
    let x = r.left() + d::PX_24;
    let y = r.top() + d::PX_22;
    ui.painter().text(
        pos2(x, y),
        Align2::LEFT_TOP,
        l.t("Select an application"),
        t::medium(d::PX_18),
        t::TEXT,
    );
    wrapped(
        ui,
        rect(x, y + d::PX_31, d::PX_472, d::PX_44),
        &l.t("Choose the application whose tool settings you want to customize."),
        d::PX_15,
        t::MUTED,
    );
    ui.painter()
        .rect_filled(rect(x, y + d::PX_70, d::PX_265, d::PX_40), 8, t::VIEW);
    if segment(
        ui,
        rect(x + d::PX_3, y + d::PX_73, d::PX_123, d::PX_34),
        &l.t("Running apps"),
        !s.app_source,
    ) {
        s.app_source = false;
        s.app_choice = None;
    }
    if segment(
        ui,
        rect(x + d::PX_129, y + d::PX_73, d::PX_133, d::PX_34),
        &l.t("Installed apps"),
        s.app_source,
    ) {
        s.app_source = true;
        s.app_choice = None;
    }
    crate::widgets::text_field(
        ui,
        rect(x + d::PX_275, y + d::PX_70, d::PX_197, d::PX_40),
        &mut s.search,
        &l.t("Search applications"),
        usize::MAX,
    );
    let list = rect(x, y + d::PX_122, d::PX_472, d::PX_300);
    ui.painter().rect(
        list,
        8,
        t::VIEW,
        Stroke::new(1., t::LINE),
        StrokeKind::Inside,
    );
    let pool = if s.app_source {
        s.installed_apps.clone()
    } else {
        s.running_apps.clone()
    };
    want_icons(ui, s, pool.iter().filter_map(|a| a.icon.as_deref()));
    let pool = &pool;
    let icons = &s.app_icons;
    let query = s.search.trim().to_lowercase();
    let visible: Vec<usize> = pool
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            a.name.to_lowercase().contains(&query) || a.id.to_lowercase().contains(&query)
        })
        .map(|(i, _)| i)
        .collect();
    if s.app_choice.is_some_and(|i| !visible.contains(&i)) {
        s.app_choice = None;
    }
    let delta = ui.input(|i| {
        i.key_pressed(egui::Key::ArrowDown) as i32 - i.key_pressed(egui::Key::ArrowUp) as i32
    });
    if delta != 0 && !visible.is_empty() {
        let index = s
            .app_choice
            .and_then(|i| visible.iter().position(|v| *v == i))
            .unwrap_or(0);
        let next = (index as i32 + delta).clamp(0, visible.len() as i32 - 1) as usize;
        s.app_choice = Some(visible[next]);
    }
    let mut submit = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let mut list_ui = ui.new_child(egui::UiBuilder::new().max_rect(list.shrink(d::PX_4)));
    list_ui.set_clip_rect(list.shrink(d::PX_4));
    // The dialog layer is not an egui Area, so ScrollArea never sees the
    // pointer over it; the wheel is applied here instead.
    if crate::widgets::pointer(ui).is_some_and(|p| list.contains(p)) {
        s.app_list_scroll = (s.app_list_scroll - ui.input(|i| i.smooth_scroll_delta.y)).max(0.);
    }
    let output = egui::ScrollArea::vertical()
        .vertical_scroll_offset(s.app_list_scroll)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .show(&mut list_ui, |ui| {
            let mut any = false;
            for (i, app) in pool.iter().enumerate().filter(|(_, a)| {
                a.name.to_lowercase().contains(&query) || a.id.to_lowercase().contains(&query)
            }) {
                any = true;
                let (r, response) =
                    ui.allocate_exact_size(vec2(d::PX_456, d::PX_50), egui::Sense::click());
                response
                    .clone()
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if response.hovered() {
                    ui.painter().rect_filled(r, t::CONTROL_RADIUS, t::RAISED);
                }
                if s.app_choice == Some(i) {
                    ui.painter().rect_filled(r, 6, t::CHIP);
                }
                let icon = rect(r.left() + d::PX_10, r.top() + d::PX_9, d::PX_32, d::PX_32);
                paint_app_icon(ui, icons, app.icon.as_deref(), &app.name, icon);
                text(
                    ui,
                    r.left() + d::PX_50,
                    r.top() + d::PX_5,
                    &app.name,
                    d::PX_15,
                    t::TEXT,
                );
                text(
                    ui,
                    r.left() + d::PX_50,
                    r.top() + d::PX_26,
                    &app.id,
                    d::PX_12,
                    t::FAINT,
                );
                if response.clicked() {
                    s.app_choice = Some(i);
                }
                if response.double_clicked() {
                    s.app_choice = Some(i);
                    submit = true;
                }
                if s.app_choice == Some(i) && delta != 0 {
                    response.scroll_to_me(None);
                }
            }
            if !any {
                ui.add_space(d::PX_30);
                ui.painter().text(
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
        });
    s.app_list_scroll = output.state.offset.y;
    // Display only: the dialog layer is not hit-tested, so the wheel scrolls it.
    crate::widgets::scroll_indicator(
        ui,
        egui::Id::new("app-list-scroll"),
        crate::widgets::scroll_track(output.inner_rect),
        &mut s.app_list_scroll,
        (output.content_size.y - output.inner_rect.height()).max(0.),
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
        s.add_app = false;
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
        s.app_choice.is_some(),
    ) || submit
    {
        let pool = pool.clone();
        submit_app(c, s, &pool);
    }
}

fn capture(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
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
fn device_model(status: &Response) -> (&'static str, &'static str) {
    status
        .device
        .as_ref()
        .and_then(|d| crate::shared::model_name(d.product))
        .unwrap_or(("Intuos S", "CTL-4100"))
}
fn tablet_art_rect(r: egui::Rect, product: u16, o: Orientation, fit: bool) -> egui::Rect {
    let portrait = matches!(o, Orientation::Portrait | Orientation::PortraitFlipped);
    if tablet_config::intuos_pro_2013(product) {
        let ratio = t::device_art::PRO_2013_TABLET_RATIO;
        let width = if portrait {
            r.width().min(r.height()).min(r.width() / ratio)
        } else {
            r.width().min(r.height() / ratio)
        };
        egui::Rect::from_center_size(r.center(), vec2(width, width * ratio))
    } else if crate::shared::device_art(product).name != "intuos-s" {
        let ratio = crate::shared::device_art(product).ratio;
        let width = if portrait {
            r.height().min(r.width() / ratio)
        } else {
            r.width().min(r.height() / ratio)
        };
        egui::Rect::from_center_size(r.center(), vec2(width, width * ratio))
    } else if fit && portrait {
        egui::Rect::from_center_size(r.center(), r.size() * d::PORTRAIT_SCALE)
    } else {
        r
    }
}
fn tablet_art(ui: &Ui, r: egui::Rect, product: u16, site: &str, o: Orientation) {
    let turns = match o {
        Orientation::Landscape => 0,
        Orientation::Portrait => 1,
        Orientation::LandscapeFlipped => 2,
        Orientation::PortraitFlipped => 3,
    };
    let art = crate::shared::device_art(product);
    let (key, bytes) = if tablet_config::intuos_pro_2013(product) {
        ("pro-2013", art.bytes)
    } else {
        (art.name, art.bytes)
    };
    let glow =
        if o.is_portrait() && art.name != "intuos-s" && !tablet_config::intuos_pro_2013(product) {
            egui::Rect::from_center_size(r.center(), vec2(r.height(), r.width()))
        } else {
            r
        };
    crate::assets::glow(ui, glow);
    assets::artwork_rotated(
        ui,
        &format!("{site}-{key}"),
        bytes,
        r,
        turns as f32 * std::f32::consts::FRAC_PI_2,
    );
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
fn confirmation(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
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
fn configure(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
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
/// Card heading: a line icon and its title.
fn card_heading(ui: &Ui, r: egui::Rect, icon: &str, label: &str) {
    egui::Image::new(crate::icons::source(icon))
        .tint(t::TEXT)
        .paint_at(
            ui,
            rect(r.left() + ab::PAD, r.top() + ab::PAD, ab::ICON, ab::ICON),
        );
    ui.painter().text(
        pos2(
            r.left() + ab::PAD + ab::TITLE_LEFT,
            r.top() + ab::PAD + ab::ICON / 2.,
        ),
        Align2::LEFT_CENTER,
        label,
        t::medium(ab::TITLE),
        t::TEXT,
    );
}
/// About page: device details, then live tilt and pressure meters.
fn about(ui: &Ui, l: &Locale, w: f32, status: &Response) {
    let device = status.device.as_ref();
    let (name, model) = device_model(status);
    let pen = status.pen.clone().unwrap_or_default();
    // Two columns when both cards fit side by side, else stacked.
    let room = w - ab::LEFT - p::CONTENT_RIGHT;
    let columns = room >= ab::MIN_COLUMN * 2. + ab::GAP;
    let width = if columns {
        ((room - ab::GAP) / 2.).min(ab::MAX_COLUMN)
    } else {
        room.min(ab::MAX_WIDTH)
    };
    let dash = || "—".to_string();
    let rows = [
        ("Connection", "USB".to_string()),
        ("Model", format!("{name} {model}")),
        (
            "Product ID",
            device.map_or_else(dash, |d| format!("{:04x}:{:04x}", d.vendor, d.product)),
        ),
        (
            "Firmware",
            device.and_then(|d| d.firmware.clone()).unwrap_or_else(dash),
        ),
        (
            "Serial number",
            status
                .pen
                .as_ref()
                .and_then(|p| p.serial)
                .map_or_else(dash, |v| v.to_string()),
        ),
        (
            "Pen tool ID",
            status
                .pen
                .as_ref()
                .and_then(|p| p.tool_id)
                .map_or_else(dash, |v| format!("0x{v:x}")),
        ),
    ];
    let height = ab::ROWS_TOP + rows.len() as f32 * ab::ROW_STEP + ab::PAD - d::PX_10;
    let r = rect(ab::LEFT, ab::TOP, width, height);
    crate::widgets::card(ui, r);
    card_heading(ui, r, "info", &l.t("Device details"));
    text(
        ui,
        r.left() + ab::PAD,
        r.top() + ab::NAME_TOP,
        name,
        d::PX_16,
        t::TEXT,
    );
    text(
        ui,
        r.left() + ab::PAD,
        r.top() + ab::KIND_TOP,
        &l.t("Tablet"),
        ab::LABEL,
        t::MUTED,
    );
    // Connection pill: a green dot while the daemon reports the tablet.
    let connected = device.is_some();
    let label = l.t(if connected {
        "Connected"
    } else {
        "Disconnected"
    });
    let pill_width = crate::widgets::text_width(ui, &label, d::PX_13)
        + ab::PILL_PAD * 2.
        + ab::DOT * 2.
        + d::PX_8;
    let pill = rect(
        r.right() - ab::PAD - pill_width,
        r.top() + ab::NAME_TOP,
        pill_width,
        ab::PILL_HEIGHT,
    );
    ui.painter().rect(
        pill,
        t::FULL_RADIUS,
        t::VIEW,
        Stroke::new(t::BORDER, t::LINE),
        StrokeKind::Inside,
    );
    ui.painter().circle_filled(
        pos2(pill.left() + ab::PILL_PAD + ab::DOT, pill.center().y),
        ab::DOT,
        if connected { t::GREEN } else { t::FAINT },
    );
    ui.painter().text(
        pos2(
            pill.left() + ab::PILL_PAD + ab::DOT * 2. + d::PX_8,
            pill.center().y,
        ),
        Align2::LEFT_CENTER,
        label,
        t::font(d::PX_13),
        t::MUTED,
    );
    ui.painter().hline(
        r.left() + ab::PAD..=r.right() - ab::PAD,
        r.top() + ab::DIVIDER_TOP,
        Stroke::new(t::BORDER, t::LINE),
    );
    for (i, (label, value)) in rows.iter().enumerate() {
        let y = r.top() + ab::ROWS_TOP + i as f32 * ab::ROW_STEP;
        text(ui, r.left() + ab::PAD, y, &l.t(label), ab::LABEL, t::MUTED);
        text(
            ui,
            r.left() + ab::PAD + ab::VALUE_COL,
            y - 1.,
            value,
            ab::VALUE,
            t::TEXT,
        );
    }
    // Live pen input: tilt meters fill from their center, pressure from the left.
    let meters = [
        ("Tilt X", pen.tilt_x as f32, true),
        ("Tilt Y", pen.tilt_y as f32, true),
        ("Pressure", pen.pressure as f32 * d::PX_100, false),
    ];
    let (left, top) = if columns {
        (r.right() + ab::GAP, r.top())
    } else {
        (r.left(), r.bottom() + ab::GAP)
    };
    let r = rect(
        left,
        top,
        width,
        ab::METERS_TOP + meters.len() as f32 * ab::METER_STEP,
    );
    crate::widgets::card(ui, r);
    card_heading(ui, r, "pencil", &l.t("Pen input"));
    for (i, (label, value, centered)) in meters.into_iter().enumerate() {
        let y = r.top() + ab::METERS_TOP + i as f32 * ab::METER_STEP;
        text(ui, r.left() + ab::PAD, y, &l.t(label), ab::LABEL, t::MUTED);
        ui.painter().text(
            pos2(r.right() - ab::PAD, y),
            Align2::RIGHT_TOP,
            if centered {
                format!("{value:.0}°")
            } else {
                format!("{value:.0}%")
            },
            t::font(ab::LABEL),
            t::TEXT,
        );
        let track = rect(
            r.left() + ab::PAD,
            y + ab::BAR_TOP,
            r.width() - ab::PAD * 2.,
            ab::BAR,
        );
        ui.painter().rect_filled(track, t::FULL_RADIUS, t::VIEW);
        let fill = if centered {
            let f = (value / d::PX_90).clamp(-1., 1.);
            let mid = track.center().x;
            let end = mid + f * track.width() / 2.;
            egui::Rect::from_x_y_ranges(mid.min(end)..=mid.max(end), track.y_range())
        } else {
            let f = (value / d::PX_100).clamp(0., 1.);
            rect(track.left(), track.top(), track.width() * f, track.height())
        };
        ui.painter().rect_filled(fill, t::FULL_RADIUS, t::ACCENT);
        if centered {
            ui.painter().vline(
                track.center().x,
                track.top() - d::PX_2..=track.bottom() + d::PX_2,
                Stroke::new(t::BORDER, t::FAINT),
            );
        }
    }
}
/// Display toggle explanation, opened from the help mark of its action.
fn toggle_dialog(ui: &mut Ui, l: &Locale, c: &Config, status: &Response, s: &mut State) {
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
fn run_fields(ui: &mut Ui, l: &Locale, s: &mut State, r: egui::Rect) {
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
fn current_action(c: &Config, s: &State) -> serde_json::Value {
    let Some(index) = s.selected else {
        return serde_json::Value::Null;
    };
    if let Some(menu) = s.menu {
        return super::shortcuts::menu(&c.shortcuts, menu)
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
