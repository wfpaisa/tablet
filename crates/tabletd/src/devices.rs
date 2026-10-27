//! Physical pen discovery and the uinput devices that replace it.

use evdev::{
    uinput::VirtualDevice, AbsoluteAxisCode, AttributeSet, BusType, Device, InputId, KeyCode,
    RelativeAxisCode, UinputAbsSetup,
};
use std::io;

use crate::{mapping::Tablet, pen::Geometry};

/// Marks our own uinput devices so discovery never grabs them.
pub const VIRTUAL_SUFFIX: &str = "(tabletd)";

/// First Wacom pen interface (has BTN_TOOL_PEN) that is not one of ours.
pub fn find_pen() -> Option<Device> {
    evdev::enumerate().map(|(_, dev)| dev).find(|dev| {
        tablet_config::wacom_vendor(dev.input_id().vendor())
            && !dev.name().unwrap_or_default().ends_with(VIRTUAL_SUFFIX)
            && dev.supported_keys().is_some_and(|k| k.contains(KeyCode::BTN_TOOL_PEN))
    })
}

/// The ExpressKeys interface of a Wacom tablet: has BTN_0 but no pen tool.
pub fn find_pad() -> Option<Device> {
    evdev::enumerate().map(|(_, dev)| dev).find(|dev| {
        tablet_config::wacom_vendor(dev.input_id().vendor())
            && !dev.name().unwrap_or_default().ends_with(VIRTUAL_SUFFIX)
            && dev.supported_keys().is_some_and(|k| {
                k.contains(KeyCode::BTN_0) && !k.contains(KeyCode::BTN_TOOL_PEN)
            })
    })
}

/// The finger interface of a Wacom touch display or touch tablet:
/// multitouch axes and no pen tool.
pub fn find_touch() -> Option<Device> {
    evdev::enumerate().map(|(_, dev)| dev).find(|dev| {
        tablet_config::wacom_vendor(dev.input_id().vendor())
            && !dev.name().unwrap_or_default().ends_with(VIRTUAL_SUFFIX)
            && dev.supported_absolute_axes().is_some_and(|a| a.contains(AbsoluteAxisCode::ABS_MT_POSITION_X))
            && !dev.supported_keys().is_some_and(|k| k.contains(KeyCode::BTN_TOOL_PEN))
    })
}

/// Keyboard that types ExpressKey shortcuts. Carries every standard key
/// (KEY_ESC ..= KEY_MICMUTE) so any recorded shortcut can be replayed.
pub fn virtual_keyboard() -> io::Result<VirtualDevice> {
    let keys: AttributeSet<KeyCode> = (1..=248).map(KeyCode).collect();
    let name = format!("Tablet ExpressKeys {VIRTUAL_SUFFIX}");
    VirtualDevice::builder()?
        .name(&name)
        .input_id(InputId::new(BusType::BUS_VIRTUAL, SYNTH_VENDOR, SYNTH_KEYBOARD, 1))
        .with_keys(&keys)?
        .build()
}

pub fn geometry(pen: &Device) -> io::Result<Geometry> {
    let mut geo = Geometry { x_units_per_mm: 100.0, y_units_per_mm: 100.0, pressure_max: 2047 };
    for (axis, info) in pen.get_absinfo()? {
        match axis {
            AbsoluteAxisCode::ABS_X if info.resolution() > 0 => geo.x_units_per_mm = info.resolution().into(),
            AbsoluteAxisCode::ABS_Y if info.resolution() > 0 => geo.y_units_per_mm = info.resolution().into(),
            AbsoluteAxisCode::ABS_PRESSURE => geo.pressure_max = info.maximum(),
            _ => {}
        }
    }
    Ok(geo)
}

/// Synthetic ids for our uinput devices (pid.codes test vendor). Reusing the
/// real Wacom vendor/product makes mutter pair the clone with the silent
/// physical tablet and has crashed gnome-shell.
const SYNTH_VENDOR: u16 = 0x1209;
const SYNTH_PEN: u16 = 0x5701;
const SYNTH_POINTER: u16 = 0x5702;
const SYNTH_KEYBOARD: u16 = 0x5703;
const SYNTH_RELATIVE: u16 = 0x5704;

/// Clone of the physical pen's keys, axes and properties, minus MSC_SERIAL.
/// With the real serial libinput treats both pens as one shared tool, and
/// mutter keeps a single "current tablet" for it: proximity seen on the
/// physical tablet (before our grab) then desyncs it and a later proximity
/// out dereferences NULL in gnome-shell. Without a serial the tool is local
/// to our tablet.
pub fn virtual_pen(pen: &Device) -> io::Result<VirtualDevice> {
    let name = format!("{} {VIRTUAL_SUFFIX}", pen.name().unwrap_or("Tablet Pen"));
    let mut builder = VirtualDevice::builder()?
        .name(&name)
        .input_id(InputId::new(BusType::BUS_VIRTUAL, SYNTH_VENDOR, SYNTH_PEN, 1))
        .with_properties(pen.properties())?;
    if let Some(keys) = pen.supported_keys() {
        builder = builder.with_keys(keys)?;
    }
    for (axis, info) in pen.get_absinfo()? {
        builder = builder.with_absolute_axis(&UinputAbsSetup::new(axis, info))?;
    }
    builder.build()
}

/// Absolute pointer used for scrolling and for side buttons mapped to mouse
/// clicks. On Wayland it has its own cursor, separate from the tablet tool's,
/// so the pen coordinates are mirrored onto it before it acts. It shares the
/// pen's X/Y ranges but has no pressure, so udev tags it as a mouse; wheel
/// events on a tablet device would be dropped by libinput.
pub fn virtual_pointer(pen: &Device) -> io::Result<VirtualDevice> {
    let keys = AttributeSet::from_iter([
        KeyCode::BTN_LEFT,
        KeyCode::BTN_RIGHT,
        KeyCode::BTN_MIDDLE,
        KeyCode::BTN_SIDE,
        KeyCode::BTN_EXTRA,
    ]);
    let rel = AttributeSet::from_iter([
        RelativeAxisCode::REL_WHEEL,
        RelativeAxisCode::REL_HWHEEL,
        RelativeAxisCode::REL_WHEEL_HI_RES,
        RelativeAxisCode::REL_HWHEEL_HI_RES,
    ]);
    let name = format!("Tablet Pen Pointer {VIRTUAL_SUFFIX}");
    let mut builder = VirtualDevice::builder()?
        .name(&name)
        .input_id(InputId::new(BusType::BUS_VIRTUAL, SYNTH_VENDOR, SYNTH_POINTER, 1))
        .with_keys(&keys)?
        .with_relative_axes(&rel)?;
    for (axis, info) in pen.get_absinfo()? {
        if matches!(axis, AbsoluteAxisCode::ABS_X | AbsoluteAxisCode::ABS_Y) {
            builder = builder.with_absolute_axis(&UinputAbsSetup::new(axis, info))?;
        }
    }
    builder.build()
}

/// Plain relative mouse for the pen's relative mode. Kept apart from
/// [`virtual_pointer`]: a device with both absolute and relative X/Y may be
/// classified differently by udev and libinput.
pub fn virtual_relative_mouse() -> io::Result<VirtualDevice> {
    let keys = AttributeSet::from_iter([KeyCode::BTN_LEFT, KeyCode::BTN_RIGHT, KeyCode::BTN_MIDDLE]);
    let rel = AttributeSet::from_iter([RelativeAxisCode::REL_X, RelativeAxisCode::REL_Y]);
    let name = format!("Tablet Pen Mouse {VIRTUAL_SUFFIX}");
    VirtualDevice::builder()?
        .name(&name)
        .input_id(InputId::new(BusType::BUS_VIRTUAL, SYNTH_VENDOR, SYNTH_RELATIVE, 1))
        .with_keys(&keys)?
        .with_relative_axes(&rel)?
        .build()
}

/// Pen or eraser currently in proximity of the tablet.
pub fn in_proximity(pen: &Device) -> io::Result<bool> {
    let keys = pen.get_key_state()?;
    Ok(keys.contains(KeyCode::BTN_TOOL_PEN) || keys.contains(KeyCode::BTN_TOOL_RUBBER))
}

/// Names under /dev/input; a change means something was plugged or removed.
pub fn input_nodes() -> Vec<std::ffi::OsString> {
    let mut names: Vec<_> = std::fs::read_dir("/dev/input")
        .map(|dir| dir.filter_map(|e| e.ok().map(|e| e.file_name())).collect())
        .unwrap_or_default();
    names.sort();
    names
}

/// Axis ranges and physical size of the active area.
pub fn tablet(pen: &Device) -> io::Result<Tablet> {
    let mut tablet = Tablet { x_range: (0, 1), y_range: (0, 1), width_mm: 0.0, height_mm: 0.0 };
    for (axis, info) in pen.get_absinfo()? {
        let range = (info.minimum(), info.maximum());
        let mm = |res: i32| if res > 0 { f64::from(range.1 - range.0) / f64::from(res) } else { 0.0 };
        match axis {
            AbsoluteAxisCode::ABS_X => (tablet.x_range, tablet.width_mm) = (range, mm(info.resolution())),
            AbsoluteAxisCode::ABS_Y => (tablet.y_range, tablet.height_mm) = (range, mm(info.resolution())),
            _ => {}
        }
    }
    Ok(tablet)
}

/// Locate the physical input node without depending on its event number.
pub fn sysfs_path(pen: &Device) -> Option<std::path::PathBuf> {
    std::fs::read_dir("/sys/class/input").ok()?.flatten().find_map(|entry| {
        if !entry.file_name().to_string_lossy().starts_with("event") { return None; }
        let path = entry.path().join("device");
        let read = |name| std::fs::read_to_string(path.join(name)).unwrap_or_default().trim().to_string();
        (read("name") == pen.name().unwrap_or_default()
            && read("phys") == pen.physical_path().unwrap_or_default()
            && u16::from_str_radix(&read("id/product"), 16).ok() == Some(pen.input_id().product()))
            .then(|| path.canonicalize().ok()).flatten()
    })
}

pub fn firmware(path: &std::path::Path) -> Option<String> {
    path.ancestors().flat_map(|p| [p.join("version"), p.join("id/version")]).find_map(|p| {
        let value = std::fs::read_to_string(p).ok()?.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

/// The tablet's battery, read again at most every few seconds: live views
/// ask for status about 25 times a second.
pub fn battery(path: Option<&std::path::Path>, product: u16) -> Option<tablet_config::Battery> {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    type Cached = (Instant, Option<std::path::PathBuf>, Option<tablet_config::Battery>);
    static CACHE: Mutex<Option<Cached>> = Mutex::new(None);
    if !tablet_config::wireless_model(product) { return None; }
    let mut cache = CACHE.lock().unwrap();
    if let Some((at, cached_path, battery)) = cache.as_ref() {
        if at.elapsed() < Duration::from_secs(5) && cached_path.as_deref() == path { return battery.clone(); }
    }
    let battery = read_battery(std::path::Path::new("/sys/class/power_supply"), path);
    *cache = Some((Instant::now(), path.map(|p| p.to_path_buf()), battery.clone()));
    battery
}

fn read_battery(root: &std::path::Path, tablet: Option<&std::path::Path>) -> Option<tablet_config::Battery> {
    let mut candidates: Vec<_> = std::fs::read_dir(root).ok()?.flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("wacom_battery_"))
        .map(|e| e.path()).collect();
    candidates.sort();
    let matched = tablet.and_then(|tablet| candidates.iter().find(|battery| {
        let Ok(device) = battery.join("device").canonicalize().or_else(|_| battery.canonicalize()) else { return false; };
        tablet.ancestors().any(|ancestor| {
            let name = ancestor.file_name().map(|n| n.to_string_lossy().to_uppercase()).unwrap_or_default();
            let hid = name.contains(":056A:") || name.contains(":0531:");
            let usb = std::fs::read_to_string(ancestor.join("idVendor")).is_ok_and(|v| matches!(v.trim(), "056a" | "0531"));
            (hid || usb) && device.starts_with(ancestor)
        })
    }));
    let battery = matched.or_else(|| candidates.first())?;
    let percent = std::fs::read_to_string(battery.join("capacity")).ok()?.trim().parse::<u8>().ok()?.min(100);
    let status = std::fs::read_to_string(battery.join("status")).ok()?;
    Some(tablet_config::Battery { percent, charging: status.trim().eq_ignore_ascii_case("charging") })
}

/// GNOME's double-click interval is read only at startup and explicit reload.
pub fn double_click_time() -> std::time::Duration {
    let ms = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.peripherals.mouse", "double-click"])
        .output().ok().filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split_whitespace().last()?.parse::<u64>().ok())
        .filter(|&ms| ms > 0).unwrap_or(400);
    std::time::Duration::from_millis(ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_matches_ancestry_falls_back_and_reads_live_capacity() {
        use std::{fs, os::unix::fs::symlink};
        let root = std::env::current_dir().unwrap().join("target").join(format!("battery-test-{}-{}",
            std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let supplies = root.join("power_supply");
        let first = supplies.join("wacom_battery_0");
        let second = supplies.join("wacom_battery_1");
        let hid = root.join("0003:056A:0376.0001");
        let input = hid.join("input/input20");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::create_dir_all(&input).unwrap();
        symlink(&hid, second.join("device")).unwrap();
        fs::write(first.join("capacity"), "30\n").unwrap();
        fs::write(first.join("status"), "Discharging\n").unwrap();
        fs::write(second.join("capacity"), "85\n").unwrap();
        fs::write(second.join("status"), "Charging\n").unwrap();
        let matched = read_battery(&supplies, Some(&input)).unwrap();
        assert_eq!(matched.percent, 85);
        assert!(matched.charging);
        assert_eq!(read_battery(&supplies, None).unwrap().percent, 30);
        fs::write(second.join("capacity"), "100\n").unwrap();
        fs::write(second.join("status"), "Full\n").unwrap();
        let full = read_battery(&supplies, Some(&input)).unwrap();
        assert_eq!(full.percent, 100);
        assert!(!full.charging);
        fs::create_dir_all(input.join("id")).unwrap();
        fs::write(input.join("id/version"), "0102\n").unwrap();
        assert_eq!(firmware(&input).as_deref(), Some("0102"));
        assert!(battery(Some(&input), 0x0374).is_none(), "wired devices never inherit another tablet's battery");
        fs::remove_dir_all(root).unwrap();
    }
}
