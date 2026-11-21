//! Persistent stdin shortcut host and transparent native overlay viewports.
mod mode_osd;
pub use mode_osd::{draw_mode_osd, ModeOsd};

use crate::{classic::shortcuts, i18n::Locale, theme};
use egui::{pos2, vec2, Pos2, Rect, Vec2};
use std::{
    io::BufRead,
    sync::{mpsc, Arc, OnceLock},
    time::{Duration, Instant},
};
use tablet_config::{Config, KeyAction, Shortcut};

#[derive(Clone, Debug, PartialEq)]
pub struct ShowRequest {
    pub name: Shortcut,
    pub at: Option<Pos2>,
    pub centered: bool,
}
impl ShowRequest {
    pub fn parse(line: &str) -> Option<Self> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let name = match v["name"].as_str()? {
            "express_menu" => Shortcut::ExpressMenu,
            "radial_menu" => Shortcut::RadialMenu,
            "app_shortcuts" => Shortcut::AppShortcuts,
            "keypad" => Shortcut::Keypad,
            _ => return None,
        };
        let at = v.get("at").and_then(|at| {
            Some(pos2(
                at.get(0)?.as_f64()? as f32,
                at.get(1)?.as_f64()? as f32,
            ))
        });
        Some(Self {
            name,
            at,
            centered: v["centered"].as_bool().unwrap_or(false),
        })
    }
}
#[derive(Clone, Debug)]
pub struct Monitor {
    pub bounds: Rect,
}
/// Account for panels excluded by maximization, then clamp all menu edges.
/// `cursor` is the pointer inside the window, for when tabletd sent no point.
pub fn place(
    request: &ShowRequest,
    at_cursor: bool,
    cursor: Option<Pos2>,
    monitors: &[Monitor],
    viewport: Vec2,
    menu: Vec2,
) -> Pos2 {
    let monitor = request
        .at
        .and_then(|p| monitors.iter().find(|m| m.bounds.contains(p)));
    let target = if at_cursor && !request.centered {
        match (request.at, monitor, cursor) {
            (Some(p), Some(m), _) => p - m.bounds.min.to_vec2() - (m.bounds.size() - viewport),
            (None, _, Some(p)) => p,
            _ => pos2(viewport.x / 2., viewport.y / 2.),
        }
    } else {
        pos2(viewport.x / 2., viewport.y / 2.)
    };
    let clamp = |v: f32, size: f32, max: f32| {
        v.max(size / 2. + theme::shortcuts_layout::EDGE).min(
            (max - size / 2. - theme::shortcuts_layout::EDGE)
                .max(size / 2. + theme::shortcuts_layout::EDGE),
        )
    };
    pos2(
        clamp(target.x, menu.x, viewport.x) - menu.x / 2.,
        clamp(target.y, menu.y, viewport.y) - menu.y / 2.,
    )
}
enum HostMessage {
    Shortcut(ShowRequest, Box<Config>),
    Mode(ModeOsd, Arc<Locale>),
}

struct ModeNotice {
    modes: ModeOsd,
    locale: Arc<Locale>,
    opened: Arc<OnceLock<Instant>>,
    presented: bool,
}

pub struct Host {
    pub config: Config,
    pub pending: Option<ShowRequest>,
    pub shown: bool,
    monitors: Vec<Monitor>,
    primary: Option<Monitor>,
    messages: mpsc::Receiver<HostMessage>,
    mode_notice: Option<ModeNotice>,
    eof: mpsc::Receiver<()>,
    opened: Instant,
    closing: Option<(Instant, Option<KeyAction>)>,
    focused: std::sync::Arc<std::sync::atomic::AtomicBool>,
    locale: std::sync::Arc<Locale>,
    events: mpsc::Receiver<Option<KeyAction>>,
    event_tx: mpsc::Sender<Option<KeyAction>>,
}
impl Host {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::assets::install(&cc.egui_ctx);
        let monitor_bounds = |m: winit::monitor::MonitorHandle| {
            let scale = m.scale_factor() as f32;
            let p = m.position();
            let size = m.size();
            Monitor {
                bounds: Rect::from_min_size(
                    pos2(p.x as f32 / scale, p.y as f32 / scale),
                    vec2(size.width as f32 / scale, size.height as f32 / scale),
                ),
            }
        };
        let mut primary = None;
        let monitors = cc.winit_window().map_or_else(Vec::new, |window| {
            primary = window
                .primary_monitor()
                .or_else(|| window.current_monitor())
                .map(monitor_bounds);
            window.available_monitors().map(monitor_bounds).collect()
        });
        let (tx, messages) = mpsc::channel();
        let (end, eof) = mpsc::channel();
        let ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            for line in std::io::stdin().lock().lines() {
                let Ok(line) = line else { break };
                // Read fresh settings off the event loop.
                let message = if let Some(mode) = ModeOsd::parse(&line) {
                    let language = std::fs::read_to_string(
                        tablet_config::config_path().with_file_name("language"),
                    )
                    .unwrap_or_else(|_| "en".into());
                    HostMessage::Mode(mode, Locale::new(language.trim()).into())
                } else if let Some(request) = ShowRequest::parse(&line) {
                    HostMessage::Shortcut(request, Box::new(tablet_config::load()))
                } else {
                    continue;
                };
                if tx.send(message).is_err() {
                    return;
                }
                ctx.request_repaint();
            }
            let _ = end.send(());
            ctx.request_repaint();
        });
        let (event_tx, events) = mpsc::channel();
        Self {
            config: Config::default(),
            pending: None,
            shown: false,
            monitors,
            primary,
            messages,
            mode_notice: None,
            eof,
            opened: Instant::now(),
            closing: None,
            focused: Default::default(),
            locale: Locale::new("en").into(),
            events,
            event_tx,
        }
    }
    pub fn pending_shortcut(&self) -> Option<&ShowRequest> {
        self.pending.as_ref()
    }
    pub fn close_shortcut(&mut self) {
        if self.closing.is_none() {
            self.closing = Some((Instant::now(), None));
        }
    }
    pub fn run_shortcut_action(&mut self, action: KeyAction) {
        if action != KeyAction::Disabled && self.closing.is_none() {
            self.closing = Some((Instant::now(), Some(action)));
        }
    }
    fn receive(&mut self, request: ShowRequest, config: Config) {
        let replacing = self.shown;
        if self.shown
            && self.closing.is_none()
            && self
                .pending
                .as_ref()
                .is_some_and(|p| p.name == request.name)
        {
            self.close_shortcut();
            return;
        }
        self.config = config;
        self.pending = Some(request);
        self.shown = true;
        self.focused
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.closing = None;
        self.opened = Instant::now()
            - if replacing {
                Duration::from_secs_f32(theme::interaction::OVERLAY_MAP_DELAY)
            } else {
                Duration::ZERO
            };
    }
    fn mode_viewport() -> egui::ViewportId {
        egui::ViewportId::from_hash_of("mode-osd")
    }

    fn show_mode_notice(&mut self, ctx: &egui::Context) {
        let Some(notice) = &mut self.mode_notice else {
            return;
        };
        let mut builder = egui::ViewportBuilder::default()
            .with_title("Tablet Center mode")
            .with_app_id("tablet-center-mode-osd")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_active(false)
            .with_mouse_passthrough(true)
            .with_window_type(egui::X11WindowType::Notification)
            .with_override_redirect(true)
            .with_maximized(true);
        if let Some(monitor) = self.primary.as_ref().or_else(|| self.monitors.first()) {
            builder = builder
                .with_position(monitor.bounds.min)
                .with_inner_size(monitor.bounds.size());
        }
        let ModeNotice {
            modes,
            locale,
            opened,
            presented,
        } = notice;
        let (modes, locale, opened) = (*modes, locale.clone(), opened.clone());
        ctx.show_viewport_deferred(Self::mode_viewport(), builder, move |ctx, _| {
            use theme::mode_osd as motion;
            let elapsed = opened.get_or_init(Instant::now).elapsed().as_secs_f32();
            if elapsed >= motion::LIFETIME {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                ctx.request_repaint_of(egui::ViewportId::ROOT);
                return;
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    draw_mode_osd(ui, &locale, modes, elapsed);
                });
            let delay =
                if (motion::SHOW_TIME + motion::POP_TIME..motion::FADE_AT).contains(&elapsed) {
                    Duration::from_secs_f32(motion::FADE_AT - elapsed)
                } else {
                    Duration::from_millis(16)
                };
            ctx.request_repaint_after(delay);
        });
        if !*presented {
            *presented = true;
            ctx.send_viewport_cmd_to(Self::mode_viewport(), egui::ViewportCommand::Visible(true));
        }
    }
}
impl eframe::App for Host {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        theme::CLEAR.to_normalized_gamma_f32()
    }
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Some(action) => self.run_shortcut_action(action),
                None => self.close_shortcut(),
            }
        }
        while let Ok(message) = self.messages.try_recv() {
            match message {
                HostMessage::Shortcut(request, config) => self.receive(request, *config),
                HostMessage::Mode(modes, locale) => {
                    self.mode_notice = Some(ModeNotice {
                        modes,
                        locale,
                        opened: Arc::default(),
                        presented: false,
                    });
                }
            }
            // Hidden roots run logic only; briefly wake the transparent anchor to create the child.
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }
        if self.eof.try_recv().is_ok() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if let Some((start, _)) = &self.closing {
            if start.elapsed() >= Duration::from_millis(120) {
                let action = self.closing.take().and_then(|(_, a)| a);
                self.shown = false;
                if let Some(action) = action {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(150));
                        if let Err(error) = crate::daemon::Client::default()
                            .ask(&tablet_config::Request::Run { action })
                        {
                            eprintln!("tablet-center: shortcut action failed: {error}");
                        }
                    });
                }
            }
        }
        if let Some(notice) = &self.mode_notice {
            if let Some(opened) = notice.opened.get() {
                let remaining = Duration::from_secs_f32(theme::mode_osd::LIFETIME)
                    .saturating_sub(opened.elapsed());
                if remaining.is_zero() {
                    self.mode_notice = None;
                    ctx.send_viewport_cmd_to(
                        Self::mode_viewport(),
                        egui::ViewportCommand::Visible(false),
                    );
                } else {
                    ctx.request_repaint_after(remaining);
                }
            } else {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }
        if self.shown {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.show_mode_notice(ui.ctx());
        let Some(request) = self.pending.clone() else {
            return;
        };
        if !self.shown {
            return;
        }
        let monitor = request
            .at
            .and_then(|p| self.monitors.iter().find(|m| m.bounds.contains(p)));
        let mut builder = egui::ViewportBuilder::default()
            .with_title("Tablet Center")
            .with_app_id("tablet-center-shortcut")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_active(true)
            .with_maximized(true);
        if let Some(m) = monitor {
            builder = builder
                .with_position(m.bounds.min)
                .with_inner_size(m.bounds.size());
        }
        let ctx = ui.ctx().clone();
        let config = self.config.clone();
        let monitors = self.monitors.clone();
        let locale = self.locale.clone();
        let opened = self.opened;
        let closing = self.closing.as_ref().map(|(start, _)| *start);
        let focused_before = self.focused.clone();
        let events = self.event_tx.clone();
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of("shortcut-overlay"),
            builder,
            move |ctx, _| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        let m = shortcuts::menu(&config.shortcuts, request.name);
                        let size = shortcuts::menu_size(m, request.name);
                        // The first pointer position seen, kept for this opening
                        // so the menu does not follow the cursor.
                        let anchor = egui::Id::new(("overlay-anchor", opened));
                        let cursor =
                            ui.ctx()
                                .data_mut(|d| d.get_temp::<Pos2>(anchor))
                                .or_else(|| {
                                    let p = ui.ctx().input(|i| i.pointer.latest_pos())?;
                                    ui.ctx().data_mut(|d| d.insert_temp(anchor, p));
                                    Some(p)
                                });
                        let radial = shortcuts::is_radial(request.name);
                        let label_margin = if radial {
                            shortcuts::radial_label_margin(ui, m, size.x)
                        } else {
                            0.
                        };
                        let origin = place(
                            &request,
                            m.at_cursor,
                            cursor,
                            &monitors,
                            ui.ctx().content_rect().size(),
                            size + vec2(0., 2. * label_margin),
                        ) + vec2(0., label_margin);
                        let elapsed = opened.elapsed().as_secs_f32();
                        use crate::theme::interaction as motion;
                        let (opacity, scale_progress) = if let Some(start) = &closing {
                            let remaining = 1.
                                - crate::widgets::css_ease(
                                    start.elapsed().as_secs_f32() / motion::OVERLAY_HIDE_TIME,
                                    motion::EASE_IN,
                                );
                            (remaining, remaining)
                        } else if radial {
                            // Viewport creation may take longer than the animation.
                            let id = egui::Id::new("radial-first-frame");
                            let first_frame = ui.ctx().data_mut(|data| {
                                if let Some((request, first_frame)) =
                                    data.get_temp::<(Instant, Instant)>(id)
                                {
                                    if request == opened {
                                        return first_frame;
                                    }
                                }
                                let first_frame = Instant::now();
                                data.insert_temp(id, (opened, first_frame));
                                first_frame
                            });
                            let progress = (first_frame.elapsed().as_secs_f32()
                                / motion::OVERLAY_SHOW_TIME)
                                .clamp(0., 1.);
                            let eased = egui::emath::easing::quadratic_out(progress);
                            (eased, eased)
                        } else {
                            let time = elapsed - motion::OVERLAY_MAP_DELAY;
                            (
                                crate::widgets::css_ease(
                                    time / motion::OVERLAY_SHOW_TIME,
                                    motion::EASE_OUT,
                                ),
                                crate::widgets::css_ease(
                                    time / motion::OVERLAY_SCALE_TIME,
                                    motion::OVERLAY_EASE,
                                ),
                            )
                        };
                        let center = origin + size / 2.;
                        let scale = motion::OVERLAY_SCALE_START
                            + scale_progress * (1. - motion::OVERLAY_SCALE_START);
                        let layer =
                            egui::LayerId::new(egui::Order::Middle, egui::Id::new("overlay-menu"));
                        let mut content = ui.new_child(egui::UiBuilder::new().layer_id(layer));
                        content.ctx().set_transform_layer(
                            layer,
                            egui::emath::TSTransform::from_translation(center.to_vec2())
                                * egui::emath::TSTransform::from_scaling(scale)
                                * egui::emath::TSTransform::from_translation(-center.to_vec2()),
                        );
                        content.set_opacity(opacity);
                        let ui = &mut content;
                        let response = shortcuts::draw_menu(
                            ui,
                            &locale,
                            &config.shortcuts,
                            request.name,
                            origin,
                            None,
                            false,
                        );
                        if let Some(i) = response.item {
                            if let Some(item) =
                                shortcuts::items(&config.shortcuts, request.name).get(i)
                            {
                                let _ = events.send(Some(item.action.clone()));
                                ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                            }
                        }
                        let transform = ui.ctx().layer_transform_from_global(layer);
                        let (focused, escape, outside, closed) = ui.ctx().input(|i| {
                            (
                                i.viewport().focused.unwrap_or(false),
                                i.key_pressed(egui::Key::Escape),
                                i.pointer.any_click()
                                    && i.pointer.interact_pos().is_some_and(|p| {
                                        let local = transform.map_or(p, |transform| transform * p);
                                        !Rect::from_min_size(origin, size).contains(local)
                                    }),
                                i.viewport().close_requested(),
                            )
                        });
                        if response.close
                            || escape
                            || outside
                            || closed
                            || focused_before.load(std::sync::atomic::Ordering::Relaxed) && !focused
                        {
                            let _ = events.send(None);
                            ui.ctx().request_repaint_of(egui::ViewportId::ROOT);
                        }
                        if focused {
                            focused_before.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        ui.ctx().request_repaint_after(Duration::from_millis(16));
                    });
            },
        );
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stdin_mode_osd_accepts_transitions_and_rejects_invalid_requests() {
        use tablet_config::PenMode;
        for (name, mode) in [("mouse", PenMode::Mouse), ("relative", PenMode::Relative)] {
            let line = format!(r#"{{"name":"mode_osd","from":"pen","to":"{name}"}}"#);
            assert_eq!(
                ModeOsd::parse(&line),
                Some(ModeOsd {
                    from: PenMode::Pen,
                    to: mode
                })
            );
            let line = format!(r#"{{"name":"mode_osd","from":"{name}","to":"pen"}}"#);
            assert_eq!(
                ModeOsd::parse(&line),
                Some(ModeOsd {
                    from: mode,
                    to: PenMode::Pen
                })
            );
        }
        for line in [
            r#"{"name":"mode_osd","from":"unknown","to":"mouse"}"#,
            r#"{"name":"mode_osd","from":"pen"}"#,
            r#"{"name":"mode_osd","from":"pen","to":"pen"}"#,
            r#"{"name":"keypad","from":"pen","to":"mouse"}"#,
            "invalid JSON",
        ] {
            assert_eq!(ModeOsd::parse(line), None, "{line}");
        }
        assert!(ShowRequest::parse(r#"{"name":"mode_osd","from":"pen","to":"mouse"}"#).is_none());
    }
    #[test]
    fn stdin_protocol_and_monitor_placement_match_shortcut_js() {
        assert!(ShowRequest::parse(r#"{"name":"unknown"}"#).is_none());
        let request =
            ShowRequest::parse(r#"{"name":"keypad","at":[2700,110],"centered":false}"#).unwrap();
        let monitors = [Monitor {
            bounds: Rect::from_min_size(pos2(2560., 97.), vec2(1920., 1200.)),
        }];
        let menu = vec2(364., 484.);
        let position = place(&request, true, None, &monitors, vec2(1920., 1170.), menu);
        assert_eq!(position, pos2(8., 8.));
        let centered = place(&request, false, None, &monitors, vec2(1920., 1170.), menu);
        assert_eq!(centered, pos2(778., 343.));
        let mut key_request = request.clone();
        key_request.centered = true;
        assert_eq!(
            place(
                &key_request,
                true,
                None,
                &monitors,
                vec2(1920., 1170.),
                menu
            ),
            centered
        );
        // Mouse mode: no point from tabletd, so the pointer in the window.
        let mut mouse = request.clone();
        mouse.at = None;
        assert_eq!(
            place(
                &mouse,
                true,
                Some(pos2(600., 500.)),
                &monitors,
                vec2(1920., 1170.),
                menu
            ),
            pos2(418., 258.)
        );
        assert_eq!(
            place(&mouse, true, None, &monitors, vec2(1920., 1170.), menu),
            centered
        );
    }
    #[test]
    fn same_request_toggles_and_disabled_action_does_not_close() {
        let (_tx, messages) = mpsc::channel();
        let (_end, eof) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let mut host = Host {
            config: Config::default(),
            pending: None,
            shown: false,
            monitors: vec![],
            primary: None,
            messages,
            mode_notice: None,
            eof,
            opened: Instant::now(),
            closing: None,
            focused: Default::default(),
            locale: Locale::new("en").into(),
            events,
            event_tx,
        };
        let request = ShowRequest::parse(r#"{"name":"express_menu"}"#).unwrap();
        host.receive(request.clone(), Config::default());
        assert!(host.shown);
        assert_eq!(host.pending_shortcut(), Some(&request));
        host.run_shortcut_action(KeyAction::Disabled);
        assert!(host.closing.is_none());
        host.receive(request, Config::default());
        assert!(host.closing.is_some());
        host.receive(
            ShowRequest::parse(r#"{"name":"keypad"}"#).unwrap(),
            Config::default(),
        );
        assert!(host.closing.is_none());
        assert!(host.shown);
        host.run_shortcut_action(KeyAction::LeftClick);
        assert!(matches!(
            host.closing,
            Some((_, Some(KeyAction::LeftClick)))
        ));
    }
}
