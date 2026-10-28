//! Tablet Center native UI and deterministic screenshot state.
pub mod assets;
pub mod classic;
pub mod daemon;
pub mod frame;
pub mod i18n;
pub mod icons;
pub mod overlay;
pub mod shared;
pub mod system;
pub mod tablet_input;
pub mod theme;
pub mod widgets;
use tablet_config::{Config, DeviceInfo, Display, PenState, Response};
pub struct App {
    pub tablet: Option<tablet_input::TabletInput>,
    pub config: Config,
    pub status: Response,
    pub locale: i18n::Locale,
    pub tab: usize,
    pub page: usize,
    pub pen: classic::pen::State,
    pub devices: classic::devices::State,
    pub shortcuts: classic::shortcuts::State,
    pub error: Option<String>,
    pub settings: classic::settings::State,
    worker: Option<daemon::Worker>,
    installed: bool,
    fitted: bool,
    shell_language: String,
}
impl App {
    pub fn simulated() -> Self {
        Self {
            tablet: None,
            config: Config::default(),
            status: Response {
                ok: true,
                device: Some(DeviceInfo {
                    name: "Wacom Intuos S Pen".into(),
                    vendor: 0x056a,
                    product: 0x0374,
                    ..Default::default()
                }),
                pen: Some(PenState::default()),
                displays: vec![
                    Display {
                        id: "DEL|DELL S2721QS|HH2HM43".into(),
                        name: "Dell Inc. 27\"".into(),
                        width: 2560.,
                        height: 1440.,
                        primary: true,
                        ..Default::default()
                    },
                    Display {
                        id: "DEL|DELL U2412M|YMYH12BL49CL".into(),
                        name: "Dell Inc. 24\"".into(),
                        x: 2560.,
                        y: 97.,
                        width: 1920.,
                        height: 1200.,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            locale: i18n::Locale::new("en"),
            tab: 0,
            page: 0,
            pen: Default::default(),
            devices: Default::default(),
            shortcuts: Default::default(),
            error: None,
            settings: Default::default(),
            worker: None,
            installed: false,
            fitted: false,
            shell_language: String::new(),
        }
    }
    pub fn native(ctx: &egui::Context) -> Self {
        let mut app = Self::simulated();
        app.status = Response::default();
        app.devices.native = true;
        app.settings.native = true;
        app.shortcuts.actions.native = true;
        app.page = 6;
        app.config = tablet_config::load();
        app.worker = Some(daemon::Worker::start(
            ctx.clone(),
            tablet_config::config_path(),
        ));
        app
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        self.paint(ui);
        frame::round_window(ui.ctx());
    }
    /// Shrinks the first window to the monitor once its size is known.
    fn fit_to_monitor(&mut self, ctx: &egui::Context) {
        if self.fitted {
            return;
        }
        let Some((monitor, inner)) =
            ctx.input(|i| i.viewport().monitor_size.zip(i.viewport().inner_rect))
        else {
            return;
        };
        self.fitted = true;
        let limit = monitor * theme::WINDOW_SCREEN_SHARE;
        let size = inner.size().min(limit);
        if size != inner.size() {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        }
    }
    /// The current classic tab's page, drawn on the shifted page layer.
    fn pages(&mut self, ui: &mut egui::Ui, old_page: usize) {
        let view = crate::widgets::view(ui);
        if self.tab == 0
            && self.devices.selected.is_some()
            && !self.devices.capture
            && self.devices.action_dialog.is_none()
        {
            let background = ui.interact(
                crate::widgets::rect(
                    theme::devices_layout::PX_431,
                    theme::HEADER,
                    view.right() - theme::devices_layout::PX_432,
                    view.bottom() - theme::HEADER,
                ),
                ui.id().with("dismiss-device-actions"),
                egui::Sense::click(),
            );
            if background.clicked() {
                self.devices.selected = None;
            }
        }
        if self.tab == 0 {
            if self.page != old_page {
                self.devices.selected = None;
                // A panel from the last page must not slide out on this one.
                self.devices.panel_last = None;
            }
            if self.status.device.is_none() {
                classic::devices::welcome(ui, &self.locale, self.error.is_some());
            } else if self.page == 0 {
                let live = self.status.pen.clone().unwrap_or_default();
                // The pen page shows its actions panel on the right.
                self.devices.panel_right = true;
                classic::devices::strip(
                    ui,
                    &self.locale,
                    &mut self.config,
                    &mut self.devices,
                    self.status.focused_app.as_deref(),
                );
                let pen = if let Some(i) = self.devices.app {
                    &mut self.config.apps[i].pen
                } else {
                    &mut self.config.pen
                };
                self.pen.selected = self
                    .devices
                    .selected
                    .filter(|i| classic::devices::is_pen(*i))
                    .map(|i| i - classic::devices::PEN_UPPER);
                classic::pen::show(ui, &self.locale, pen, &mut self.pen, &live);
                if let Some(index) = self.pen.action.take() {
                    self.devices.selected =
                        if self.devices.selected == Some(index + classic::devices::PEN_UPPER) {
                            None
                        } else {
                            Some(index + classic::devices::PEN_UPPER)
                        };
                }
                classic::devices::overlays(
                    ui,
                    &self.locale,
                    &mut self.config,
                    &self.status,
                    &mut self.devices,
                );
                self.devices.panel_right = false;
            } else {
                classic::devices::show(
                    ui,
                    &self.locale,
                    &mut self.config,
                    &self.status,
                    self.page,
                    &mut self.devices,
                );
                if let Some(page) = ui
                    .ctx()
                    .data_mut(|d| d.remove_temp::<usize>(egui::Id::new("navigate")))
                {
                    self.page = page;
                }
            }
        } else if self.tab == 1 {
            classic::shortcuts::show(ui, &self.locale, &mut self.config, &mut self.shortcuts);
        } else {
            classic::settings::show(
                ui,
                &mut self.locale,
                &mut self.config,
                &self.status,
                &mut self.settings,
            );
        }
    }
    fn paint(&mut self, ui: &mut egui::Ui) {
        if ui.input(|i| i.pointer.any_pressed()) {
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("css-keyboard-focus"), false));
        } else if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            ui.ctx()
                .data_mut(|d| d.insert_temp(egui::Id::new("css-keyboard-focus"), true));
        }
        self.fit_to_monitor(ui.ctx());
        if !self.installed {
            assets::install(ui.ctx());
            self.installed = true;
            ui.ctx().request_repaint();
            return;
        }
        if let Some(worker) = &self.worker {
            while let Ok(update) = worker.updates.try_recv() {
                match update {
                    Ok(status) => {
                        self.status = status;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                        self.status = Response::default();
                    }
                }
            }
            while let Ok(error) = worker.errors.try_recv() {
                self.settings.notify(self.locale.t(&error));
                self.error = Some(error);
            }
        }
        self.settings.initialize(ui.ctx());
        let previous = self.config.clone();
        self.settings.poll(&mut self.config, &mut self.locale);
        if self
            .devices
            .app
            .is_some_and(|index| index >= self.config.apps.len())
        {
            self.devices.app = None;
            self.devices.selected = None;
        }
        let old_page = self.page;
        let r = ui.ctx().content_rect();
        {
            ui.ctx().style_mut_of(ui.ctx().theme(), |style| {
                style.visuals.text_options.color_transfer_function =
                    egui::epaint::FontColorTransferFunction::Gamma(theme::TEXT_GAMMA)
            });
            frame::show(ui, r);
            let connected = self.status.device.is_some();
            classic::navigation(ui, r.height(), &self.locale, &mut self.tab);
            if self.tab == 0 && connected {
                classic::sidebar(
                    ui,
                    &self.locale,
                    &mut self.page,
                    self.status.device.as_ref(),
                    self.status.touch
                        || self
                            .status
                            .device
                            .as_ref()
                            .is_some_and(|d| d.product == 0x03ce),
                );
            } else if self.tab == 1 {
                classic::shortcuts::sidebar(ui, &self.locale, &mut self.shortcuts);
            }
            // Pages keep their original header-based layout on a layer shifted
            // under the title bar; full-width pages also move right of the sidebar.
            let shift = if self.tab == 2 || !connected {
                theme::SIDEBAR
            } else {
                0.
            };
            let layer = egui::LayerId::new(ui.layer_id().order, egui::Id::new("page"));
            ui.ctx().set_transform_layer(
                layer,
                egui::emath::TSTransform::from_translation(egui::vec2(shift, -theme::PAGE_SHIFT)),
            );
            ui.ctx().set_sublayer(ui.layer_id(), layer);
            let mut page = ui.new_child(egui::UiBuilder::new().layer_id(layer));
            crate::widgets::shifted_access(&page);
            let view = crate::widgets::view(&page);
            page.set_clip_rect(egui::Rect::from_min_max(
                egui::pos2(view.left().max(theme::SIDEBAR - shift), theme::HEADER),
                view.max,
            ));
            self.pages(&mut page, old_page);
        }
        if self.config != previous || self.shell_language != self.settings.language {
            if let Some(worker) = &self.worker {
                let menus = system::shell_menus(&self.config, &self.locale);
                let _ = worker
                    .save
                    .send((self.config.clone(), menus, system::config_epoch()));
            }
        }
        self.shell_language = self.settings.language.clone();
        {
            let mut top = ui.new_child(egui::UiBuilder::new().layer_id(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("toast"),
            )));
            classic::settings::toast(&mut top, &mut self.settings);
            frame::finish(ui, r);
        }
    }
}
impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        theme::CLEAR.to_normalized_gamma_f32()
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if let Some(tablet) = &mut self.tablet {
            tablet.inject(ctx, raw);
        }
        frame::release_hook(ctx, raw);
        if let Some(worker) = &self.worker {
            if let Some(minimized) = raw
                .viewports
                .get(&egui::ViewportId::ROOT)
                .and_then(|v| v.minimized)
            {
                worker
                    .visible
                    .store(!minimized, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Some(tablet) = &mut self.tablet {
            tablet.run_grab(ui.ctx(), frame);
        }

        self.show(ui);
    }
}
