//! Non-interactive mode transition card shared by the native host and screenshots.
use crate::{classic::shortcuts::paint_radial_button, i18n::Locale, icons, theme};
use egui::{emath::easing, pos2, vec2, FontFamily, FontId, Rect, Stroke, Ui, Vec2};
use tablet_config::PenMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct ModeOsd {
    pub from: PenMode,
    pub to: PenMode,
}

impl ModeOsd {
    pub fn parse(line: &str) -> Option<Self> {
        #[derive(serde::Deserialize)]
        #[serde(tag = "name", rename_all = "snake_case")]
        enum Request {
            ModeOsd(ModeOsd),
        }
        let Request::ModeOsd(mode) = serde_json::from_str(line).ok()?;
        (mode.from != mode.to).then_some(mode)
    }
}

fn mode_content(mode: PenMode) -> (&'static str, &'static str) {
    match mode {
        PenMode::Pen => ("Pen", "mode-pen"),
        PenMode::Mouse => ("Mouse", "mode-mouse"),
        PenMode::Relative => ("Relative", "mode-relative"),
    }
}

pub fn draw_mode_osd(ui: &mut Ui, locale: &Locale, modes: ModeOsd, elapsed: f32) {
    use theme::mode_osd as t;

    let opening = easing::quadratic_out((elapsed / t::SHOW_TIME).clamp(0., 1.));
    let closing = ((elapsed - t::FADE_AT) / t::HIDE_TIME).clamp(0., 1.);
    let zoom = egui::lerp(t::START_SCALE..=1., opening);
    let center = ui.ctx().content_rect().center();
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("mode-osd"));
    let mut card = ui.new_child(egui::UiBuilder::new().layer_id(layer));
    card.ctx().set_transform_layer(
        layer,
        egui::emath::TSTransform::from_translation(center.to_vec2() * (1. - zoom))
            * egui::emath::TSTransform::from_scaling(zoom),
    );
    card.set_opacity(opening * (1. - easing::quadratic_in(closing)));
    let rect = Rect::from_center_size(center, t::SIZE);
    card.painter().rect(
        rect,
        t::RADIUS,
        t::FILL,
        Stroke::new(t::BORDER_WIDTH, t::BORDER),
        egui::StrokeKind::Inside,
    );

    let (from, from_icon) = mode_content(modes.from);
    let (to, to_icon) = mode_content(modes.to);
    let y = rect.top() + t::ICON_CENTER_Y;
    let pop = ((elapsed - t::SHOW_TIME) / t::POP_TIME).clamp(0., 1.);
    let pop = 1. + t::POP_SCALE * (std::f32::consts::PI * pop).sin();
    for (x, scale, icon, active, color) in [
        (
            center.x - t::BUTTON_OFFSET,
            1.,
            from_icon,
            false,
            t::PREVIOUS_ICON,
        ),
        (center.x + t::BUTTON_OFFSET, pop, to_icon, true, theme::TEXT),
    ] {
        let at = pos2(x, y);
        paint_radial_button(card.painter(), at, t::BUTTON_RADIUS * scale, scale, active);
        egui::Image::new(icons::source(icon)).tint(color).paint_at(
            &card,
            Rect::from_center_size(at, Vec2::splat(theme::radial::ACTIVE_ICON * scale)),
        );
    }

    let arrow = pos2(center.x - (1. - opening) * t::ARROW_TRAVEL, y);
    egui::Image::new(icons::source("mode-arrow"))
        .tint(theme::TEXT)
        .paint_at(
            &card,
            Rect::from_center_size(arrow, Vec2::splat(t::ARROW_SIZE)),
        );

    let mut label = egui::text::LayoutJob::default();
    for (value, family) in [
        (format!("{} → ", locale.t(from)), FontFamily::Proportional),
        (locale.t(to), FontFamily::Name("bold".into())),
    ] {
        label.append(
            &value,
            0.,
            egui::TextFormat::simple(FontId::new(t::FONT, family), theme::TEXT),
        );
    }
    let label = card.painter().layout_job(label);
    let position = rect.center_top() + vec2(0., t::LABEL_CENTER_Y) - label.size() / 2.;
    card.painter().galley(position, label, theme::TEXT);
}
