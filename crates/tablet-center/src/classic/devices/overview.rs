//! ExpressKeys overview and Touch Ring pages.
use super::*;

pub(super) fn touch_ring(
    ui: &mut Ui,
    l: &Locale,
    c: &mut Config,
    status: &Response,
    s: &mut State,
) {
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
    let fractions = if tablet_config::family(product) == Some(tablet_config::Family::Pro2013S) {
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
            crate::classic::callout::label(ui, &l.t(name))
        })
        .collect();
    let heights: Vec<_> = labels
        .iter()
        .map(|g| crate::classic::callout::height(g))
        .collect();
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
        let clicked = crate::classic::callout::card(
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
                crate::classic::settings::shadow(
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
                            "../../../../../assets/icons/select-check.svg"
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

pub(super) fn keys_art_rect(product: u16, w: f32, h: f32, slide: f32) -> egui::Rect {
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

pub(super) fn wrapped(ui: &Ui, r: egui::Rect, s: &str, size: f32, color: egui::Color32) {
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
pub(super) fn overview(ui: &mut Ui, l: &Locale, c: &mut Config, status: &Response, s: &mut State) {
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
