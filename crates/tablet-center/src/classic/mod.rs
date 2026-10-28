//! Classic left column (tab switch, device navigation, settings) and Pen page.
pub mod callout;
pub mod devices;
pub mod pen;
pub mod settings;
pub mod shortcuts;
use crate::theme::sidebar_layout as layout;
use crate::{
    assets, theme as t,
    widgets::{click, rect, text},
};
use egui::{pos2, Align2, Stroke, StrokeKind, Ui};
/// Left column shared by every tab: the Devices | Shortcuts switch on top,
/// Settings pinned to the bottom. Tabs draw their own
/// navigation between both.
pub fn navigation(ui: &mut Ui, h: f32, locale: &crate::i18n::Locale, tab: &mut usize) {
    ui.painter().rect_filled(
        rect(1., t::TITLE_BAR, t::SIDEBAR - 1., h - t::TITLE_BAR - 1.),
        t::SQUARE,
        t::PANEL,
    );
    ui.painter().vline(
        layout::BORDER_X,
        t::TITLE_BAR..=h - 1.,
        Stroke::new(t::BORDER, t::LINE),
    );
    ui.painter().rect(
        rect(
            layout::LEFT,
            layout::SWITCH_TOP,
            layout::ITEM_WIDTH,
            layout::SWITCH_HEIGHT,
        ),
        layout::SWITCH_RADIUS,
        t::VIEW,
        Stroke::new(t::BORDER, t::LINE),
        StrokeKind::Inside,
    );
    let segment = (layout::ITEM_WIDTH - 2. * layout::SWITCH_PAD - layout::SWITCH_GAP) / 2.;
    for (i, short, label) in [
        (0, "Devices", "Devices"),
        (1, "Shortcuts", "On-screen shortcuts"),
    ] {
        let r = rect(
            layout::LEFT + layout::SWITCH_PAD + i as f32 * (segment + layout::SWITCH_GAP),
            layout::SWITCH_TOP + layout::SWITCH_PAD,
            segment,
            layout::SWITCH_HEIGHT - 2. * layout::SWITCH_PAD,
        );
        let response = click(ui, r, ("tab", i), label);
        let color = if *tab == i {
            ui.painter().rect_filled(r, t::RADIUS, t::RAISED);
            t::ACCENT
        } else {
            crate::widgets::hover_color(&response, t::MUTED, t::TEXT)
        };
        egui::Image::new(assets::icon(i)).tint(color).paint_at(
            ui,
            rect(
                r.center().x - layout::SWITCH_ICON / 2.,
                r.top() + layout::SWITCH_ICON_TOP,
                layout::SWITCH_ICON,
                layout::SWITCH_ICON,
            ),
        );
        // Long translations shrink to the segment instead of overflowing.
        let caption = locale.t(short);
        let room = segment - t::SPACE_6;
        let width = crate::widgets::text_width(ui, &caption, layout::SWITCH_FONT);
        let size = layout::SWITCH_FONT * (room / width).min(1.);
        ui.painter().text(
            pos2(r.center().x, r.top() + layout::SWITCH_LABEL_TOP),
            Align2::CENTER_CENTER,
            caption,
            t::font(size),
            color,
        );
        if response.clicked() {
            *tab = i;
        }
    }
    let top = h - layout::FOOTER;
    ui.painter()
        .hline(1. ..=t::SIDEBAR, top + 0.5, Stroke::new(t::BORDER, t::LINE));
    let y = top + layout::FOOTER_ITEM_TOP;
    let r = rect(layout::LEFT, y, layout::ITEM_WIDTH, layout::ITEM_HEIGHT);
    let response = click(ui, r, ("tab", 2), "Settings");
    if *tab == 2 {
        ui.painter().rect_filled(r, t::RADIUS, t::ACCENT);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(r, t::RADIUS, t::interaction::WHITE_6);
    }
    egui::Image::new(assets::icon(2)).tint(t::TEXT).paint_at(
        ui,
        rect(
            layout::ICON_LEFT,
            y + t::SPACE_9,
            layout::ICON_SIZE,
            layout::ICON_SIZE,
        ),
    );
    text(
        ui,
        layout::LABEL_LEFT,
        y + t::SPACE_10,
        &locale.t("Settings"),
        layout::FONT,
        t::TEXT,
    );
    if response.clicked() {
        *tab = 2;
    }
}
/// Device pages under the switch.
pub fn sidebar(
    ui: &mut Ui,
    locale: &crate::i18n::Locale,
    page: &mut usize,
    device: Option<&tablet_config::DeviceInfo>,
    touch: bool,
) {
    ui.painter().rect_filled(
        rect(
            layout::LEFT,
            layout::DEVICE_TOP,
            layout::ITEM_WIDTH,
            layout::DEVICE_HEIGHT,
        ),
        t::RADIUS,
        t::DEVICE,
    );
    let image = rect(
        layout::DEVICE_IMAGE_LEFT,
        layout::DEVICE_IMAGE_TOP,
        layout::DEVICE_IMAGE_WIDTH,
        layout::DEVICE_IMAGE_HEIGHT,
    );
    if !device.is_some_and(|d| crate::shared::thumbnail(ui, d.product, image)) {
        assets::artwork(
            ui,
            "tablet",
            include_bytes!("../../../../assets/devices/intuos-s.png"),
            image,
        );
    }
    if click(
        ui,
        rect(
            layout::LEFT,
            layout::DEVICE_TOP,
            layout::ITEM_WIDTH,
            layout::DEVICE_HEIGHT,
        ),
        "overview",
        "Intuos S",
    )
    .clicked()
    {
        *page = 6;
    }
    let name = device
        .map(|d| crate::shared::model_name(d.product).map_or(d.name.as_str(), |(name, _)| name))
        .unwrap_or("Tablet");
    text(
        ui,
        layout::DEVICE_LABEL_LEFT,
        layout::DEVICE_LABEL_TOP,
        name,
        layout::DEVICE_FONT,
        t::TEXT,
    );
    let mut items = vec![(0, 3, "Pen")];
    if device.is_none_or(|d| tablet_config::express_key_count(d.product) > 0) {
        items.push((1, 4, "ExpressKeys"));
    }
    if touch {
        items.push((5, 7, "Touch"));
    }
    items.push((2, 5, "Mapping"));
    for (i, (target, icon, label)) in items.into_iter().enumerate() {
        let y = layout::ITEM_TOP + i as f32 * layout::ITEM_STEP;
        let response = click(
            ui,
            rect(layout::LEFT, y, layout::ITEM_WIDTH, layout::ITEM_HEIGHT),
            ("page", i),
            label,
        );
        if *page != target && response.hovered() {
            ui.painter()
                .rect_filled(response.rect, t::RADIUS, t::interaction::WHITE_6);
        }
        if *page == target {
            ui.painter().rect_filled(
                rect(layout::LEFT, y, layout::ITEM_WIDTH, layout::ITEM_HEIGHT),
                t::RADIUS,
                t::ACCENT,
            );
        }
        egui::Image::new(assets::icon(icon)).tint(t::TEXT).paint_at(
            ui,
            rect(
                layout::ICON_LEFT,
                y + t::SPACE_9,
                layout::ICON_SIZE,
                layout::ICON_SIZE,
            ),
        );
        crate::widgets::fit_text(
            ui,
            egui::pos2(
                layout::LABEL_LEFT,
                y + t::SPACE_10 + if i == 1 { 1. } else { 0. },
            ),
            &locale.t(label),
            layout::FONT,
            layout::LEFT + layout::ITEM_WIDTH - layout::LABEL_LEFT - layout::LABEL_PAD,
            rect(layout::LEFT, y, layout::ITEM_WIDTH, layout::ITEM_HEIGHT),
            t::TEXT,
        );
        if response.clicked() {
            *page = target;
        }
    }
}
