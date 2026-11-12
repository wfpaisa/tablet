//! Config path helpers, device names and tablet artwork shared by the interfaces.
use crate::theme::device_art as t;
use egui::{Rect, Ui};
use serde_json::Value;
use tablet_config::{family, Family, Family::*};

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
    tablet_config::model(product).map(|m| (m.name, m.number))
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
    match family(product) {
        Some(IntuosSWireless) => art!("intuos-s-wl-wacom", t::INTUOS_S_WL_RATIO, "intuos-pen"),
        Some(IntuosM) => art!("intuos-m-wacom", t::INTUOS_M_RATIO, "intuos-pen"),
        Some(IntuosMWireless) => art!("intuos-m-wl-wacom", t::INTUOS_M_WL_RATIO, "intuos-pen"),
        Some(OneS | OneM | OneByWacomS) => {
            art!("wacom-one-wacom", t::ONE_RATIO, "wacom-one-pen-wacom")
        }
        Some(One12) => art!("wacom-one-12-wacom", t::ONE_12_RATIO, "wacom-one-pen-wacom"),
        Some(One13Touch) => art!(
            "wacom-one-13-touch-wacom",
            t::ONE_13_RATIO,
            "wacom-one-pen-wacom"
        ),
        Some(ProS) => art!("intuos-pro-s-wacom", t::PRO_S_RATIO, "pro-pen-2-wacom"),
        Some(ProM) => art!("intuos-pro-m-wacom", t::PRO_M_RATIO, "pro-pen-2-wacom"),
        Some(ProL) => art!("intuos-pro-l-wacom", t::PRO_L_RATIO, "pro-pen-2-wacom"),
        Some(Pro2013S | Pro2013M | Pro2013L) => art!(
            "intuos-pro-2013-wacom",
            t::PRO_2013_TABLET_RATIO,
            "grip-pen-wacom"
        ),
        Some(IntuosS) | None => art!("intuos-s", t::INTUOS_S_RATIO, "intuos-pen"),
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
    family(product).is_some_and(Family::intuos)
}
/// Height / width of the ExpressKeys artwork, `None` for the generic drawing.
pub fn art_ratio(product: u16) -> Option<f32> {
    match family(product)? {
        Pro2013S => Some(t::PRO_2013_S_RATIO),
        Pro2013M | Pro2013L => Some(t::PRO_2013_ML_RATIO),
        ProS => Some(t::PRO_S_KEYS_RATIO),
        ProM => Some(t::PRO_M_KEYS_RATIO),
        ProL => Some(t::PRO_L_KEYS_RATIO),
        IntuosM | IntuosSWireless | IntuosMWireless => Some(device_art(product).ratio),
        IntuosS | OneS | OneM | One12 | One13Touch | OneByWacomS => None,
    }
}
pub fn key_spot(product: u16, index: usize, _count: usize) -> (f32, f32) {
    use crate::theme::settings_layout as d;
    let column = |x: f32, ys: &[f32]| (x, ys[index.min(ys.len() - 1)]);
    match family(product) {
        Some(IntuosM) => (t::INTUOS_M_KEY_X[index.min(3)], t::INTUOS_M_KEY_Y),
        Some(IntuosMWireless) => (t::INTUOS_M_WL_KEY_X[index.min(3)], t::INTUOS_M_WL_KEY_Y),
        Some(IntuosS | IntuosSWireless) => (t::INTUOS_KEY_X[index.min(3)], t::INTUOS_KEY_Y),
        Some(Pro2013S) => column(t::PRO_2013_KEY_X, &t::PRO_2013_S_KEY_Y),
        Some(Pro2013M | Pro2013L) => column(t::PRO_2013_KEY_X, &t::PRO_2013_ML_KEY_Y),
        Some(ProS) => column(t::PRO_S_KEY_X, &t::PRO_S_KEY_Y),
        Some(ProM) => column(t::PRO_M_KEY_X, &t::PRO_M_KEY_Y),
        Some(ProL) => column(t::PRO_L_KEY_X, &t::PRO_L_KEY_Y),
        Some(OneS | OneM | One12 | One13Touch | OneByWacomS) | None => {
            (d::PX_0_23 + index as f32 * d::PX_0_18, d::PX_0_13)
        }
    }
}
/// Active surface inside the trimmed whole-tablet image.
pub fn active_surface(product: u16) -> [f32; 4] {
    match family(product) {
        Some(IntuosM | IntuosMWireless) => t::INTUOS_M_ACTIVE,
        Some(OneS | OneM | OneByWacomS) => t::ONE_ACTIVE,
        Some(One12) => t::ONE_12_ACTIVE,
        Some(One13Touch) => t::ONE_13_ACTIVE,
        Some(ProS) => t::PRO_S_ACTIVE,
        Some(ProM) => t::PRO_M_ACTIVE,
        Some(ProL) => t::PRO_L_ACTIVE,
        Some(IntuosS | IntuosSWireless | Pro2013S | Pro2013M | Pro2013L) | None => {
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
    match family(product)? {
        ProS => Some(t::PRO_S_PANEL_RING),
        ProM => Some(t::PRO_M_PANEL_RING),
        ProL => Some(t::PRO_L_PANEL_RING),
        _ => None,
    }
}
/// ExpressKeys panel artwork of the Intuos Pro lines.
fn keys_panel(family: Option<Family>) -> Option<(&'static str, &'static [u8])> {
    Some(match family? {
        ProS => (
            "pro-s-keys",
            include_bytes!("../../../assets/devices/intuos-pro-s-keys-wacom.png"),
        ),
        ProM => (
            "pro-m-keys",
            include_bytes!("../../../assets/devices/intuos-pro-m-keys-wacom.png"),
        ),
        ProL => (
            "pro-l-keys",
            include_bytes!("../../../assets/devices/intuos-pro-l-keys-wacom.png"),
        ),
        // Intuos Pro 2013: the ExpressKeys panel only, as in the macOS driver.
        Pro2013S => (
            "pro-2013-s-keys",
            include_bytes!("../../../assets/devices/intuos-pro-2013-s-keys-wacom.png"),
        ),
        Pro2013M | Pro2013L => (
            "pro-2013-ml-keys",
            include_bytes!("../../../assets/devices/intuos-pro-2013-ml-keys-wacom.png"),
        ),
        _ => return None,
    })
}
pub fn tablet_art(ui: &Ui, product: u16, name: &str, area: Rect) {
    let family = family(product);
    if let Some((key, bytes)) = keys_panel(family) {
        crate::assets::glow(ui, area);
        crate::assets::artwork(ui, &format!("{name}-{key}"), bytes, area);
        return;
    }
    if intuos_art(product) {
        let art = device_art(product);
        crate::assets::glow(ui, area);
        let cache = if family == Some(IntuosS) {
            name.to_owned()
        } else {
            format!("{name}-{}", art.name)
        };
        crate::assets::artwork(ui, &cache, art.bytes, area);
        return;
    }
    use crate::theme::settings_layout as d;
    let count = tablet_config::express_key_count(product);
    let mut buttons = String::new();
    for i in 0..count {
        let (x, y) = key_spot(product, i, count);
        buttons += &include_str!("../../../assets/devices/tablet-generic-button.svg")
            .replace("{x}", &(x * d::PX_560 - d::PX_13).to_string())
            .replace("{y}", &(y * d::PX_360 - d::PX_8).to_string());
    }
    let svg = include_str!("../../../assets/devices/tablet-generic.svg")
        .replace("{x}", "55")
        .replace("{y}", "73")
        .replace("{width}", "450")
        .replace("{ring}", "")
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
