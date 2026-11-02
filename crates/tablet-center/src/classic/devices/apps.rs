//! App profiles: icon loading, picking and adding apps.
use super::*;

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

pub(super) fn app_dialog(ui: &mut Ui, l: &Locale, c: &mut Config, s: &mut State) {
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
