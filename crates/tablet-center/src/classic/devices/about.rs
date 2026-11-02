//! About page and the tablet artwork it shares with other pages.
use super::*;

pub(super) fn device_model(status: &Response) -> (&'static str, &'static str) {
    status
        .device
        .as_ref()
        .and_then(|d| crate::shared::model_name(d.product))
        .unwrap_or(("Intuos S", "CTL-4100"))
}
pub(super) fn tablet_art_rect(
    r: egui::Rect,
    product: u16,
    o: Orientation,
    fit: bool,
) -> egui::Rect {
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
pub(super) fn tablet_art(ui: &Ui, r: egui::Rect, product: u16, site: &str, o: Orientation) {
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
/// Card heading: a line icon and its title.
pub(super) fn card_heading(ui: &Ui, r: egui::Rect, icon: &str, label: &str) {
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
pub(super) fn about(ui: &Ui, l: &Locale, w: f32, status: &Response) {
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
