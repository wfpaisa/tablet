//! Watches the real mice and touchpads (read-only, never grabbed) so the pen
//! cursor can step aside when the user goes back to the mouse.

use evdev::{Device, EventType, KeyCode, RelativeAxisCode};
use std::{
    collections::HashSet,
    ffi::OsString,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::devices::{input_nodes, VIRTUAL_SUFFIX};

const WACOM_VENDOR: u16 = 0x056a;

/// Calls `on_move` from a background thread whenever a mouse or touchpad
/// moves the pointer. Picks up devices plugged in later.
pub fn watch(on_move: impl Fn() + Send + Sync + 'static) {
    let on_move = Arc::new(on_move);
    let mut watched: HashSet<PathBuf> = HashSet::new();
    let mut nodes: Vec<OsString> = Vec::new();
    loop {
        let now = input_nodes();
        if now != nodes {
            nodes = now;
            watched.retain(|path| path.exists());
            for (path, dev) in evdev::enumerate() {
                if is_pointer(&dev) && watched.insert(path.clone()) {
                    let on_move = Arc::clone(&on_move);
                    thread::spawn(move || follow(dev, &*on_move));
                }
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn is_pointer(dev: &Device) -> bool {
    let name = dev.name().unwrap_or_default();
    if dev.input_id().vendor() == WACOM_VENDOR || name.ends_with(VIRTUAL_SUFFIX) {
        return false;
    }
    let mouse = dev
        .supported_relative_axes()
        .is_some_and(|r| r.contains(RelativeAxisCode::REL_X) && r.contains(RelativeAxisCode::REL_Y));
    let touchpad = dev.supported_keys().is_some_and(|k| k.contains(KeyCode::BTN_TOOL_FINGER))
        && dev.properties().contains(evdev::PropType::POINTER);
    mouse || touchpad
}

/// Mouse counts (or touchpad events) within [`WINDOW`] that count as the
/// user really moving it. A mouse resting on a desk can report a stray
/// count now and then, which must not take the cursor from the pen.
const MOUSE_TRAVEL: i32 = 12;
const TOUCHPAD_EVENTS: i32 = 6;
const WINDOW: Duration = Duration::from_millis(250);

/// Reports deliberate motion until the device goes away.
fn follow(mut dev: Device, on_move: &(dyn Fn() + Send + Sync)) {
    let mut since = Instant::now();
    let (mut travel, mut touches) = (0, 0);
    while let Ok(events) = dev.fetch_events() {
        if since.elapsed() > WINDOW {
            (since, travel, touches) = (Instant::now(), 0, 0);
        }
        for e in events {
            match e.event_type() {
                EventType::RELATIVE
                    if matches!(RelativeAxisCode(e.code()), RelativeAxisCode::REL_X | RelativeAxisCode::REL_Y) =>
                {
                    travel += e.value().abs()
                }
                EventType::ABSOLUTE => touches += 1,
                _ => {}
            }
        }
        if travel >= MOUSE_TRAVEL || touches >= TOUCHPAD_EVENTS {
            (since, travel, touches) = (Instant::now(), 0, 0);
            on_move();
        }
    }
}
