//! Monitor layout from GNOME (org.gnome.Mutter.DisplayConfig).

use std::collections::HashMap;
use tablet_config::Display;
use zbus::{blocking::Connection, zvariant::OwnedValue};

const DEST: &str = "org.gnome.Mutter.DisplayConfig";
const PATH: &str = "/org/gnome/Mutter/DisplayConfig";

type Props = HashMap<String, OwnedValue>;
/// (connector, vendor, product, serial)
type Spec = (String, String, String, String);
type Mode = (String, i32, i32, f64, f64, Vec<f64>, Props);
type Monitor = (Spec, Vec<Mode>, Props);
type Logical = (i32, i32, f64, u32, bool, Vec<Spec>, Props);
type State = (u32, Vec<Monitor>, Vec<Logical>, Props);

/// Layout mode 2 means logical monitors are sized in physical pixels.
const LAYOUT_PHYSICAL: u32 = 2;

pub fn current(conn: &Connection) -> zbus::Result<Vec<Display>> {
    let reply = conn.call_method(Some(DEST), PATH, Some(DEST), "GetCurrentState", &())?;
    let (_, monitors, logical, props): State = reply.body().deserialize()?;
    let physical_layout = props
        .get("layout-mode")
        .and_then(|v| u32::try_from(v).ok())
        .is_some_and(|mode| mode == LAYOUT_PHYSICAL);

    let mut displays = Vec::new();
    for (x, y, scale, transform, primary, specs, _) in logical {
        let Some(spec) = specs.first() else { continue };
        let Some((_, modes, mprops)) = monitors.iter().find(|(s, _, _)| s == spec) else { continue };
        let Some(mode) = modes.iter().find(|m| is_true(&m.6, "is-current")) else { continue };
        let (mut w, mut h) = (f64::from(mode.1), f64::from(mode.2));
        // Transforms 1, 3, 5 and 7 rotate by 90 or 270 degrees.
        if transform % 2 == 1 {
            (w, h) = (h, w);
        }
        if !physical_layout && scale > 0.0 {
            (w, h) = (w / scale, h / scale);
        }
        let name = mprops
            .get("display-name")
            .and_then(|v| String::try_from(v.try_clone().ok()?).ok())
            .unwrap_or_else(|| spec.2.clone());
        displays.push(Display {
            id: format!("{}|{}|{}", spec.1, spec.2, spec.3),
            connector: spec.0.clone(),
            name,
            x: x.into(),
            y: y.into(),
            width: w,
            height: h,
            primary,
        });
    }
    displays.sort_by(|a, b| b.primary.cmp(&a.primary).then(a.x.total_cmp(&b.x)));
    Ok(displays)
}

fn is_true(props: &Props, key: &str) -> bool {
    props.get(key).and_then(|v| bool::try_from(v).ok()).unwrap_or(false)
}

/// Calls `on_change` with the new layout whenever monitors change.
pub fn watch(conn: &Connection, mut on_change: impl FnMut(Vec<Display>)) -> zbus::Result<()> {
    let proxy = zbus::blocking::Proxy::new(conn, DEST, PATH, DEST)?;
    for _ in proxy.receive_signal("MonitorsChanged")? {
        if let Ok(displays) = current(conn) {
            on_change(displays);
        }
    }
    Ok(())
}
