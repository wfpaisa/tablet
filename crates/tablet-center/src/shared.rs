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

pub fn device_name(status: &tablet_config::Response) -> String {
    let Some(device) = &status.device else {
        return "No tablet connected".into();
    };
    let (name, model) = match device.product {
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
        _ => (device.name.as_str(), ""),
    };
    format!("{name} {model}").trim().into()
}

pub fn intuos_art(product: u16) -> bool {
    matches!(product, 0x0374 | 0x0376 | 0x0377 | 0x03c6)
}
pub fn key_spot(product: u16, index: usize, count: usize) -> (f32, f32) {
    use crate::theme::settings_layout as d;
    if intuos_art(product) {
        return (t::INTUOS_KEY_X[index.min(3)], t::INTUOS_KEY_Y);
    }
    if matches!(
        product,
        0x0392 | 0x0393 | 0x03dd | 0x0357 | 0x0360 | 0x0358 | 0x0361
    ) {
        (
            d::PX_0_1,
            d::PX_0_16
                + index as f32 * (d::PX_0_68 / (count - 1).max(1) as f32)
                + if index < count / 2 {
                    -d::PX_0_04
                } else {
                    d::PX_0_04
                },
        )
    } else {
        (d::PX_0_23 + index as f32 * d::PX_0_18, d::PX_0_13)
    }
}
pub fn tablet_art(ui: &Ui, product: u16, name: &str, area: Rect) {
    if intuos_art(product) {
        crate::assets::glow(ui, area);
        crate::assets::artwork(
            ui,
            name,
            include_bytes!("../../../assets/devices/intuos-s.png"),
            area,
        );
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
