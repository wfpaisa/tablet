//! Config path helpers, device names and tablet artwork shared by the interfaces.
use crate::theme::device_art as t;
use egui::{Rect, Ui};
use serde_json::Value;

pub fn get<'a>(v: &'a Value, path: &str) -> &'a Value {
    let mut v = v;
    for k in path.split('.') {
        v = if let Ok(i) = k.parse::<usize>() {
            &v[i]
        } else {
            &v[k]
        };
    }
    v
}
pub fn set(v: &mut Value, path: &str, value: Value) {
    let keys: Vec<_> = path.split('.').collect();
    let mut v = v;
    let default = if path.contains(".items.") {
        serde_json::json!({"label":"","action":{"type":"disabled"}})
    } else {
        serde_json::json!({"type":"disabled"})
    };
    for (k, key) in keys.iter().enumerate() {
        if let Ok(index) = key.parse::<usize>() {
            if let Some(array) = v.as_array_mut() {
                if array.len() <= index {
                    array.resize(index + 1, default.clone());
                }
            }
            if k == keys.len() - 1 {
                v[index] = value;
                return;
            }
            v = &mut v[index];
        } else {
            if k == keys.len() - 1 {
                v[*key] = value;
                return;
            }
            v = &mut v[*key];
        }
    }
}
/// Flatten an action preset into the shape stored at `path` (pen is a string or object).
pub fn action_value(path: &str, mut value: Value) -> Value {
    if path.starts_with("pen.") {
        let id = value["type"].as_str().unwrap_or("disabled").to_owned();
        if let Some(obj) = value.as_object_mut() {
            obj.remove("type");
        }
        value = if value.as_object().is_none_or(|v| v.is_empty()) {
            Value::from(id)
        } else {
            serde_json::json!({id:value})
        };
    }
    value
}
pub fn apply_action(config: &mut tablet_config::Config, path: &str, value: Value) {
    let value = action_value(path, value);
    let mut data = serde_json::to_value(&*config).unwrap_or_default();
    set(&mut data, path, value);
    if let Ok(updated) = serde_json::from_value(data) {
        *config = updated;
    }
}

/// Marketing name and model number for known product ids.
pub fn model_name(product: u16) -> Option<(&'static str, &'static str)> {
    Some(match product {
        0x0374 => ("Intuos S", "CTL-4100"),
        0x0375 => ("Intuos M", "CTL-6100"),
        0x0376 | 0x0377 | 0x03c6 => ("Intuos S", "CTL-4100WL"),
        0x0378 | 0x0379 | 0x03c8 => ("Intuos M", "CTL-6100WL"),
        0x0100 | 0x0101 | 0x0104 => ("Wacom One S", "CTC4110WL"),
        0x0102 | 0x0103 | 0x0105 => ("Wacom One M", "CTC6110WL"),
        0x03ce => ("Wacom One 12", "DTC121"),
        0x03cb => ("Wacom One 13 touch", "DTH134"),
        0x0392 | 0x0393 | 0x03dd => ("Intuos Pro S", "PTH-460"),
        0x0357 | 0x0360 => ("Intuos Pro M", "PTH-660"),
        0x0358 | 0x0361 => ("Intuos Pro L", "PTH-860"),
        0x0314 => ("Intuos Pro S", "PTH-451"),
        0x0315 => ("Intuos Pro M", "PTH-651"),
        0x0317 => ("Intuos Pro L", "PTH-851"),
        _ => return None,
    })
}
pub fn device_name(status: &tablet_config::Response) -> String {
    let Some(device) = &status.device else {
        return "No tablet connected".into();
    };
    let (name, model) = model_name(device.product).unwrap_or((device.name.as_str(), ""));
    format!("{name} {model}").trim().into()
}

/// Product artwork shared by every rendering site.
pub struct DeviceArt {
    pub name: &'static str,
    pub bytes: &'static [u8],
    pub ratio: f32,
    pub pen: (&'static str, &'static [u8]),
}
pub fn device_art(product: u16) -> DeviceArt {
    macro_rules! art {
        ($name:literal, $ratio:expr, $pen:literal) => {
            DeviceArt {
                name: $name,
                bytes: include_bytes!(concat!("../../../assets/devices/", $name, ".png")),
                ratio: $ratio,
                pen: (
                    $pen,
                    include_bytes!(concat!("../../../assets/devices/", $pen, ".png")),
                ),
            }
        };
    }
    match product {
        0x0376 | 0x0377 | 0x03c6 => art!("intuos-s-wl-wacom", t::INTUOS_S_WL_RATIO, "intuos-pen"),
        0x0375 => art!("intuos-m-wacom", t::INTUOS_M_RATIO, "intuos-pen"),
        0x0378 | 0x0379 | 0x03c8 => art!("intuos-m-wl-wacom", t::INTUOS_M_WL_RATIO, "intuos-pen"),
        0x0100..=0x0105 => art!("wacom-one-wacom", t::ONE_RATIO, "wacom-one-pen-wacom"),
        0x03ce => art!("wacom-one-12-wacom", t::ONE_12_RATIO, "wacom-one-pen-wacom"),
        0x03cb => art!(
            "wacom-one-13-touch-wacom",
            t::ONE_13_RATIO,
            "wacom-one-pen-wacom"
        ),
        0x0392 | 0x0393 | 0x03dd => art!("intuos-pro-s-wacom", t::PRO_S_RATIO, "pro-pen-2-wacom"),
        0x0357 | 0x0360 => art!("intuos-pro-m-wacom", t::PRO_M_RATIO, "pro-pen-2-wacom"),
        0x0358 | 0x0361 => art!("intuos-pro-l-wacom", t::PRO_L_RATIO, "pro-pen-2-wacom"),
        0x0314 | 0x0315 | 0x0317 => art!(
            "intuos-pro-2013-wacom",
            t::PRO_2013_TABLET_RATIO,
            "grip-pen-wacom"
        ),
        _ => art!("intuos-s", t::INTUOS_S_RATIO, "intuos-pen"),
    }
}
/// Callers add their site to keep texture sizes independent.
pub fn pen_art(product: u16) -> (&'static str, &'static [u8]) {
    device_art(product).pen
}

/// Match the halo to the opaque pen rather than its padded canvas.
pub fn pen_glow(ui: &Ui, product: u16, area: Rect) {
    use crate::theme::pen_layout as p;
    let bounds = match device_art(product).pen.0 {
        "pro-pen-2-wacom" => Some(p::PRO_PEN_BOUNDS),
        "wacom-one-pen-wacom" => Some(p::ONE_PEN_BOUNDS),
        _ => None,
    };
    let area = bounds.map_or(area, |[x, y, width, height]| {
        Rect::from_min_size(
            area.min + egui::vec2(x * area.width(), y * area.height()),
            egui::vec2(width * area.width(), height * area.height()),
        )
    });
    crate::assets::glow(ui, area);
}

/// Upper and lower rocker halves, then the nib, in artwork coordinates.
pub fn pen_spots(product: u16) -> [(f32, f32); 3] {
    use crate::theme::pen_layout as p;
    if tablet_config::intuos_pro_2013(product) {
        [
            (p::GRIP_PEN_BUTTON_X_RATIO, p::GRIP_PEN_UPPER_Y_RATIO),
            (p::GRIP_PEN_BUTTON_X_RATIO, p::GRIP_PEN_LOWER_Y_RATIO),
            (p::GRIP_PEN_TIP_X_RATIO, p::GRIP_PEN_TIP_Y_RATIO),
        ]
    } else if device_art(product).pen.0 == "pro-pen-2-wacom" {
        p::PRO_PEN_SPOTS
    } else if device_art(product).pen.0 == "wacom-one-pen-wacom" {
        p::ONE_PEN_SPOTS
    } else {
        [
            (p::BUTTON_X_RATIO, p::UPPER_Y_RATIO),
            (p::BUTTON_X_RATIO, p::LOWER_Y_RATIO),
            (p::TIP_X_RATIO, p::TIP_Y_RATIO),
        ]
    }
}

pub fn intuos_art(product: u16) -> bool {
    matches!(
        product,
        0x0374 | 0x0375 | 0x0376 | 0x0377 | 0x03c6 | 0x0378 | 0x0379 | 0x03c8
    )
}
/// Height / width of the ExpressKeys artwork, `None` for the generic drawing.
pub fn art_ratio(product: u16) -> Option<f32> {
    match product {
        0x0314 => Some(t::PRO_2013_S_RATIO),
        0x0315 | 0x0317 => Some(t::PRO_2013_ML_RATIO),
        0x0392 | 0x0393 | 0x03dd => Some(t::PRO_S_KEYS_RATIO),
        0x0357 | 0x0360 => Some(t::PRO_M_KEYS_RATIO),
        0x0358 | 0x0361 => Some(t::PRO_L_KEYS_RATIO),
        0x0375 | 0x0376 | 0x0377 | 0x03c6 | 0x0378 | 0x0379 | 0x03c8 => {
            Some(device_art(product).ratio)
        }
        _ => None,
    }
}
pub fn key_spot(product: u16, index: usize, _count: usize) -> (f32, f32) {
    use crate::theme::settings_layout as d;
    if matches!(product, 0x0375 | 0x0378 | 0x0379 | 0x03c8) {
        let (xs, y) = if product == 0x0375 {
            (t::INTUOS_M_KEY_X, t::INTUOS_M_KEY_Y)
        } else {
            (t::INTUOS_M_WL_KEY_X, t::INTUOS_M_WL_KEY_Y)
        };
        return (xs[index.min(3)], y);
    }
    if intuos_art(product) {
        return (t::INTUOS_KEY_X[index.min(3)], t::INTUOS_KEY_Y);
    }
    if tablet_config::intuos_pro_2013(product) {
        let ys: &[f32] = if product == 0x0314 {
            &t::PRO_2013_S_KEY_Y
        } else {
            &t::PRO_2013_ML_KEY_Y
        };
        return (t::PRO_2013_KEY_X, ys[index.min(ys.len() - 1)]);
    }
    if matches!(
        product,
        0x0392 | 0x0393 | 0x03dd | 0x0357 | 0x0360 | 0x0358 | 0x0361
    ) {
        let (x, ys) = match product {
            0x0392 | 0x0393 | 0x03dd => (t::PRO_S_KEY_X, t::PRO_S_KEY_Y.as_slice()),
            0x0358 | 0x0361 => (t::PRO_L_KEY_X, t::PRO_L_KEY_Y.as_slice()),
            _ => (t::PRO_M_KEY_X, t::PRO_M_KEY_Y.as_slice()),
        };
        (x, ys[index.min(ys.len() - 1)])
    } else {
        (d::PX_0_23 + index as f32 * d::PX_0_18, d::PX_0_13)
    }
}
/// Active surface inside the trimmed whole-tablet image.
pub fn active_surface(product: u16) -> [f32; 4] {
    match product {
        0x0375 | 0x0378 | 0x0379 | 0x03c8 => t::INTUOS_M_ACTIVE,
        0x0100..=0x0105 => t::ONE_ACTIVE,
        0x03ce => t::ONE_12_ACTIVE,
        0x03cb => t::ONE_13_ACTIVE,
        0x0392 | 0x0393 | 0x03dd => t::PRO_S_ACTIVE,
        0x0357 | 0x0360 => t::PRO_M_ACTIVE,
        0x0358 | 0x0361 => t::PRO_L_ACTIVE,
        _ => {
            use crate::theme::devices_layout as d;
            [
                d::TABLET_ACTIVE_X,
                d::TABLET_ACTIVE_Y,
                d::TABLET_ACTIVE_WIDTH,
                d::TABLET_ACTIVE_HEIGHT,
            ]
        }
    }
}
/// Centre mode button of the 2017 touch ring.
pub fn ring_spot(product: u16) -> Option<(f32, f32)> {
    Some(match product {
        0x0392 | 0x0393 | 0x03dd => t::PRO_S_PANEL_RING,
        0x0357 | 0x0360 => t::PRO_M_PANEL_RING,
        0x0358 | 0x0361 => t::PRO_L_PANEL_RING,
        _ => return None,
    })
}
pub fn tablet_art(ui: &Ui, product: u16, name: &str, area: Rect) {
    let panel: Option<(&str, &[u8])> = match product {
        0x0392 | 0x0393 | 0x03dd => Some((
            "pro-s-keys",
            include_bytes!("../../../assets/devices/intuos-pro-s-keys-wacom.png"),
        )),
        0x0357 | 0x0360 => Some((
            "pro-m-keys",
            include_bytes!("../../../assets/devices/intuos-pro-m-keys-wacom.png"),
        )),
        0x0358 | 0x0361 => Some((
            "pro-l-keys",
            include_bytes!("../../../assets/devices/intuos-pro-l-keys-wacom.png"),
        )),
        _ => None,
    };
    if let Some((key, bytes)) = panel {
        crate::assets::glow(ui, area);
        crate::assets::artwork(ui, &format!("{name}-{key}"), bytes, area);
        return;
    }
    if intuos_art(product) {
        let art = device_art(product);
        crate::assets::glow(ui, area);
        let cache = if product == 0x0374 {
            name.to_owned()
        } else {
            format!("{name}-{}", art.name)
        };
        crate::assets::artwork(ui, &cache, art.bytes, area);
        return;
    }
    // Intuos Pro 2013: the ExpressKeys panel only, as in the macOS driver.
    if tablet_config::intuos_pro_2013(product) {
        let (key, bytes): (&str, &[u8]) = if product == 0x0314 {
            (
                "pro-2013-s-keys",
                include_bytes!("../../../assets/devices/intuos-pro-2013-s-keys-wacom.png"),
            )
        } else {
            (
                "pro-2013-ml-keys",
                include_bytes!("../../../assets/devices/intuos-pro-2013-ml-keys-wacom.png"),
            )
        };
        crate::assets::glow(ui, area);
        crate::assets::artwork(ui, &format!("{name}-{key}"), bytes, area);
        return;
    }
    use crate::theme::settings_layout as d;
    let column = matches!(
        product,
        0x0392 | 0x0393 | 0x03dd | 0x0357 | 0x0360 | 0x0358 | 0x0361
    );
    let count = tablet_config::express_key_count(product);
    let mut buttons = String::new();
    for i in 0..count {
        let (x, y) = key_spot(product, i, count);
        buttons += &include_str!("../../../assets/devices/tablet-generic-button.svg")
            .replace("{x}", &(x * d::PX_560 - d::PX_13).to_string())
            .replace("{y}", &(y * d::PX_360 - d::PX_8).to_string());
    }
    let svg = include_str!("../../../assets/devices/tablet-generic.svg")
        .replace("{x}", if column { "110" } else { "55" })
        .replace("{y}", if column { "42" } else { "73" })
        .replace("{width}", if column { "400" } else { "450" })
        .replace(
            "{ring}",
            if column {
                include_str!("../../../assets/devices/tablet-generic-ring.svg")
            } else {
                ""
            },
        )
        .replace("{buttons}", &buttons);
    egui::Image::new(egui::ImageSource::Bytes {
        uri: format!("bytes://tablet-art-{product}.svg").into(),
        bytes: svg.into_bytes().into(),
    })
    .paint_at(ui, area);
}
/// Whole-tablet thumbnail for the device list, when the model has one.
pub fn thumbnail(ui: &Ui, product: u16, name: &str, area: Rect) -> bool {
    // Preserve the default model's established thumbnail geometry.
    if device_art(product).name == "intuos-s" {
        return false;
    }
    let art = device_art(product);
    let width = area.width().min(area.height() / art.ratio);
    let r = Rect::from_center_size(area.center(), egui::vec2(width, width * art.ratio));
    crate::assets::glow(ui, r);
    let key = if tablet_config::intuos_pro_2013(product) {
        format!("{name}-pro-2013-tablet")
    } else {
        format!("{name}-{}", art.name)
    };
    crate::assets::artwork(ui, &key, art.bytes, r);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_artwork_matches_its_geometry() {
        let mut checked = std::collections::HashSet::new();
        for product in 0..=u16::MAX {
            if model_name(product).is_none() {
                continue;
            }
            let art = device_art(product);
            if checked.insert(art.name) {
                let image = image::load_from_memory(art.bytes).unwrap();
                let ratio = image.height() as f32 / image.width() as f32;
                assert!((ratio - art.ratio).abs() < 0.0001, "{}", art.name);
            }
            let pen = image::load_from_memory(art.pen.1).unwrap();
            assert_eq!(pen.width() * 1599, pen.height() * 131);
            let spots = pen_spots(product);
            assert!(spots.windows(2).all(|pair| pair[0].1 < pair[1].1));
            assert!(spots
                .iter()
                .all(|&(x, y)| (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y)));
        }
        assert_eq!(device_art(0xffff).name, "intuos-s");
        assert_eq!(pen_art(0xffff).0, "intuos-pen");
    }
}
