//! Custom title bar, window buttons, resize handles and inner highlight.
use crate::theme::frame_layout as layout;
use crate::{
    assets, theme as t,
    widgets::{click, rect, text},
};
use egui::{pos2, vec2, Rect, Stroke, StrokeKind, Ui, ViewportCommand as Cmd};
pub fn show(ui: &mut Ui, r: Rect) {
    ui.painter().rect_filled(r, t::WINDOW_RADIUS, t::BG);
    ui.painter().rect_filled(
        rect(1., 1., r.width() - t::SPACE_2, t::TITLE_BAR - 1.),
        t::INNER_RADIUS,
        t::BAR,
    );
    ui.painter().rect_filled(
        rect(
            1.,
            layout::BAR_SQUARE_TOP,
            r.width() - t::SPACE_2,
            t::TITLE_BAR - layout::BAR_SQUARE_TOP,
        ),
        t::SQUARE,
        t::BAR,
    );
    ui.painter().hline(
        1. ..=r.right() - 1.,
        layout::TITLE_LINE_Y,
        Stroke::new(t::BORDER, t::LINE),
    );
    egui::Image::new(assets::LOGO).paint_at(
        ui,
        rect(
            layout::LOGO_LEFT,
            layout::INNER_RADIUS,
            layout::LOGO_SIZE,
            layout::LOGO_SIZE,
        ),
    );
    ui.painter().text(
        pos2(layout::TITLE_LEFT, layout::CAPTION_RADIUS),
        egui::Align2::LEFT_TOP,
        "Tablet Center",
        t::semibold(layout::CAPTION_RADIUS),
        t::TEXT.gamma_multiply(0.8),
    );
    let title = ui.interact(
        rect(
            1.,
            1.,
            r.width() - layout::CAPTION_RESERVED,
            t::TITLE_HEIGHT,
        ),
        ui.id().with("title"),
        egui::Sense::click_and_drag(),
    );
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if title.double_clicked() {
        ui.ctx().send_viewport_cmd(Cmd::Maximized(!maximized));
    } else if title.drag_started() {
        grab(ui.ctx(), Cmd::StartDrag);
    }
    for (i, label, command) in [
        (0, "Minimize", Cmd::Minimized(true)),
        (1, "Maximize", Cmd::Maximized(!maximized)),
        (2, "Close", Cmd::Close),
    ] {
        let x = r.right() - layout::CAPTION_RIGHT + i as f32 * layout::CAPTION_STEP;
        let response = click(
            ui,
            Rect::from_center_size(
                pos2(x, layout::CAPTION_Y),
                vec2(layout::CAPTION_SIZE, layout::CAPTION_SIZE),
            ),
            label,
            label,
        );
        response.clone().on_hover_cursor(egui::CursorIcon::Default);
        ui.painter().circle_filled(
            pos2(x, layout::CAPTION_Y),
            layout::CAPTION_RADIUS,
            crate::widgets::hover_color(
                &response,
                t::RAISED,
                if i == 2 {
                    t::interaction::CLOSE
                } else {
                    t::RAISED.blend(t::interaction::FRAME_HOVER)
                },
            ),
        );
        if response.clicked() {
            ui.ctx().send_viewport_cmd(command);
        }
        let p = ui.painter();
        match i {
            0 => {
                p.line_segment(
                    [
                        pos2(x - t::SPACE_3, layout::CAPTION_Y),
                        pos2(x + t::SPACE_3, layout::CAPTION_Y),
                    ],
                    Stroke::new(layout::GLYPH_STROKE, t::TEXT),
                );
            }
            1 => {
                p.rect_stroke(
                    rect(x - t::SPACE_3, layout::MAXIMIZE_TOP, t::SPACE_6, t::SPACE_6),
                    1,
                    Stroke::new(layout::GLYPH_STROKE, t::TEXT),
                    StrokeKind::Inside,
                );
            }
            _ => {
                for sign in [-1., 1.] {
                    p.line_segment(
                        [
                            pos2(
                                x - layout::CLOSE_HALF,
                                layout::CAPTION_Y - layout::CLOSE_HALF * sign,
                            ),
                            pos2(
                                x + layout::CLOSE_HALF,
                                layout::CAPTION_Y + layout::CLOSE_HALF * sign,
                            ),
                        ],
                        Stroke::new(layout::GLYPH_STROKE, t::TEXT),
                    );
                }
            }
        }
    }
}
pub fn finish(ui: &mut Ui, r: Rect) {
    // Above the shifted page layer, which would otherwise cover the edge.
    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("frame-edge"),
        ))
        .rect_stroke(
            r.shrink(1.),
            11,
            Stroke::new(t::BORDER, t::HIGHLIGHT),
            StrokeKind::Inside,
        );
    resize(ui, r);
}
/// Border resize handles, shared by every presentation. They sit on their own
/// top layer, show the resize cursor and start resizing on press.
pub fn resize(ui: &mut Ui, r: Rect) {
    if ui.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    use egui::{CursorIcon as C, ResizeDirection as D};
    let (w, h, e, c) = (r.width(), r.height(), layout::EDGE, layout::CORNER);
    let ui = &mut ui.new_child(
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::new(
                egui::Order::Tooltip,
                egui::Id::new("resize"),
            ))
            .max_rect(r),
    );
    for (i, (area, direction, cursor)) in [
        (rect(0., 0., c, c), D::NorthWest, C::ResizeNorthWest),
        (rect(w - c, 0., c, c), D::NorthEast, C::ResizeNorthEast),
        (rect(0., h - c, c, c), D::SouthWest, C::ResizeSouthWest),
        (rect(w - c, h - c, c, c), D::SouthEast, C::ResizeSouthEast),
        (rect(c, 0., w - 2. * c, e), D::North, C::ResizeNorth),
        (rect(c, h - e, w - 2. * c, e), D::South, C::ResizeSouth),
        (rect(0., c, e, h - 2. * c), D::West, C::ResizeWest),
        (rect(w - e, c, e, h - 2. * c), D::East, C::ResizeEast),
    ]
    .into_iter()
    .enumerate()
    {
        let response = ui
            .interact(area, ui.id().with(("resize", i)), egui::Sense::drag())
            .on_hover_cursor(cursor);
        if response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()) {
            grab(ui.ctx(), Cmd::BeginResize(direction));
        }
    }
}
pub fn pending(ui: &Ui, label: &str) {
    text(
        ui,
        layout::PLACEHOLDER_LEFT,
        layout::PLACEHOLDER_TOP,
        label,
        layout::MAXIMIZE_TOP,
        t::MUTED,
    );
}

pub(crate) fn grab(ctx: &egui::Context, command: Cmd) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("window-grab"), true));
    if !crate::tablet_input::grab(ctx, &command) {
        ctx.send_viewport_cmd(command);
    }
}
pub fn release_hook(ctx: &egui::Context, raw: &mut egui::RawInput) {
    if ctx
        .data_mut(|d| d.remove_temp::<bool>(egui::Id::new("window-grab")))
        .is_some()
    {
        raw.events.push(egui::Event::PointerButton {
            pos: pos2(-layout::RELEASE_DISTANCE, -layout::RELEASE_DISTANCE),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
    }
}
/// Rounds every rectangle that reaches a window corner, so sidebars, panels and
/// modal veils keep the window's transparent rounded corners.
pub fn round_window(ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let window = ctx.content_rect();
    let mut layers: Vec<egui::LayerId> = ctx.memory(|m| m.layer_ids().collect());
    layers.push(egui::LayerId::background());
    layers.dedup();
    // Shifted layers (pages) see the window in their own coordinates.
    let layers: Vec<(egui::LayerId, Rect)> = layers
        .into_iter()
        .map(|l| {
            (
                l,
                ctx.layer_transform_from_global(l)
                    .map_or(window, |t| t * window),
            )
        })
        .collect();
    ctx.graphics_mut(|g| {
        for (layer, window) in layers {
            let Some(list) = g.get_mut(layer) else {
                continue;
            };
            for i in 0..list.next_idx().0 {
                list.mutate_shape(egui::layers::ShapeIdx(i), |clipped| {
                    round_corners(&mut clipped.shape, window)
                });
            }
        }
    });
}
fn round_corners(shape: &mut egui::Shape, window: Rect) {
    match shape {
        egui::Shape::Vec(shapes) => shapes.iter_mut().for_each(|s| round_corners(s, window)),
        egui::Shape::Rect(rect) => {
            let r = rect.rect;
            let left = r.left() - window.left();
            let right = window.right() - r.right();
            let top = r.top() - window.top();
            let bottom = window.bottom() - r.bottom();
            // A corner inset by `d` px follows the window curve with radius - d.
            let fit = |a: f32, b: f32, current: u8| {
                let d = a.max(b);
                if d < 1.5 && a > -1. && b > -1. {
                    current.max(t::WINDOW_RADIUS.saturating_sub(d.max(0.).round() as u8))
                } else {
                    current
                }
            };
            let c = &mut rect.corner_radius;
            c.nw = fit(left, top, c.nw);
            c.ne = fit(right, top, c.ne);
            c.sw = fit(left, bottom, c.sw);
            c.se = fit(right, bottom, c.se);
        }
        _ => {}
    }
}
