//! Native wgpu startup with a custom undecorated window.
use eframe::{egui_wgpu, wgpu};
use std::sync::Arc;

/// Small wgpu memory blocks: the default reserves ~200 MiB of VRAM up front.
fn wgpu_options() -> egui_wgpu::WgpuConfiguration {
    let mut options = egui_wgpu::WgpuConfiguration::default();
    if let egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_setup {
        let base = Arc::clone(&setup.device_descriptor);
        setup.device_descriptor = Arc::new(move |adapter| wgpu::DeviceDescriptor {
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..base(adapter)
        });
    }
    options
}

fn main() -> eframe::Result {
    let host = std::env::args().any(|arg| arg == "--shortcut-host");
    let simulated = std::env::args().any(|arg| arg == "--simulate");
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_visible(!host)
            .with_title("Tablet Center")
            .with_app_id("tablet-center")
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size(tablet_center::theme::WINDOW_SIZE)
            .with_min_inner_size(tablet_center::theme::WINDOW_MIN),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_options(),
        ..Default::default()
    };
    if host {
        options.viewport = egui::ViewportBuilder::default()
            .with_title("Tablet Center shortcut host")
            .with_app_id("tablet-center-shortcut-host")
            .with_visible(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_taskbar(false)
            .with_mouse_passthrough(true)
            .with_active(false)
            .with_inner_size([1., 1.])
            .with_min_inner_size([1., 1.]);
    }
    eframe::run_native(
        "Tablet Center",
        options,
        Box::new(move |cc| {
            if host {
                return Ok(Box::new(tablet_center::overlay::Host::new(cc)));
            }
            let mut app = if simulated {
                tablet_center::App::simulated()
            } else {
                tablet_center::App::native(&cc.egui_ctx)
            };
            app.tablet = tablet_center::tablet_input::TabletInput::new(cc);
            Ok(Box::new(app))
        }),
    )
}
