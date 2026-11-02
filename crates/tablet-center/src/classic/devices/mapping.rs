//! Mapping page: display, portion and orientation of the pen area.
use super::*;

pub(super) fn mapping(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response) {
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
pub(super) fn subrect(b: egui::Rect, p: Option<tablet_config::Rect>) -> egui::Rect {
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
pub(super) fn portion(ui: &mut Ui, id: &str, b: egui::Rect, p: &mut Option<tablet_config::Rect>) {
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
pub(super) fn toggle(
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
