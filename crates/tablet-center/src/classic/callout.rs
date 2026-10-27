//! Callout cards: one per device button, joined to its spot on the
//! artwork by a leader, showing the button, its action and a chevron.
use crate::theme::{self as t, pen_layout as layout};
use crate::widgets::{click, rect};
use egui::{pos2, Align2, Galley, Pos2, Rect, Stroke, Ui};
use std::sync::Arc;

/// The action name, wrapped to two lines in the card's text column.
pub fn label(ui: &Ui, text: &str) -> Arc<Galley> {
    crate::widgets::two_lines(
        ui,
        text,
        t::SPACE_16,
        layout::CALLOUT_WIDTH
            - layout::CALLOUT_PAD
            - layout::CALLOUT_ICON
            - t::SPACE_10
            - layout::CALLOUT_CHEVRON_ROOM,
    )
}
/// Card height for a laid out action name.
pub fn height(label: &Galley) -> f32 {
    layout::CALLOUT_ACTION_TOP + label.size().y.max(layout::CALLOUT_ICON) + layout::CALLOUT_BOTTOM
}
/// Card tops, in `spots` order. Top to bottom, then right to left, each card
/// sits level with its spot unless the one above pushes it down; a stack that
/// would pass `bottom` is lifted, but never above `top`.
pub fn stack(spots: &[Pos2], heights: &[f32], top: f32, bottom: f32) -> Vec<f32> {
    let mut order: Vec<usize> = (0..spots.len()).collect();
    order.sort_by(|a, b| {
        (spots[*a].y, -spots[*a].x)
            .partial_cmp(&(spots[*b].y, -spots[*b].x))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut tops = vec![0.; spots.len()];
    let mut next = top;
    for &i in &order {
        tops[i] = (spots[i].y - heights[i] / 2.).max(next);
        next = tops[i] + heights[i] + layout::CALLOUT_GAP;
    }
    let first = order.first().map_or(top, |&i| tops[i]);
    let lift = (next - layout::CALLOUT_GAP - bottom).clamp(0., (first - top).max(0.));
    tops.iter().map(|y| y - lift).collect()
}
/// One card in `r` for the button at `spot`, its leader bending at `bend`.
/// Lit while selected, held or its spot is hovered; true when clicked.
#[allow(clippy::too_many_arguments)] // One card mirrors a labeled button.
pub fn card(
    ui: &mut Ui,
    r: Rect,
    spot: Pos2,
    bend: Pos2,
    name: &str,
    icon: &str,
    label: Arc<Galley>,
    selected: bool,
    held: bool,
) -> bool {
    let response = click(ui, r, ("callout", name), name);
    let ring = ui.interact(
        Rect::from_center_size(spot, egui::vec2(t::SPACE_24, t::SPACE_24)),
        ui.id().with(("callout-spot", name)),
        egui::Sense::click(),
    );
    ring.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
    let lit = selected || held || ring.hovered();
    let leader = if lit {
        Stroke::new(t::SPACE_2, t::ACCENT)
    } else {
        Stroke::new(t::BORDER, t::MUTED)
    };
    let painter = ui.painter();
    painter.line_segment([spot, bend], leader);
    painter.line_segment([bend, pos2(r.left(), r.center().y)], leader);
    let fill = match (lit, response.hovered()) {
        (true, true) => t::interaction::ACCENT_TINT_HOVER,
        (true, false) => t::interaction::ACCENT_TINT,
        (false, true) => t::interaction::WHITE_8,
        (false, false) => t::RAISED,
    };
    painter.rect(
        r,
        t::CONTROL_RADIUS,
        fill,
        Stroke::new(t::BORDER, if lit { t::ACCENT } else { t::LINE }),
        egui::StrokeKind::Inside,
    );
    if lit {
        painter.circle_filled(spot, layout::SPOT_RADIUS, t::ACCENT);
    } else {
        painter.circle(
            spot,
            layout::SPOT_RADIUS,
            t::PANEL,
            Stroke::new(layout::SPOT_STROKE, t::RING),
        );
    }
    let tone = if lit { t::ACCENT } else { t::TEXT };
    painter.text(
        pos2(
            r.left() + layout::CALLOUT_PAD,
            r.top() + layout::CALLOUT_NAME_TOP,
        ),
        Align2::LEFT_TOP,
        name,
        t::font(t::SPACE_13),
        t::MUTED,
    );
    egui::Image::new(crate::classic::devices::action_icon(icon))
        .tint(tone)
        .paint_at(
            ui,
            rect(
                r.left() + layout::CALLOUT_PAD,
                r.top() + layout::CALLOUT_ACTION_TOP,
                layout::CALLOUT_ICON,
                layout::CALLOUT_ICON,
            ),
        );
    let text_x = r.left() + layout::CALLOUT_PAD + layout::CALLOUT_ICON + t::SPACE_10;
    // The icon sits level with the first line.
    let first_line = label
        .rows
        .first()
        .map_or(layout::CALLOUT_ICON, |row| row.height());
    ui.painter().galley(
        pos2(
            text_x,
            r.top() + layout::CALLOUT_ACTION_TOP + (layout::CALLOUT_ICON - first_line) / 2.,
        ),
        label,
        tone,
    );
    egui::Image::new(crate::icons::source("chevron-right"))
        .tint(t::MUTED)
        .paint_at(
            ui,
            Rect::from_center_size(
                pos2(r.right() - layout::CALLOUT_CHEVRON_ROOM / 2., r.center().y),
                egui::vec2(layout::CALLOUT_ICON - 4., layout::CALLOUT_ICON - 4.),
            ),
        );
    response.clicked() || ring.clicked()
}
