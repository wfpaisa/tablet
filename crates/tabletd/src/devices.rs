//! Physical pen discovery and the uinput devices that replace it.

use evdev::{
    uinput::VirtualDevice, AbsoluteAxisCode, AttributeSet, BusType, Device, InputId, KeyCode,
    RelativeAxisCode, UinputAbsSetup,
};
use std::io;

use crate::{mapping::Tablet, pen::Geometry};

const WACOM_VENDOR: u16 = 0x056a;
/// Marks our own uinput devices so discovery never grabs them.
pub const VIRTUAL_SUFFIX: &str = "(tabletd)";

/// First Wacom pen interface (has BTN_TOOL_PEN) that is not one of ours.
pub fn find_pen() -> Option<Device> {
    evdev::enumerate().map(|(_, dev)| dev).find(|dev| {
        dev.input_id().vendor() == WACOM_VENDOR
            && !dev.name().unwrap_or_default().ends_with(VIRTUAL_SUFFIX)
            && dev.supported_keys().is_some_and(|k| k.contains(KeyCode::BTN_TOOL_PEN))
    })
}

/// The ExpressKeys interface of a Wacom tablet: has BTN_0 but no pen tool.
pub fn find_pad() -> Option<Device> {
    evdev::enumerate().map(|(_, dev)| dev).find(|dev| {
        dev.input_id().vendor() == WACOM_VENDOR
            && !dev.name().unwrap_or_default().ends_with(VIRTUAL_SUFFIX)
            && dev.supported_keys().is_some_and(|k| {
                k.contains(KeyCode::BTN_0) && !k.contains(KeyCode::BTN_TOOL_PEN)
            })
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
