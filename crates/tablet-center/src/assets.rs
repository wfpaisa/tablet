//! Embedded fonts and original device artwork.
use egui::{FontData, FontDefinitions, FontFamily};
pub fn install(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in [
        (
            "roboto",
            include_bytes!("../../../assets/fonts/Roboto-Regular.ttf").as_slice(),
        ),
        (
            "medium",
            include_bytes!("../../../assets/fonts/Roboto-Medium.ttf").as_slice(),
        ),
        (
            "bold",
            include_bytes!("../../../assets/fonts/Roboto-Bold.ttf").as_slice(),
        ),
    ] {
        fonts.font_data.insert(name.into(), {
            let mut data = FontData::from_static(bytes);
            data.tweak.hinting = Some(false);
            data.into()
        });
    }
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "roboto".into());
    fonts.font_data.insert(
        "semibold".into(),
        FontData::from_static(include_bytes!("../../../assets/fonts/Roboto-Semibold.ttf")).into(),
    );
    for name in ["medium", "bold", "semibold"] {
        fonts.families.insert(
            FontFamily::Name(name.into()),
            vec![name.into(), "roboto".into()],
        );
    }
    for path in [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert("cjk".into(), {
                let mut data = FontData::from_owned(bytes);
                data.tweak.y_offset = crate::theme::settings_layout::CJK_OFFSET;
                data.into()
            });
            for family in fonts.families.values_mut() {
                family.push("cjk".into());
            }
            break;
        }
    }
    ctx.style_mut_of(egui::Theme::Dark, |s| {
        s.visuals.text_options.color_transfer_function =
            egui::epaint::FontColorTransferFunction::Gamma(crate::theme::TEXT_GAMMA)
    });
    ctx.set_fonts(fonts);
}
pub const PEN: egui::ImageSource<'static> =
    egui::include_image!("../../../assets/devices/intuos-pen.png");
pub const TABLET: egui::ImageSource<'static> =
    egui::include_image!("../../../assets/devices/intuos-s.png");
pub const LOGO: egui::ImageSource<'static> = egui::include_image!("../../../assets/app-icon.png");
/// Classic navigation glyph: Devices, Shortcuts, Settings, then the sidebar pages.
pub fn icon(index: usize) -> egui::ImageSource<'static> {
    const NAMES: [&str; 8] = [
        "tablet",
        "layout-grid",
        "settings",
        "pencil",
        "keyboard",
        "monitor",
        "refresh-cw",
        "hand",
    ];
    crate::icons::source(NAMES[index])
}

/// Prefilter large source images before minification, as the browser does.
pub fn artwork(ui: &egui::Ui, name: &str, bytes: &[u8], rect: egui::Rect) {
    artwork_rotated(ui, name, bytes, rect, 0.);
}
pub fn artwork_rotated(ui: &egui::Ui, name: &str, bytes: &[u8], rect: egui::Rect, angle: f32) {
    let size = [rect.width().ceil() as usize, rect.height().ceil() as usize];
    let id = egui::Id::new((name, size));
    let cached = ui.ctx().data(|d| d.get_temp::<egui::TextureHandle>(id));
    let texture = cached.or_else(|| {
        let image = image::load_from_memory(bytes)
            .ok()?
            .resize_exact(
                size[0] as u32,
                size[1] as u32,
                image::imageops::FilterType::Triangle,
            )
            .into_rgba8();
        let texture = ui.ctx().load_texture(
            name,
            egui::ColorImage::from_rgba_unmultiplied(size, &image),
            egui::TextureOptions::LINEAR,
        );
        ui.ctx().data_mut(|d| d.insert_temp(id, texture.clone()));
        Some(texture)
    });
    if let Some(texture) = texture {
        egui::Image::new(&texture)
            .rotate(angle, egui::vec2(0.5, 0.5))
            .paint_at(ui, rect);
    }
}
/// Soft light behind dark device art in dark mode, fading out from its center.
pub fn glow(ui: &egui::Ui, rect: egui::Rect) {
    if !ui.visuals().dark_mode {
        return;
    }
    use crate::theme as t;
    let center = rect.center();
    // Small thumbnails get a tighter halo than full-size pictures.
    let spread = (rect.size().max_elem() * 0.15).clamp(12., t::GLOW_SPREAD);
    let radius = rect.size() / 2. + egui::vec2(spread, spread);
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, t::DEVICE_GLOW);
    let steps = 64u32;
    for i in 0..steps {
        let angle = i as f32 / steps as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(
            center + egui::vec2(angle.cos() * radius.x, angle.sin() * radius.y),
            egui::Color32::TRANSPARENT,
        );
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % steps);
    }
    ui.painter().add(mesh);
}
