//! Pen artwork, pressure canvas, sensitivity controls and mode selection.
use super::callout;
use crate::theme::pen_layout as layout;
use crate::{
    assets,
    i18n::Locale,
    theme as t,
    widgets::{self, card, click, heading, rect, text},
};
use egui::{pos2, Align2, Stroke, Ui};
use tablet_config::{PenMode, PenSettings};
pub const MOUSE_MODE_HINT: &str = "One cursor that changes over text and borders, like the mouse. The tip clicks; drawing apps get no pressure.";
pub const MODE_HINT:&str="The pen point is the cursor: each spot on the tablet is a spot on the screen. Drawing apps get pressure and tilt.";
#[derive(Default)]
pub struct State {
    pub strokes: Vec<Vec<(egui::Pos2, f32)>>,
    pub last_draw: Option<std::time::Instant>,
    pub scroll: f32,
    pub action: Option<usize>,
    /// Pen button (0 upper, 1 lower, 2 tip) whose actions panel is open.
    pub selected: Option<usize>,
    /// Extra height of the pen mode card when its hint wraps longer.
    pub mode_extra: f32,
}
pub fn show(
    ui: &mut Ui,
    locale: &Locale,
    pen: &mut PenSettings,
    state: &mut State,
    live: &tablet_config::PenState,
    product: u16,
) -> bool {
    // The compositor's pressure arrives with the pen motion; the daemon's lags it.
    let pen_pressure = crate::tablet_input::pressure(ui.ctx());
    let pressure = pen_pressure.unwrap_or(live.pressure.clamp(0., 1.) as f32);
    let bounds = crate::widgets::view(ui);
    let w = bounds.width();
    let h = bounds.bottom();
    let left = t::SIDEBAR + 1. + t::CONTENT_PAD;
    let strip = rect(
        left,
        layout::STRIP_TOP,
        w - left - layout::CONTENT_RIGHT,
        layout::STRIP_HEIGHT,
    );
    let art_height = (h - layout::ART_RESERVED_HEIGHT).min(t::PEN_MAX);
    let area = rect(
        left,
        layout::PAGE_TOP,
        strip.width(),
        h - layout::PAGE_BOTTOM_RESERVED,
    );
    let test_height = (h - layout::CANVAS_RESERVED_HEIGHT)
        .clamp(layout::CANVAS_MIN_HEIGHT, layout::CANVAS_MAX_HEIGHT);
    let total_height =
        layout::COMPACT_CONTENT_HEIGHT + test_height - layout::CANVAS_MIN_HEIGHT + state.mode_extra;
    // With the actions panel open only the pen shows, and it always fits.
    let max_scroll = if state.selected.is_some() {
        0.
    } else {
        (total_height - (h - layout::PAGE_TOP)).max(0.)
    };
    state.scroll = state.scroll.min(max_scroll);
    if crate::widgets::contains_pointer(ui, area) {
        state.scroll = (state.scroll - crate::widgets::page_wheel(ui)).clamp(0., max_scroll);
    }
    // The thumb sits in the page's right padding, clear of the cards and the pen.
    widgets::scroll_indicator(
        ui,
        ui.id().with("pen-scroll"),
        widgets::scroll_track(rect(left, layout::PAGE_TOP, w - left, h - layout::PAGE_TOP)),
        &mut state.scroll,
        max_scroll,
    );
    let mut content = ui.new_child(egui::UiBuilder::new().max_rect(area));
    content.set_clip_rect(area);
    let ui = &mut content;
    let y = layout::PAGE_TOP - state.scroll;
    // Cards on the left, flush with the strip, and the pen with its callout
    // cards in the column on their right. While a button's actions panel is
    // open on the right, the cards step aside and the pen takes their place.
    // 0 with the cards shown, 1 with the actions panel open; the cards slide
    // out to the left while the panel slides in from the right.
    let slide = panel_progress(ui.ctx(), state.selected.is_some());
    let side_x = left - slide * (t::CARD_WIDTH + layout::COLUMN_GAP);
    let lerp = |closed: f32, open: f32| closed + (open - closed) * slide;
    let column_left = lerp(left + t::CARD_WIDTH + layout::COLUMN_GAP, left);
    let column_right = lerp(
        strip.right(),
        strip.right() - layout::PANEL_WIDTH - layout::COLUMN_GAP,
    );
    let aspect = layout::ART_SOURCE_WIDTH / layout::ART_SOURCE_HEIGHT;
    // Fill the useful height while keeping the pen's native proportions.
    let art_height = if crate::shared::pen_art(product).0 == "pro-pen-2-wacom" {
        art_height / layout::PRO_PEN_BOUNDS[3]
    } else {
        art_height
    };
    let art_width = art_height * aspect;
    let group = art_width + layout::CALLOUT_REACH + layout::CALLOUT_WIDTH;
    let art_x = column_left + ((column_right - column_left - group) / 2.).max(0.);
    let art_top = y + t::SPACE_14;
    let art = rect(art_x, art_top, art_width, art_height);
    crate::shared::pen_glow(ui, product, art);
    let (key, bytes) = crate::shared::pen_art(product);
    assets::artwork(ui, &format!("pen-page-{key}"), bytes, art);
    let spots = crate::shared::pen_spots(product)
        .map(|(fx, fy)| pos2(art.left() + art.width() * fx, art.top() + art.height() * fy));
    let held = [live.upper_button, live.lower_button, live.tip_down];
    let names = ["Upper button", "Lower button", "Tip"];
    // Spots run top to bottom, so each card sits at its spot's height unless
    // the card above pushes it down.
    let actions = [&pen.upper_button, &pen.lower_button, &pen.tip];
    let labels = actions
        .map(|action| callout::label(ui, &locale.t(&crate::classic::devices::pen_label(action))));
    let heights = labels.clone().map(|label| callout::height(&label));
    let tops = callout::stack(&spots, &heights, art_top, h - t::CONTENT_PAD);
    for (index, spot) in spots.into_iter().enumerate() {
        let r = rect(
            art.right() + layout::CALLOUT_REACH,
            tops[index],
            layout::CALLOUT_WIDTH,
            heights[index],
        );
        if callout::card(
            ui,
            r,
            spot,
            pos2(spot.x + layout::CALLOUT_BEND, spot.y),
            &locale.t(names[index]),
            &crate::classic::devices::pen_action_id(actions[index]),
            labels[index].clone(),
            state.selected == Some(index),
            held[index],
        ) {
            state.action = Some(index);
        }
    }
    if slide >= 1. {
        return false;
    }
    ui.multiply_opacity(1. - slide);
    let top = y + t::SPACE_14;
    let test = rect(
        side_x,
        top,
        t::CARD_WIDTH,
        test_height + layout::TEST_CHROME_HEIGHT,
    );
    card(ui, test);
    heading(
        ui,
        side_x + layout::CARD_INSET,
        top + t::SPACE_18,
        &locale.t("Test area"),
    );
    ui.painter().text(
        pos2(test.right() - layout::CARD_INSET, top + t::SPACE_18),
        Align2::RIGHT_TOP,
        format!("{}%", (pressure * 100.).round()),
        t::font(t::SPACE_13),
        t::MUTED,
    );
    let canvas = rect(
        side_x + layout::CARD_INSET,
        top + layout::CANVAS_TOP,
        layout::CARD_INNER_WIDTH,
        test_height,
    );
    ui.painter().rect_filled(canvas, t::RADIUS, t::VIEW);
    ui.painter().rect_filled(
        rect(
            canvas.left(),
            canvas.bottom() - t::SPACE_8,
            canvas.width(),
            t::SPACE_8,
        ),
        t::SQUARE,
        t::VIEW,
    );
    let response = ui.interact(canvas, ui.id().with("pressure-canvas"), egui::Sense::drag());
    if response.drag_started() {
        state.strokes.push(Vec::new());
    }
    for p in widgets::drag_points(ui, &response) {
        if let Some(stroke) = state.strokes.last_mut() {
            // Like canvas pointermove: a still pen pressing harder adds no blob.
            let p = p - canvas.min.to_vec2();
            if stroke.last().is_some_and(|(last, _)| *last == p) {
                continue;
            }
            // Half width only for a mouse; a pen starts at its real pressure.
            let force = match pen_pressure {
                Some(p) => p,
                None if pressure > 0. => pressure,
                None => 0.5,
            };
            let target = layout::STROKE_MIN + force * layout::STROKE_RANGE;
            let width = stroke
                .last()
                .map(|(_, width)| width + (target - width) * layout::PRESSURE_SMOOTHING)
                .unwrap_or(target);
            stroke.push((p, width));
            state.last_draw = Some(std::time::Instant::now());
        }
    }
    if state
        .last_draw
        .is_some_and(|at| at.elapsed() > std::time::Duration::from_secs(2))
        && !response.dragged()
    {
        state.strokes.clear();
        state.last_draw = None;
    }
    if state.last_draw.is_some() {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(2));
    }
    let painter = ui.painter().with_clip_rect(canvas);
    for stroke in &state.strokes {
        for pair in stroke.windows(2) {
            widgets::round_segment(
                &painter,
                canvas.min + pair[0].0.to_vec2(),
                canvas.min + pair[1].0.to_vec2(),
                Stroke::new(pair[1].1, t::STROKE),
            );
        }
    }
    ui.painter().rect_filled(
        rect(canvas.left(), canvas.bottom(), canvas.width(), t::SPACE_6),
        t::SQUARE,
        t::TRACK,
    );
    ui.painter().rect_filled(
        rect(
            canvas.left(),
            canvas.bottom(),
            canvas.width()
                * ui.ctx().animate_value_with_time(
                    ui.id().with("pressure-width"),
                    pressure,
                    t::interaction::PRESSURE_TIME,
                ),
            t::SPACE_6,
        ),
        t::SQUARE,
        t::ACCENT,
    );
    text(
        ui,
        canvas.left(),
        canvas.bottom() + t::SPACE_15,
        &locale.t("Click"),
        t::SPACE_13,
        t::MUTED,
    );
    ui.painter().text(
        pos2(canvas.right(), canvas.bottom() + t::SPACE_15),
        Align2::RIGHT_TOP,
        locale.t("Maximum"),
        t::font(t::SPACE_13),
        t::MUTED,
    );
    let tip_y = test.bottom() + layout::CARD_GAP;
    card(ui, rect(side_x, tip_y, t::CARD_WIDTH, layout::TIP_HEIGHT));
    heading(
        ui,
        side_x + layout::CARD_INSET,
        tip_y + t::SPACE_18,
        &locale.t("Tip"),
    );
    let mut changed = false;
    changed |= widgets::slider(
        ui,
        locale,
        rect(
            side_x + layout::CARD_INSET,
            tip_y + layout::FIRST_SLIDER_TOP,
            layout::CARD_INNER_WIDTH,
            layout::SLIDER_HEIGHT,
        ),
        "Tip sensitivity",
        "How hard you need to press to reach full pressure. Soft gives thick, dark strokes with a light touch; Firm needs a heavier hand for the same stroke.",
        &mut pen.tip_sensitivity,
        1..=7,
        ("Soft", "Firm"),
    );
    changed|=widgets::slider(ui,locale,rect(side_x+layout::CARD_INSET,tip_y+layout::SECOND_SLIDER_TOP,layout::CARD_INNER_WIDTH,layout::SLIDER_HEIGHT),"Tip feel","How much pressure the tip needs before it registers a click. Light clicks at the slightest touch; Firm needs a deliberate press, which avoids accidental clicks while you hover or rest the pen.",&mut pen.click_threshold,1..=7,("Light","Firm"));
    let distances = [0, 1, 2, 3, 4, 6];
    let mut distance = distances
        .iter()
        .position(|v| *v == pen.double_click_mm)
        .unwrap_or(2) as u8;
    if widgets::slider(ui,locale,rect(side_x+layout::CARD_INSET,tip_y+layout::THIRD_SLIDER_TOP,layout::CARD_INNER_WIDTH,layout::SLIDER_HEIGHT),"Double-click distance","How far apart two quick taps can land and still count as a double-click. A larger distance makes double-clicking easier with a shaky hand; Off turns this help off.",&mut distance,0..=5,("Off","Large")){pen.double_click_mm=distances[usize::from(distance)];changed=true;}
    let mode_y = tip_y + layout::MODE_TOP;
    let hint=match pen.mode {PenMode::Pen=>MODE_HINT,PenMode::Mouse=>MOUSE_MODE_HINT,PenMode::Relative=>"Moves the cursor like a mouse: lift the pen and set it down again to keep going. Handy on large screens."};
    let galley = ui.fonts_mut(|f| {
        let mut job = egui::text::LayoutJob::simple(
            locale.t(hint),
            t::font(t::SPACE_13),
            t::MUTED,
            layout::CARD_INNER_WIDTH,
        );
        job.sections[0].format.line_height = Some(layout::HINT_LINE_HEIGHT);
        f.layout_job(job)
    });
    // The hint may wrap to more lines in the narrower card.
    let extra = (layout::MODE_HINT_TOP + galley.size().y + t::SPACE_18 - layout::CHIP_TOP).max(0.);
    state.mode_extra = extra;
    card(
        ui,
        rect(side_x, mode_y, t::CARD_WIDTH, layout::CHIP_TOP + extra),
    );
    heading(
        ui,
        side_x + layout::CARD_INSET,
        mode_y + t::SPACE_18,
        &locale.t("Pen mode"),
    );
    let segment = rect(
        side_x + layout::CARD_INSET,
        mode_y + layout::COLUMN_GAP,
        layout::CARD_INNER_WIDTH,
        layout::MODE_HEIGHT,
    );
    ui.painter().rect_filled(segment, t::RADIUS, t::VIEW);
    let modes = [
        (PenMode::Pen, "Pen"),
        (PenMode::Mouse, "Mouse"),
        (PenMode::Relative, "Relative mouse"),
    ];
    // Each item fits its label and the rest is shared, so long labels still fit.
    let label_widths =
        modes.map(|(_, label)| widgets::text_width(ui, &locale.t(label), t::SPACE_14));
    let spare = ((segment.width() - 2. * t::SPACE_3 - label_widths.iter().sum::<f32>())
        / modes.len() as f32)
        .max(0.);
    let mut item_x = segment.left() + t::SPACE_3;
    for (i, (mode, label)) in modes.into_iter().enumerate() {
        let r = rect(
            item_x,
            segment.top() + t::SPACE_3,
            label_widths[i] + spare,
            layout::MODE_ITEM_HEIGHT,
        );
        item_x = r.right();
        let response = click(ui, r, ("mode", i), label);
        if pen.mode == mode {
            ui.painter().rect_filled(r, t::CONTROL_RADIUS, t::RAISED);
        }
        ui.painter().text(
            r.center(),
            Align2::CENTER_CENTER,
            locale.t(label),
            t::font(t::SPACE_14),
            if pen.mode == mode {
                t::TEXT
            } else {
                crate::widgets::hover_color(&response, t::MUTED, t::TEXT)
            },
        );
        if response.clicked() {
            pen.mode = mode;
            changed = true;
        }
    }
    ui.painter().galley(
        pos2(side_x + layout::CARD_INSET, mode_y + layout::MODE_HINT_TOP),
        galley,
        t::MUTED,
    );
    let restore = rect(
        side_x + layout::RESTORE_LEFT,
        mode_y + layout::RESTORE_TOP + extra,
        layout::RESTORE_WIDTH,
        layout::RESTORE_HEIGHT,
    );
    let response = click(ui, restore, "restore", "Restore defaults");
    if response.hovered() {
        ui.painter()
            .rect_filled(restore, t::CONTROL_RADIUS, t::RAISED);
    }
    text(
        ui,
        restore.left() + t::SPACE_10,
        restore.top() + t::SPACE_6,
        &locale.t("Default"),
        t::SPACE_14,
        crate::widgets::hover_color(&response, t::MUTED, t::TEXT),
    );
    if response.clicked() {
        *pen = PenSettings::default();
        state.strokes.clear();
        changed = true;
    }
    changed
}

/// Progress of the pen actions panel: 0 closed, 1 open, eased between.
pub fn panel_progress(ctx: &egui::Context, open: bool) -> f32 {
    // Linear progress, eased in and out so the slide starts and lands softly.
    egui::emath::easing::cubic_in_out(ctx.animate_bool_with_time(
        egui::Id::new("pen-actions-panel"),
        open,
        t::interaction::PANEL_SLIDE_TIME,
    ))
}
