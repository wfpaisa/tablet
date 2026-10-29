//! Embedded fonts and original device artwork.
use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;
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
#[derive(Clone, Default)]
struct ArtworkTexture {
    texture: Option<egui::TextureHandle>,
    made: Option<[usize; 2]>,
    last: Option<[usize; 2]>,
    last_change: f64,
}

/// Rebuild the texture when there is none yet or the size has settled on a new one.
fn needs_texture(made: Option<[usize; 2]>, want: [usize; 2], last_change: f64, now: f64) -> bool {
    made.is_none()
        || (made != Some(want) && now - last_change >= crate::theme::interaction::ART_SETTLE_TIME)
}

/// Each name identifies one paint site, including thumbnails shown beside a page.
pub fn artwork_rotated(ui: &egui::Ui, name: &str, bytes: &[u8], rect: egui::Rect, angle: f32) {
    // Physical pixels, so HiDPI screens get a sharp texture.
    let ppp = ui.ctx().pixels_per_point();
    let size = [
        (rect.width() * ppp).ceil() as usize,
        (rect.height() * ppp).ceil() as usize,
    ];
    let id = egui::Id::new((name, "texture"));
    let mut cached = ui
        .ctx()
        .data(|d| d.get_temp::<ArtworkTexture>(id))
        .unwrap_or_default();
    let now = ui.input(|i| i.time);
    if cached.last != Some(size) {
        cached.last = Some(size);
        cached.last_change = now;
    }
    if needs_texture(cached.made, size, cached.last_change, now) {
        let decoded_id = egui::Id::new((name, "decoded"));
        let decoded = ui
            .ctx()
            .data(|d| d.get_temp::<Arc<image::RgbaImage>>(decoded_id))
            .or_else(|| {
                let decoded = Arc::new(image::load_from_memory(bytes).ok()?.into_rgba8());
                ui.ctx()
                    .data_mut(|d| d.insert_temp(decoded_id, decoded.clone()));
                Some(decoded)
            });
        if let Some(decoded) = decoded {
            let image = image::imageops::resize(
                decoded.as_ref(),
                size[0] as u32,
                size[1] as u32,
                image::imageops::FilterType::Triangle,
            );
            cached.texture = Some(ui.ctx().load_texture(
                name,
                egui::ColorImage::from_rgba_unmultiplied(size, &image),
                egui::TextureOptions::LINEAR,
            ));
            cached.made = Some(size);
        }
    } else if cached.made != Some(size) {
        // Resize on the CPU only after the animation settles.
        let remaining = crate::theme::interaction::ART_SETTLE_TIME - (now - cached.last_change);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(remaining.max(0.)));
    }
    if let Some(texture) = &cached.texture {
        egui::Image::new(texture)
            .rotate(angle, egui::vec2(0.5, 0.5))
            .paint_at(ui, rect);
    }
    ui.ctx().data_mut(|d| d.insert_temp(id, cached));
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

#[cfg(test)]
mod tests {
    use super::needs_texture;

    #[test]
    fn texture_follows_settled_sizes() {
        let initial = [100, 200];
        let middle = [90, 180];
        let final_size = [80, 160];
        let settle = crate::theme::interaction::ART_SETTLE_TIME;
        assert!(needs_texture(None, initial, 0., 0.));
        assert!(!needs_texture(Some(initial), initial, 0., settle));
        // Continuous changes reset the quiet period, including repeated pixel sizes.
        for frame in 1..60 {
            let now = f64::from(frame) / 60.;
            assert!(!needs_texture(Some(initial), middle, now, now));
            assert!(!needs_texture(Some(initial), middle, now, now + 1. / 60.));
        }
        assert!(!needs_texture(
            Some(initial),
            final_size,
            1.,
            1. + settle / 2.
        ));
        assert!(needs_texture(Some(initial), final_size, 0., settle));
        assert!(!needs_texture(Some(final_size), final_size, 1., 2.));
        assert!(!needs_texture(Some(initial), initial, 1., 2.));
    }
}
