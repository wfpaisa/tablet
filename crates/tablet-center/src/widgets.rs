//! Pixel-positioned classic controls with native egui interaction.
use crate::theme::control_layout as layout;
use crate::{i18n::Locale, theme as t};
use egui::{pos2, vec2, Align2, Rect, Sense, Stroke, StrokeKind, Ui};
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}
/// Child layer that keeps `ui`'s page shift, for overlays drawn by a page.
pub fn layer(ui: &Ui, order: egui::Order, id: egui::Id) -> egui::LayerId {
    let layer = egui::LayerId::new(order, id);
    if let Some(transform) = ui.ctx().layer_transform_to_global(ui.layer_id()) {
        ui.ctx().set_transform_layer(layer, transform);
    }
    layer
}
/// Accessibility subtrees drawn on shifted layers. egui reports their bounds
/// in layer coordinates; this moves them to the screen at the end of the pass.
#[derive(Default)]
struct ShiftedAccess {
    roots: Vec<(egui::Id, egui::Vec2)>,
}
impl egui::Plugin for ShiftedAccess {
    fn debug_name(&self) -> &'static str {
        "shifted-access"
    }
    fn output_hook(&mut self, _: &egui::Context, output: &mut egui::FullOutput) {
        let roots = std::mem::take(&mut self.roots);
        let Some(update) = output.platform_output.accesskit_update.as_mut() else {
            return;
        };
        let index: std::collections::HashMap<_, _> = update
            .nodes
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (*id, i))
            .collect();
        let mut done = std::collections::HashSet::new();
        for (root, offset) in roots {
            let mut stack = vec![root.accesskit_id()];
            while let Some(id) = stack.pop() {
                let Some(&i) = index.get(&id) else {
                    continue;
                };
                if !done.insert(id) {
                    continue;
                }
                let node = &mut update.nodes[i].1;
                if let Some(b) = node.bounds() {
                    let (dx, dy) = (f64::from(offset.x), f64::from(offset.y));
                    node.set_bounds(egui::accesskit::Rect {
                        x0: b.x0 + dx,
                        y0: b.y0 + dy,
                        x1: b.x1 + dx,
                        y1: b.y1 + dy,
                    });
                }
                stack.extend_from_slice(node.children());
            }
        }
    }
}
/// Reports `ui` and its widgets to accessibility at their screen position
/// when `ui` sits on a shifted layer.
pub fn shifted_access(ui: &Ui) {
    let ctx = ui.ctx();
    let Some(transform) = ctx.layer_transform_to_global(ui.layer_id()) else {
        return;
    };
    if ctx.plugin_opt::<ShiftedAccess>().is_none() {
        ctx.add_plugin(ShiftedAccess::default());
    }
    ctx.plugin::<ShiftedAccess>()
        .lock()
        .roots
        .push((ui.unique_id(), transform.translation));
}
/// Converts a screen position into `ui`'s layer coordinates.
fn local(ui: &Ui, p: egui::Pos2) -> egui::Pos2 {
    ui.ctx()
        .layer_transform_from_global(ui.layer_id())
        .map_or(p, |t| t * p)
}
/// Pointer position in `ui`'s layer coordinates.
pub fn pointer(ui: &Ui) -> Option<egui::Pos2> {
    ui.input(|i| i.pointer.hover_pos()).map(|p| local(ui, p))
}
/// Last press or hover position in `ui`'s layer coordinates.
pub fn press(ui: &Ui) -> Option<egui::Pos2> {
    ui.input(|i| i.pointer.interact_pos()).map(|p| local(ui, p))
}
/// Whether the pointer is over `r` and no popup or window covers it. Unlike
/// `Ui::rect_contains_pointer`, this works on shifted page layers.
pub fn contains_pointer(ui: &Ui, r: Rect) -> bool {
    let Some(screen) = ui.input(|i| i.pointer.hover_pos()) else {
        return false;
    };
    ui.ctx()
        .layer_id_at(screen)
        .is_none_or(|l| l.order <= ui.layer_id().order)
        && r.contains(local(ui, screen))
}
/// The whole window in `ui`'s layer coordinates.
pub fn window(ui: &Ui) -> Rect {
    let r = ui.ctx().content_rect();
    ui.ctx()
        .layer_transform_from_global(ui.layer_id())
        .map_or(r, |t| t * r)
}
/// The area a page lays out in, in its own coordinates: the window right of
/// the page's horizontal shift, with the page's original top at `t::HEADER`.
pub fn view(ui: &Ui) -> Rect {
    let r = ui.ctx().content_rect();
    match ui.ctx().layer_transform_to_global(ui.layer_id()) {
        Some(to) => {
            to.inverse() * Rect::from_min_max(pos2(r.left() + to.translation.x, r.top()), r.max)
        }
        None => r,
    }
}
pub fn text(ui: &Ui, x: f32, y: f32, s: &str, size: f32, color: egui::Color32) {
    ui.painter()
        .text(pos2(x, y), Align2::LEFT_TOP, s, t::font(size), color);
}
/// Single-line text box filling `r`: rounded, bordered (accent while focused)
/// and with its text centered vertically. Keeps at most `max_chars` characters.
pub fn text_field(
    ui: &mut Ui,
    r: Rect,
    value: &mut String,
    hint: &str,
    max_chars: usize,
) -> egui::Response {
    let font = t::font(layout::FIELD_FONT);
    let row = ui.fonts_mut(|f| f.row_height(&font));
    // The box is painted under the editor once its focus is known.
    let background = ui.painter().add(egui::Shape::Noop);
    let response = ui.put(
        r,
        egui::TextEdit::singleline(value)
            .font(font)
            .text_color(t::TEXT)
            .hint_text(egui::RichText::new(hint).color(t::FAINT))
            .char_limit(max_chars)
            // With a custom frame egui ignores `margin`; the padding goes here.
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::from(vec2(
                layout::FIELD_PAD_X,
                ((r.height() - row) / 2.).max(0.),
            ))))
            .desired_width(r.width()),
    );
    let border = if response.has_focus() {
        t::ACCENT
    } else {
        hover_color(&response, t::LINE, t::HIGHLIGHT)
    };
    ui.painter().set(
        background,
        egui::epaint::RectShape::new(
            r,
            layout::FIELD_RADIUS,
            t::VIEW,
            Stroke::new(t::BORDER, border),
            StrokeKind::Inside,
        ),
    );
    response
}
/// Width of `s` in the regular font at `size`.
pub fn text_width(ui: &Ui, s: &str, size: f32) -> f32 {
    ui.fonts_mut(|f| f.layout_no_wrap(s.into(), t::font(size), t::TEXT).size().x)
}
/// Draws `s` at (x, y) when it fits `max_width`; longer translations wrap at
/// `max_width` into slightly smaller lines centered vertically in `row`.
pub fn fit_text(
    ui: &Ui,
    pos: egui::Pos2,
    s: &str,
    size: f32,
    max_width: f32,
    row: Rect,
    color: egui::Color32,
) {
    if text_width(ui, s, size) <= max_width {
        return text(ui, pos.x, pos.y, s, size, color);
    }
    let job = egui::text::LayoutJob::simple(
        s.into(),
        t::font(size * layout::WRAP_SCALE),
        color,
        max_width.max(1.),
    );
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let y = row.center().y - galley.size().y / 2.;
    ui.painter().galley(pos2(pos.x, y), galley, color);
}
/// Text wrapped at `width` to at most two lines, the second cut with "…".
/// Laid out in the placeholder color, so the caller picks it when painting.
/// Round "?" mark; accent ring while `active`.
pub fn help_mark(ui: &Ui, center: egui::Pos2, active: bool) {
    ui.painter().circle_stroke(
        center,
        layout::HELP_RADIUS,
        Stroke::new(
            layout::HELP_STROKE,
            if active { t::ACCENT } else { t::MUTED },
        ),
    );
    ui.painter().text(
        center,
        Align2::CENTER_CENTER,
        "?",
        t::font(t::SPACE_12),
        if active { t::TEXT } else { t::MUTED },
    );
}
pub fn two_lines(ui: &Ui, s: &str, size: f32, width: f32) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(
        s.to_owned(),
        t::font(size),
        egui::Color32::PLACEHOLDER,
        width.max(1.),
    );
    job.wrap.max_rows = 2;
    ui.fonts_mut(|f| f.layout_job(job))
}
pub fn heading(ui: &Ui, x: f32, y: f32, s: &str) {
    let mut job = egui::text::LayoutJob::simple(
        s.to_uppercase(),
        t::semibold(t::SPACE_12),
        t::MUTED,
        f32::INFINITY,
    );
    for section in &mut job.sections {
        section.format.extra_letter_spacing = layout::HEADING_TRACKING;
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    ui.painter().galley(pos2(x, y), galley, t::MUTED);
}
pub fn card(ui: &Ui, r: Rect) {
    ui.painter().rect(
        r,
        t::CARD_RADIUS,
        t::PANEL,
        Stroke::new(t::BORDER, t::LINE),
        StrokeKind::Inside,
    );
}
pub fn click(
    ui: &mut Ui,
    r: Rect,
    id: impl std::hash::Hash + std::fmt::Debug,
    label: &str,
) -> egui::Response {
    let response = ui.interact(r, ui.id().with(id), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
    if !label.is_empty() && response.enabled() {
        response
            .clone()
            .on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    response
}
/// Pass in which a modal dialog was last drawn.
fn modal_id() -> egui::Id {
    egui::Id::new("modal-shown")
}
/// Marks a modal dialog as drawn in this pass.
pub fn mark_modal(ctx: &egui::Context) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(modal_id(), pass));
}
/// Wheel delta for page scrolling. Pages draw before their dialogs, so a dialog
/// drawn last pass keeps the wheel for itself.
pub fn page_wheel(ui: &Ui) -> f32 {
    let pass = ui.ctx().cumulative_pass_nr();
    let modal = ui
        .ctx()
        .data(|d| d.get_temp::<u64>(modal_id()))
        .is_some_and(|last| last + 1 >= pass);
    if modal {
        0.
    } else {
        ui.input(|i| i.smooth_scroll_delta.y)
    }
}
/// Track for a scroll thumb along the right edge of `area`.
pub fn scroll_track(area: Rect) -> Rect {
    let w = t::scroll::WIDTH;
    let inset = t::scroll::INSET;
    Rect::from_min_size(
        pos2(area.right() - inset - w, area.top() + inset),
        vec2(w, (area.height() - 2. * inset).max(0.)),
    )
}
/// The app's one scroll indicator: a thin thumb with no track, shown whenever
/// the content overflows. Dragging or clicking it scrolls; returns true then.
pub fn scroll_indicator(
    ui: &Ui,
    id: egui::Id,
    track: Rect,
    offset: &mut f32,
    max_scroll: f32,
) -> bool {
    if max_scroll <= 0. || track.height() <= 0. {
        return false;
    }
    let visible = track.height();
    let thumb = (visible * visible / (visible + max_scroll)).max(t::scroll::MIN_THUMB);
    let travel = (visible - thumb).max(1.);
    let hit = track.expand2(vec2(t::scroll::HIT_PAD, 0.));
    let response = ui.interact(hit, id.with("scroll-indicator"), Sense::click_and_drag());
    let mut changed = false;
    if let Some(p) = response
        .interact_pointer_pos()
        .filter(|_| response.dragged() || response.clicked())
    {
        *offset = ((p.y - track.top() - thumb / 2.) / travel * max_scroll).clamp(0., max_scroll);
        changed = true;
    }
    let top = track.top() + offset.clamp(0., max_scroll) / max_scroll * travel;
    let color = if response.dragged() {
        t::scroll::ACTIVE
    } else {
        hover_color(&response, t::scroll::THUMB, t::scroll::ACTIVE)
    };
    ui.ctx()
        .layer_painter(ui.layer_id())
        .with_clip_rect(hit)
        .rect_filled(
            Rect::from_min_size(pos2(track.left(), top), vec2(track.width(), thumb)),
            t::scroll::RADIUS,
            color,
        );
    changed
}
/// [`scroll_indicator`] for an egui `ScrollArea` drawn with its own bar hidden.
pub fn area_indicator<R>(ui: &Ui, output: &egui::scroll_area::ScrollAreaOutput<R>) {
    let max_scroll = (output.content_size.y - output.inner_rect.height()).max(0.);
    let mut offset = output.state.offset.y;
    if scroll_indicator(
        ui,
        output.id,
        scroll_track(output.inner_rect),
        &mut offset,
        max_scroll,
    ) {
        let mut state = output.state;
        state.offset.y = offset;
        state.store(ui.ctx(), output.id);
        ui.ctx().request_repaint();
    }
}
/// A click outside `r`, skipping the pass the dialog opened in: the click that
/// opened it lands outside too.
pub fn clicked_outside(ui: &Ui, id: &str, r: Rect) -> bool {
    // Pass in which the dialog was last drawn; a gap means it just opened.
    let id = egui::Id::new(("dialog-shown", id));
    let pass = ui.ctx().cumulative_pass_nr();
    let last = ui.ctx().data_mut(|d| {
        let last = d.get_temp::<u64>(id);
        d.insert_temp(id, pass);
        last
    });
    last.is_some_and(|last| last + 1 >= pass)
        && ui.input(|i| i.pointer.primary_clicked())
        && press(ui).is_some_and(|p| !r.contains(p))
}
/// Blend a CSS state without changing the resting paint.
pub fn hover_color(
    response: &egui::Response,
    rest: egui::Color32,
    hover: egui::Color32,
) -> egui::Color32 {
    if response.enabled() && (response.hovered() || response.is_pointer_button_down_on()) {
        hover
    } else {
        rest
    }
}
pub fn animated_color(
    response: &egui::Response,
    rest: egui::Color32,
    hover: egui::Color32,
    seconds: f32,
) -> egui::Color32 {
    let amount = response.ctx.animate_bool_with_time(
        response.id.with("css-hover"),
        response.enabled() && response.hovered(),
        seconds,
    );
    egui::Color32::from(egui::lerp(
        egui::Rgba::from(rest)..=egui::Rgba::from(hover),
        amount,
    ))
}
pub fn focus_visible(ui: &Ui, response: &egui::Response) -> bool {
    response.has_focus()
        && ui
            .ctx()
            .data(|d| d.get_temp::<bool>(egui::Id::new("css-keyboard-focus")))
            .unwrap_or(false)
}
pub fn focus(ui: &Ui, response: &egui::Response, radius: u8, color: egui::Color32) {
    if focus_visible(ui, response) {
        ui.painter().rect_stroke(
            response.rect.expand(t::interaction::FOCUS_OFFSET),
            radius,
            Stroke::new(t::interaction::FOCUS_WIDTH, color),
            StrokeKind::Outside,
        );
    }
}

#[allow(clippy::too_many_arguments)] // Mirrors one labeled CSS slider, including its help and scale.
pub fn slider(
    ui: &mut Ui,
    locale: &Locale,
    r: Rect,
    label: &str,
    help: &str,
    value: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    ends: (&str, &str),
) -> bool {
    let title = locale.t(label);
    text(
        ui,
        r.left(),
        r.top(),
        &title,
        if help.is_empty() {
            t::SPACE_16
        } else {
            layout::LABEL_FONT
        },
        t::TEXT,
    );
    if !help.is_empty() {
        let width = ui.fonts_mut(|f| {
            f.layout_no_wrap(title.clone(), t::font(layout::LABEL_FONT), t::TEXT)
                .size()
                .x
        });
        let center = pos2(
            r.left() + width + layout::HELP_GAP,
            r.top() + layout::HELP_CENTER_Y,
        );
        let help_response = click(
            ui,
            Rect::from_center_size(center, vec2(layout::HELP_TARGET, layout::HELP_TARGET)),
            (label, "help"),
            &locale.t("More information"),
        );
        help_response
            .clone()
            .on_hover_cursor(egui::CursorIcon::Help);
        let pinned_id = help_response.id.with("help-pinned");
        let mut pinned = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(pinned_id))
            .unwrap_or(false);
        if help_response.clicked() {
            pinned = !pinned;
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape) || i.pointer.any_pressed()) {
            pinned = false;
        }
        ui.ctx().data_mut(|d| d.insert_temp(pinned_id, pinned));
        let active = help_response.hovered() || focus_visible(ui, &help_response) || pinned;
        help_mark(ui, center, active);
        let help_text = locale.t(help);

        let opacity = ui.ctx().animate_bool_with_time(
            help_response.id.with("help-opacity"),
            active,
            t::interaction::FAST_SWITCH_TIME,
        );
        egui::Popup::from_toggle_button_response(&help_response)
            .open(active || opacity > 0.)
            .gap(t::SPACE_8 - t::SPACE_4 * (1. - opacity))
            .width(layout::HELP_WIDTH)
            .frame(
                egui::Frame::new()
                    .fill(t::RAISED.gamma_multiply(opacity))
                    .stroke(Stroke::new(t::BORDER, t::LINE.gamma_multiply(opacity)))
                    .corner_radius(t::RADIUS)
                    .inner_margin(12),
            )
            .show(|ui| {
                ui.set_opacity(opacity);
                ui.label(
                    egui::RichText::new(help_text)
                        .font(t::font(layout::THUMB_INSET))
                        .color(t::TEXT),
                );
            });
    }
    let plain_offset = if help.is_empty() { t::SPACE_6 } else { 0. };
    let track = rect(
        r.left() + t::SPACE_2,
        r.top() + plain_offset - if help.is_empty() { t::BORDER } else { 0. } + layout::TRACK_TOP,
        r.width(),
        t::SPACE_5,
    );
    let response = ui.interact(
        rect(
            r.left(),
            r.top() + plain_offset + layout::SLIDER_TARGET_TOP,
            r.width() + t::SPACE_4,
            layout::SLIDER_TARGET_HEIGHT,
        ),
        ui.id().with(label),
        Sense::click_and_drag(),
    );
    response.widget_info(|| egui::WidgetInfo::slider(true, f64::from(*value), &title));
    response
        .clone()
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if response.clicked() {
        response.request_focus();
    }
    let min = *range.start();
    let max = *range.end();
    let mut changed = false;
    if let Some(p) = response
        .interact_pointer_pos()
        .filter(|_| response.dragged() || response.clicked())
    {
        let next = (f32::from(min)
            + (p.x - (r.left() + layout::THUMB_INSET)) / (r.width() - layout::THUMB_DIAMETER)
                * f32::from(max - min))
        .round()
        .clamp(f32::from(min), f32::from(max)) as u8;
        changed = next != *value;
        *value = next;
    }
    if response.has_focus() {
        let delta = ui.input(|i| {
            i.key_pressed(egui::Key::ArrowRight) as i16 - i.key_pressed(egui::Key::ArrowLeft) as i16
        });
        let next = (i16::from(*value) + delta).clamp(i16::from(min), i16::from(max)) as u8;
        changed |= next != *value;
        *value = next;
    }
    let fraction = f32::from(value.saturating_sub(min)) / f32::from(max - min);
    let thumb_x = r.left() + layout::THUMB_INSET + fraction * (r.width() - layout::THUMB_DIAMETER);
    // Accent ring and dot over a bare track.
    ui.painter().rect_filled(track, t::TRACK_RADIUS, t::TRACK);
    let center = pos2(
        thumb_x,
        r.top() + plain_offset - if help.is_empty() { t::BORDER } else { 0. }
            + layout::THUMB_CENTER_Y,
    );
    ui.painter()
        .circle_filled(center, layout::RING_RADIUS, t::PANEL);
    ui.painter().circle_stroke(
        center,
        layout::RING_RADIUS,
        Stroke::new(t::BORDER, t::ACCENT),
    );
    ui.painter()
        .circle_filled(center, layout::DOT_RADIUS, t::ACCENT);
    for i in 0..=max - min {
        ui.painter().circle_filled(
            pos2(
                r.left()
                    + layout::TICK_INSET
                    + f32::from(i) * (r.width() - layout::TICK_INSETS) / f32::from(max - min),
                r.top() + plain_offset + layout::TICKS_Y,
            ),
            layout::HELP_STROKE,
            t::FAINT,
        );
    }
    text(
        ui,
        r.left(),
        r.top() + plain_offset + layout::SCALE_TOP,
        &locale.t(ends.0),
        if help.is_empty() {
            t::SPACE_15
        } else {
            layout::THUMB_INSET
        },
        t::MUTED,
    );
    ui.painter().text(
        pos2(r.right(), r.top() + plain_offset + layout::SCALE_TOP),
        Align2::RIGHT_TOP,
        locale.t(ends.1),
        t::font(if help.is_empty() {
            t::SPACE_15
        } else {
            layout::THUMB_INSET
        }),
        t::MUTED,
    );
    changed
}

/// CSS switch transitions share the same position and background progress.
pub fn switch_progress(
    ui: &Ui,
    id: impl std::hash::Hash + std::fmt::Debug,
    on: bool,
    seconds: f32,
) -> f32 {
    ui.ctx()
        .animate_bool_with_time(ui.id().with(("css-switch", id)), on, seconds)
}

/// Evaluate CSS cubic-bezier timing by inverting its time coordinate.
pub fn css_ease(time: f32, curve: [f32; 4]) -> f32 {
    let time = time.clamp(0., 1.);
    if time == 0. || time == 1. {
        return time;
    }
    let cubic = |t: f32, a: f32, b: f32| {
        3. * (1. - t).powi(2) * t * a + 3. * (1. - t) * t * t * b + t.powi(3)
    };
    let (mut low, mut high) = (0., 1.);
    for _ in 0..16 {
        let mid = (low + high) / 2.;
        if cubic(mid, curve[0], curve[2]) < time {
            low = mid;
        } else {
            high = mid;
        }
    }
    cubic((low + high) / 2., curve[1], curve[3])
}
/// Every pointer position of this frame while `response` is dragged, so fast
/// pen strokes keep all their samples instead of one point per frame.
pub fn drag_points(ui: &Ui, response: &egui::Response) -> Vec<egui::Pos2> {
    if !response.dragged() {
        return Vec::new();
    }
    // Events are in window coordinates; shifted page layers draw in their own.
    let to_layer = ui
        .ctx()
        .layer_transform_from_global(ui.layer_id())
        .unwrap_or_default();
    let mut points: Vec<egui::Pos2> = ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::PointerMoved(p) => Some(to_layer * *p),
                _ => None,
            })
            .collect()
    });
    if points.is_empty() {
        points.extend(response.interact_pointer_pos());
    }
    points
}
/// Draws one stroke segment with round ends, like a canvas `lineCap: round`.
pub fn round_segment(painter: &egui::Painter, a: egui::Pos2, b: egui::Pos2, stroke: Stroke) {
    painter.line_segment([a, b], stroke);
    painter.circle_filled(b, stroke.width / 2., stroke.color);
}
