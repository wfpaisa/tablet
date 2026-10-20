//! tabletd: grabs the Wacom pen, applies Tablet Center settings and re-emits
//! the result through uinput. See `pen.rs` for the behaviour.

mod devices;
mod displays;
mod mapping;
mod mice;
mod pad;
mod pen;
mod shell;

use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode, SynchronizationCode};
use signal_hook::{consts::{SIGHUP, SIGINT, SIGTERM}, iterator::Signals};
use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, Mutex, RwLock,
    },
    thread,
    os::fd::AsRawFd,
    time::{Duration, Instant},
};
use tablet_config::{Config, DeviceInfo, Display, Gesture, KeyAction, PenState, Request, Response, Shortcut};

#[derive(Default)]
struct Shared {
    config: RwLock<Config>,
    focused_app: Mutex<Option<String>>,
    /// Bumped on every reload so the device loop picks up new settings.
    generation: AtomicU64,
    device: Mutex<Option<DeviceInfo>>,
    pen: Mutex<PenState>,
    displays: Mutex<Vec<Display>>,
    /// [`Gesture::id`] of a held ExpressKey assigned to a pen gesture, or 0.
    pad_gesture: AtomicU8,
    pad_effects: AtomicU8,
    mode_toggles: AtomicU64,
    relative_mode: AtomicBool,
    mouse_mode: AtomicBool,
    /// Last pen position sent to the pointer (after mapping), so ExpressKey
    /// clicks land under the pen.
    pen_pos: Mutex<(i32, i32)>,
    /// When the virtual mouse last emitted (pen buttons, gestures or
    /// ExpressKeys); see [`POINTER_HIDE_DELAY`].
    pointer_used: Mutex<Option<Instant>>,
    /// Target picked with Display toggle; `None` means the configured one.
    /// Cleared whenever the settings are reloaded.
    display_override: Mutex<Option<Option<String>>>,
    /// Virtual keyboard for ExpressKey shortcuts, while a pad is attached.
    keyboard: Mutex<Option<VirtualDevice>>,
    /// The uinput devices while a pen is attached. Shutdown takes them from
    /// here so it can retire them cleanly while the event loop is blocked.
    outputs: Mutex<Option<Outputs>>,
    /// The attached tablet, to place on-screen shortcuts under the pen.
    tablet: Mutex<Option<mapping::Tablet>>,
    /// ExpressKeys held right now, for the UI.
    pad_keys: Mutex<Vec<bool>>,
    /// The user went back to the mouse while the pen hovered, so the virtual
    /// pen was taken out of proximity; see [`mouse_moved`].
    mouse_owns_cursor: AtomicBool,
    /// Pen position when that happened.
    hidden_at: Mutex<(i32, i32)>,
    /// The pen gave way to the mouse pointer after a click or scroll; see
    /// [`pen_gives_way`].
    following: AtomicBool,
    /// An ExpressKey clicked through the mouse; the pen loop gives way next.
    pad_clicked: AtomicBool,
    /// Feedback drawn by the GNOME Shell extension.
    shell: Mutex<Option<std::sync::mpsc::Sender<shell::Feedback>>>,
    /// An ExpressKey holds a mouse button (e.g. a middle-button drag), so
    /// the pen keeps driving the pointer until it is let go.
    pad_button_held: AtomicBool,
    /// When the pen last moved or touched, so a nudged mouse does not take
    /// the cursor from a pen in use.
    pen_active: Mutex<Option<Instant>>,
    /// `tablet-center --shortcut-host`, which shows on-screen shortcuts.
    shortcut_host: Mutex<Option<(Child, ChildStdin)>>,
}

struct Outputs {
    pen: VirtualDevice,
    pointer: VirtualDevice,
    /// Relative mode's mouse; see [`devices::virtual_relative_mouse`].
    relative: VirtualDevice,
}

impl Outputs {
    /// Removes the devices without leaving the compositor mid-gesture.
    /// Destroying a uinput tablet while its tool is in proximity (or a
    /// button is held) has crashed gnome-shell (SIGSEGV in mutter's event
    /// filters), so release everything first and give it time to settle.
    fn retire(mut self) {
        let up = |code: KeyCode| InputEvent::new(EventType::KEY.0, code.0, 0);
        let pressure = InputEvent::new(EventType::ABSOLUTE.0, evdev::AbsoluteAxisCode::ABS_PRESSURE.0, 0);
        let _ = self.pen.emit(&[up(KeyCode::BTN_TOUCH), pressure, up(KeyCode::BTN_STYLUS), up(KeyCode::BTN_STYLUS2)]);
        let _ = self.pen.emit(&[up(KeyCode::BTN_TOOL_PEN), up(KeyCode::BTN_TOOL_RUBBER)]);
        let buttons = [KeyCode::BTN_LEFT, KeyCode::BTN_RIGHT, KeyCode::BTN_MIDDLE, KeyCode::BTN_SIDE, KeyCode::BTN_EXTRA];
        let _ = self.pointer.emit(&buttons.map(up));
        let _ = self.relative.emit(&[up(KeyCode::BTN_LEFT), up(KeyCode::BTN_RIGHT), up(KeyCode::BTN_MIDDLE)]);
        thread::sleep(Duration::from_millis(300));
        drop(self);
        thread::sleep(Duration::from_millis(300));
    }
}

fn effective_config(shared: &Shared) -> Config {
    shared.config.read().unwrap().effective(shared.focused_app.lock().unwrap().as_deref())
}

fn main() {
    let shared = Arc::new(Shared {
        config: RwLock::new(tablet_config::load()),
        shell: Mutex::new(Some(shell::spawn())),
        ..Default::default()
    });
    let focus = Arc::clone(&shared);
    shell::watch_focus(move |id| {
        let mut current = focus.focused_app.lock().unwrap();
        if *current != id {
            *current = id;
            focus.generation.fetch_add(1, Ordering::AcqRel);
        }
    });
    let ipc = Arc::clone(&shared);
    thread::spawn(move || {
        if let Err(e) = serve(&ipc) {
            eprintln!("tabletd: socket disabled: {e}");
        }
    });
    let signals = Arc::clone(&shared);
    thread::spawn(move || shutdown_on_signal(&signals));
    let pads = Arc::clone(&shared);
    thread::spawn(move || pad_loop(&pads));
    let shortcuts = Arc::clone(&shared);
    thread::spawn(move || {
        // Give the session a moment to come up when started with it.
        thread::sleep(Duration::from_secs(3));
        prepare_shortcuts(&shortcuts);
    });
    let mice = Arc::clone(&shared);
    thread::spawn(move || mice::watch(move || mouse_moved(&mice)));
    let monitors = Arc::clone(&shared);
    thread::spawn(move || {
        if let Err(e) = track_displays(&monitors) {
            eprintln!("tabletd: monitor layout unavailable, mapping to the whole desktop: {e}");
        }
    });

    wait_for_shell();
    let mut waiting_logged = false;
    let mut last_error = String::new();
    loop {
        match devices::find_pen() {
            Some(pen) => {
                waiting_logged = false;
                if let Err(e) = run(pen, &shared) {
                    let message = if e.raw_os_error() == Some(libc_ebusy()) {
                        "the pen is grabbed by another program".to_string()
                    } else {
                        e.to_string()
                    };
                    if message != last_error {
                        eprintln!("tabletd: device stopped: {message}; retrying every second");
                        last_error = message;
                    }
                } else {
                    last_error.clear();
                }
                *shared.device.lock().unwrap() = None;
            }
            None if !waiting_logged => {
                eprintln!("tabletd: waiting for a Wacom pen…");
                waiting_logged = true;
            }
            None => {}
        }
        wait_for_hotplug();
    }
}

/// Returns when /dev/input changes or after a second. Taking the tablet
/// quickly after it is plugged in keeps the window in which gnome-shell
/// drives it on its own short.
fn wait_for_hotplug() {
    let before = devices::input_nodes();
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        if devices::input_nodes() != before {
            return;
        }
    }
}

/// How long GNOME Shell must have been running before we take the pen.
/// A tablet appearing or changing proximity while the shell starts up has
/// crashed it at login (mutter dereferences a NULL tablet on proximity
/// out), so until then the pen stays with GNOME, unmapped but working.
const SHELL_SETTLE: Duration = Duration::from_secs(10);

/// Returns once GNOME Shell has been up for [`SHELL_SETTLE`], at once if it
/// already has (a restart mid-session), or after a minute without a shell.
fn wait_for_shell() {
    let Ok(conn) = zbus::blocking::Connection::session() else { return };
    let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(&conn) else { return };
    let Ok(name) = zbus::names::BusName::try_from("org.gnome.Shell") else { return };
    let deadline = Instant::now() + Duration::from_secs(60);
    let pid = loop {
        if let Ok(pid) = dbus.get_connection_unix_process_id(name.clone()) {
            break pid;
        }
        if Instant::now() > deadline {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    };
    if let Some(left) = process_age(pid).and_then(|age| SHELL_SETTLE.checked_sub(age)) {
        eprintln!("tabletd: GNOME Shell is starting; taking the pen in {}s", left.as_secs().max(1));
        thread::sleep(left);
    }
}

/// Time since the process started, from /proc.
fn process_age(pid: u32) -> Option<Duration> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 22 (starttime), counted after the parenthesised command name.
    let start: f64 = stat.rsplit_once(')')?.1.split_whitespace().nth(19)?.parse().ok()?;
    let uptime: f64 = fs::read_to_string("/proc/uptime").ok()?.split_whitespace().next()?.parse().ok()?;
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    Some(Duration::from_secs_f64((uptime - start / hz).max(0.0)))
}

/// EBUSY: another process holds an exclusive grab on the device.
const fn libc_ebusy() -> i32 {
    16
}

fn run(mut pen: evdev::Device, shared: &Shared) -> io::Result<()> {
    let geometry = devices::geometry(&pen)?;
    let tablet = devices::tablet(&pen)?;
    let info = DeviceInfo {
        name: pen.name().unwrap_or("Tablet").to_string(),
        vendor: pen.input_id().vendor(),
        product: pen.input_id().product(),
        width_mm: tablet.width_mm,
        height_mm: tablet.height_mm,
    };
    take_over(&mut pen)?;
    {
        let mut keyboard = shared.keyboard.lock().unwrap();
        if keyboard.is_none() { *keyboard = Some(devices::virtual_keyboard()?); }
    }
    *shared.outputs.lock().unwrap() =
        Some(Outputs {
            pen: devices::virtual_pen(&pen)?,
            pointer: devices::virtual_pointer(&pen)?,
            relative: devices::virtual_relative_mouse()?,
        });
    let result = pump(&mut pen, &info, geometry, tablet, shared);
    precision_feedback(shared, false);
    // Release pen shortcuts as well as Zoom if the input device disappeared.
    if let Some(keyboard) = shared.keyboard.lock().unwrap().as_mut() {
        let releases: Vec<_> = (1..=248).map(|code| InputEvent::new(EventType::KEY.0, code, 0)).collect();
        let _ = keyboard.emit(&releases);
    }
    if let Some(outputs) = shared.outputs.lock().unwrap().take() {
        outputs.retire();
    }
    result
}

/// Grabs the pen while it is out of proximity. Grabbing mid-hover would
/// leave gnome-shell with a proximity in that never gets its proximity out,
/// so if the pen is near the tablet (e.g. lying on it when plugged in) let
/// the desktop see it leave first.
fn take_over(pen: &mut evdev::Device) -> io::Result<()> {
    loop {
        if devices::in_proximity(pen)? {
            eprintln!("tabletd: pen is over the tablet; taking over once it moves away");
            while devices::in_proximity(pen)? {
                pen.fetch_events()?.for_each(drop);
            }
        }
        pen.grab()?;
        if !devices::in_proximity(pen)? {
            return Ok(());
        }
        // It came back between the check and the grab.
        pen.ungrab()?;
    }
}

fn pump(
    pen: &mut evdev::Device,
    info: &DeviceInfo,
    geometry: pen::Geometry,
    tablet: mapping::Tablet,
    shared: &Shared,
) -> io::Result<()> {
    let info = info.clone();
    *shared.tablet.lock().unwrap() = Some(tablet);
    eprintln!("tabletd: using {} ({:04x}:{:04x})", info.name, info.vendor, info.product);
    *shared.device.lock().unwrap() = Some(info);

    let mut generation = shared.generation.load(Ordering::Acquire);
    let mut processor = pen::PenProcessor::new(effective_config(shared), geometry);
    let new_mapping = || {
        let mut settings = shared.config.read().unwrap().mapping.clone();
        if let Some(display) = shared.display_override.lock().unwrap().clone() {
            settings.display = display;
        }
        mapping::Mapping::new(&settings, &shared.displays.lock().unwrap(), tablet)
    };
    let mut map = new_mapping();
    let mut frame = Vec::new();
    let pad_gesture = || Gesture::from_id(shared.pad_gesture.load(Ordering::Acquire));
    let mut last_tick = Instant::now();
    let mut nudge = 1;
    let mut ring = false;
    let mut mode_toggles = shared.mode_toggles.load(Ordering::Acquire);
    loop {
        let effects = shared.pad_effects.load(Ordering::Acquire);
        let effects_changed = processor.set_pad_effects(effects);
        let toggles = shared.mode_toggles.load(Ordering::Acquire);
        let mode_changed = (toggles - mode_toggles) % 2 != 0;
        if mode_changed { processor.toggle_mode(); }
        mode_toggles = toggles;
        shared.relative_mode.store(processor.relative_mode(), Ordering::Release);
        shared.mouse_mode.store(processor.mouse_mode(), Ordering::Release);
        // A held right-or-middle button shows a ring until it is decided.
        if processor.hold_pending() != ring {
            ring = !ring;
            hold_ring(shared, ring, processor.hold_time());
        }
        // Joystick gestures keep emitting while the pen rests, so wake up
        // regularly while one is active.
        let ticking = effects_changed || mode_changed || processor.wants_ticks() || pad_gesture().is_some() || effects != 0;
        let hide_in = shared.pointer_used.lock().unwrap().map(|t| POINTER_HIDE_DELAY.saturating_sub(t.elapsed()));
        let timeout = match (ticking.then_some(TICK), hide_in) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let readable = wait_readable(pen, Some(timeout.unwrap_or(TICK).min(TICK)))?;
        if hide_in.is_some() {
            // Mouse mode has no tablet cursor to give way to.
            let pen_cursor = !processor.mouse_mode() && !shared.mouse_owns_cursor.load(Ordering::Acquire);
            hide_pointer(shared, processor.state().in_proximity && pen_cursor, &mut nudge)?;
        }
        if effects_changed || mode_changed || (ticking && last_tick.elapsed() >= TICK) {
            let dt = last_tick.elapsed().as_secs_f64().min(0.1);
            let mut out = processor.process_frame(&[], pad_gesture());
            let tick = processor.tick(dt, pad_gesture());
            out.mouse.extend(tick.mouse);
            out.keyboard.extend(tick.keyboard);
            remap(&map, &mut out.pen);
            remember_position(shared, &out.pen);
            out.mouse.iter_mut().for_each(|batch| remap(&map, batch));
            // A held button may just have become a middle-button drag.
            let state = processor.state();
            let hovering = state.in_proximity && !processor.mouse_mode();
            pen_gives_way(shared, &mut out, hovering, state.tip_down && !state.panning);
            emit_pen(shared, &out)?;
            *shared.pen.lock().unwrap() = processor.state();
            last_tick = Instant::now();
        } else if !ticking {
            last_tick = Instant::now();
        }
        if !readable {
            continue;
        }
        for ev in pen.fetch_events()? {
            let report = ev.event_type() == EventType::SYNCHRONIZATION
                && ev.code() == SynchronizationCode::SYN_REPORT.0;
            if !report {
                frame.push(ev);
                continue;
            }
            let current = shared.generation.load(Ordering::Acquire);
            if current != generation {
                generation = current;
                processor.set_config(effective_config(shared));
                map = new_mapping();
            }
            let mut out = processor.process_frame(&frame, pad_gesture());
            shared.relative_mode.store(processor.relative_mode(), Ordering::Release);
            shared.mouse_mode.store(processor.mouse_mode(), Ordering::Release);
            remap(&map, &mut out.pen);
            let was_near = shared.pen.lock().unwrap().in_proximity;
            remember_position(shared, &out.pen);
            if !out.pen.is_empty() {
                *shared.pen_active.lock().unwrap() = Some(Instant::now());
            }
            // While the mouse has the cursor, leaving must not pull it back.
            let mouse_has_it = shared.mouse_owns_cursor.load(Ordering::Acquire);
            pen_takes_cursor_back(shared, &mut out.pen, geometry);
            if was_near && !mouse_has_it && !processor.state().in_proximity && !processor.mouse_mode() {
                mouse_to_pen(shared);
            }
            if out.toggle_display {
                toggle_display(shared);
            }
            if let Some(shortcut) = out.shortcut {
                show_shortcut(shared, shortcut, false);
            }
            out.mouse.iter_mut().for_each(|batch| remap(&map, batch));
            let state = processor.state();
            let hovering = state.in_proximity && !processor.mouse_mode();
            pen_gives_way(shared, &mut out, hovering, state.tip_down && !state.panning);
            frame.clear();
            if !emit_pen(shared, &out)? {
                return Ok(()); // shutting down
            }
            *shared.pen.lock().unwrap() = processor.state();
        }
    }
}

/// How often joystick gestures emit while active.
const TICK: Duration = Duration::from_millis(8);

/// Mutter hides the mouse pointer on tablet events, but only once 300 ms
/// have passed since the last pointer event (HIDDEN_POINTER_TIMEOUT in
/// meta-backend.c). Clicks and scrolling go through the virtual mouse, so
/// after it emits we nudge the pen this long after, in case it is resting.
const POINTER_HIDE_DELAY: Duration = Duration::from_millis(350);

/// Moves the pen by one tablet unit (well under a pixel) once the virtual
/// mouse has been idle for [`POINTER_HIDE_DELAY`], so mutter hides the
/// mouse pointer and only the pen's cursor stays on screen.
fn hide_pointer(shared: &Shared, in_proximity: bool, nudge: &mut i32) -> io::Result<()> {
    let mut used = shared.pointer_used.lock().unwrap();
    if !used.is_some_and(|t| t.elapsed() >= POINTER_HIDE_DELAY) {
        return Ok(());
    }
    // Never while the tip, a side button or an ExpressKey holds a click on
    // the pointer: wait for the release.
    let holding = {
        let pen = shared.pen.lock().unwrap();
        pen.tip_down || pen.mouse_held || shared.pad_button_held.load(Ordering::Acquire)
    };
    if shared.following.load(Ordering::Acquire) && holding {
        *used = Some(Instant::now());
        return Ok(());
    }
    *used = None;
    drop(used);
    if shared.following.swap(false, Ordering::AcqRel) {
        // The pen rested long enough: bring its cursor back where the
        // mouse pointer is, and mutter hides the pointer.
        let (x, y) = *shared.pen_pos.lock().unwrap();
        let abs = |code: evdev::AbsoluteAxisCode, v| InputEvent::new(EventType::ABSOLUTE.0, code.0, v);
        if let Some(outputs) = shared.outputs.lock().unwrap().as_mut() {
            outputs.pen.emit(&[
                InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOOL_PEN.0, 1),
                abs(evdev::AbsoluteAxisCode::ABS_X, x),
                abs(evdev::AbsoluteAxisCode::ABS_Y, y),
            ])?;
        }
        return Ok(());
    }
    if !in_proximity {
        return Ok(());
    }
    let x = {
        let mut pos = shared.pen_pos.lock().unwrap();
        pos.0 += *nudge;
        pos.0
    };
    *nudge = -*nudge;
    let mut outputs = shared.outputs.lock().unwrap();
    if let Some(outputs) = outputs.as_mut() {
        outputs.pen.emit(&[InputEvent::new(EventType::ABSOLUTE.0, evdev::AbsoluteAxisCode::ABS_X.0, x)])?;
    }
    Ok(())
}

/// Waits until the pen has events (true) or `timeout` passes (false).
fn wait_readable(pen: &evdev::Device, timeout: Option<Duration>) -> io::Result<bool> {
    let mut fd = libc::pollfd { fd: pen.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    let ms = timeout.map_or(-1, |t| t.as_millis() as i32);
    loop {
        // SAFETY: one valid pollfd for the duration of the call.
        match unsafe { libc::poll(&mut fd, 1, ms) } {
            -1 if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted => continue,
            -1 => return Err(io::Error::last_os_error()),
            n => return Ok(n > 0),
        }
    }
}

/// Sends a pen [`pen::Output`]; false once shutdown took the devices.
fn emit_pen(shared: &Shared, out: &pen::Output) -> io::Result<bool> {
    action_effects(out.run.as_deref(), out.on_screen_keyboard);
    if let Some(active) = out.precision {
        precision_feedback(shared, active);
    }
    if !out.keyboard.is_empty() {
        if let Some(keyboard) = shared.keyboard.lock().unwrap().as_mut() {
            for batch in &out.keyboard {
                keyboard.emit(batch)?;
            }
        }
    }
    let mut outputs = shared.outputs.lock().unwrap();
    let Some(outputs) = outputs.as_mut() else {
        return Ok(false);
    };
    if !out.pen.is_empty() {
        outputs.pen.emit(&out.pen)?;
    }
    for batch in &out.mouse {
        let relative = batch.iter().any(|e| {
            e.event_type() == EventType::RELATIVE
                && (e.code() == evdev::RelativeAxisCode::REL_X.0 || e.code() == evdev::RelativeAxisCode::REL_Y.0)
        });
        if relative { outputs.relative.emit(batch)? } else { outputs.pointer.emit(batch)? }
    }
    if !out.mouse.is_empty() {
        *shared.pointer_used.lock().unwrap() = Some(Instant::now());
    }
    if let Some(keyboard) = shared.keyboard.lock().unwrap().as_mut() {
        for batch in &out.keyboard_after { keyboard.emit(batch)?; }
    }
    Ok(true)
}

/// Launching and desktop settings never wait on the input thread.
fn action_effects(command: Option<&str>, keyboard: bool) {
    if command.is_none() && !keyboard { return; }
    let command = command.map(str::to_owned);
    thread::spawn(move || {
        if keyboard {
            let enabled = Command::new("gsettings").args(["get", "org.gnome.desktop.a11y.applications", "screen-keyboard-enabled"])
                .output().ok().is_some_and(|o| o.stdout.starts_with(b"true"));
            let _ = Command::new("gsettings").args(["set", "org.gnome.desktop.a11y.applications", "screen-keyboard-enabled", if enabled { "false" } else { "true" }])
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
        if let Some(command) = command.filter(|s| !s.trim().is_empty()) {
            let mut child = Command::new("setsid");
            child.arg("--fork");
            if let Some(id) = command.strip_prefix("desktop:") { child.args(["gtk-launch", id]); }
            else if std::path::Path::new(&command).exists() || command.split_once(':').is_some_and(|(scheme, _)| !scheme.is_empty() && scheme.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))) {
                child.arg("xdg-open").arg(&command);
            } else { child.args(["sh", "-c", &command]); }
            let _ = child.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
        }
    });
}

fn precision_feedback(shared: &Shared, active: bool) {
    let config = shared.config.read().unwrap();
    let feedback = if active && config.precision.overlay {
        let pos = *shared.pen_pos.lock().unwrap();
        let displays = shared.displays.lock().unwrap();
        let Some(tablet) = *shared.tablet.lock().unwrap() else { return };
        let Some((x, y)) = mapping::desktop_point(&displays, tablet, pos) else { return };
        let Some(display) = displays.iter().find(|d| x >= d.x && x < d.x + d.width && y >= d.y && y < d.y + d.height) else { return };
        let scale = f64::from(config.precision.scale.max(2));
        let (w, h) = (display.width / scale, display.height / scale);
        shell::Feedback::Precision { x: (x - w / 2.0).round() as i32, y: (y - h / 2.0).round() as i32, w: w.round() as i32, h: h.round() as i32 }
    } else { shell::Feedback::HidePrecision };
    if let Some(tx) = shared.shell.lock().unwrap().as_ref() { let _ = tx.send(feedback); }
}

/// Moves the pen to the next target: all displays, then each monitor.
fn toggle_display(shared: &Shared) {
    let targets: Vec<Option<String>> =
        std::iter::once(None).chain(shared.displays.lock().unwrap().iter().map(|d| Some(d.id.clone()))).collect();
    let mut current = shared.display_override.lock().unwrap();
    let active = current.clone().unwrap_or_else(|| shared.config.read().unwrap().mapping.display.clone());
    let next = targets.iter().position(|t| *t == active).map_or(0, |i| (i + 1) % targets.len());
    *current = Some(targets[next].clone());
    drop(current);
    shared.generation.fetch_add(1, Ordering::AcqRel);
}

/// Opens an on-screen shortcut under the pen. The host window closes it
/// instead if it is already showing that menu, as on macOS. The menu runs
/// the chosen action through [`Request::Run`]. `centered` (ExpressKeys, and
/// menus opened from another menu) shows it in the middle of the monitor,
/// since the pen may be anywhere or away.
/// Shows the GNOME Shell extension's ring at the pen, or hides it.
fn hold_ring(shared: &Shared, show: bool, wait: Duration) {
    let feedback = if show {
        let pos = *shared.pen_pos.lock().unwrap();
        let Some((x, y)) = shared
            .tablet
            .lock()
            .unwrap()
            .and_then(|tablet| mapping::desktop_point(&shared.displays.lock().unwrap(), tablet, pos))
        else {
            return;
        };
        let ms = wait.as_millis() as u32;
        shell::Feedback::HoldRing { x: x.round() as i32, y: y.round() as i32, ms }
    } else {
        shell::Feedback::Hide
    };
    if let Some(tx) = shared.shell.lock().unwrap().as_ref() {
        let _ = tx.send(feedback);
    }
}

/// How long to wait for the GNOME Shell extension before falling back to
/// the Tablet Center window.
const SHELL_MENU_WAIT: Duration = Duration::from_millis(150);

/// Asks the GNOME Shell extension to draw the menu; false if it did not.
fn shell_shortcut(shared: &Shared, shortcut: Shortcut, centered: bool) -> bool {
    let pos = *shared.pen_pos.lock().unwrap();
    let at = shared
        .tablet
        .lock()
        .unwrap()
        .and_then(|tablet| mapping::desktop_point(&shared.displays.lock().unwrap(), tablet, pos))
        .map(|(x, y)| (x.round() as i32, y.round() as i32));
    let (tx, rx) = std::sync::mpsc::channel();
    let feedback = shell::Feedback::Menu { name: shortcut.name(), at, centered, shown: tx };
    let sent = shared.shell.lock().unwrap().as_ref().is_some_and(|shell| shell.send(feedback).is_ok());
    sent && rx.recv_timeout(SHELL_MENU_WAIT).unwrap_or(false)
}

fn show_shortcut(shared: &Shared, shortcut: Shortcut, centered: bool) {
    // The shell draws it where asked: no pointer to move, no window.
    if shared.config.read().unwrap().shortcuts.in_shell && shell_shortcut(shared, shortcut, centered) {
        return;
    }
    let pos = *shared.pen_pos.lock().unwrap();
    let pen = shared.pen.lock().unwrap().clone();
    let pen_mode = !shared.mouse_mode.load(Ordering::Acquire);
    // Mutter maps the window on the mouse pointer's monitor, which may not
    // be the pen's. Moving the pointer shows it, so a hovering pen gives way
    // to it as for a context menu (see pen_gives_way): one cursor, which
    // the pen drives and clicks with until it rests.
    if let Some(outputs) = shared.outputs.lock().unwrap().as_mut() {
        let abs = |code: evdev::AbsoluteAxisCode, v| InputEvent::new(EventType::ABSOLUTE.0, code.0, v);
        let gives_way = pen_mode
            && pen.in_proximity
            && !pen.tip_down
            && !shared.mouse_owns_cursor.load(Ordering::Acquire)
            && !shared.following.load(Ordering::Acquire);
        if gives_way && outputs.pen.emit(&[InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOOL_PEN.0, 0)]).is_ok() {
            shared.following.store(true, Ordering::Release);
        }
        let _ = outputs.pointer.emit(&[abs(evdev::AbsoluteAxisCode::ABS_X, pos.0), abs(evdev::AbsoluteAxisCode::ABS_Y, pos.1)]);
        *shared.pointer_used.lock().unwrap() = Some(Instant::now());
    }
    let point = shared
        .tablet
        .lock()
        .unwrap()
        .and_then(|tablet| mapping::desktop_point(&shared.displays.lock().unwrap(), tablet, pos));
    let request = serde_json::json!({
        "name": shortcut.name(),
        "at": point.map(|(x, y)| (x.round() as i32, y.round() as i32)),
        "centered": centered,
    });
    let mut line = request.to_string();
    line.push('\n');
    let mut host = shared.shortcut_host.lock().unwrap();
    // A host that exited (or was never started) is started again.
    for _ in 0..2 {
        if host.is_none() {
            *host = spawn_shortcut_host();
        }
        let Some((_, stdin)) = host.as_mut() else { return };
        if stdin.write_all(line.as_bytes()).and_then(|()| stdin.flush()).is_ok() {
            return;
        }
        if let Some((mut child, _)) = host.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn spawn_shortcut_host() -> Option<(Child, ChildStdin)> {
    let spawned = Command::new(tablet_center()).arg("--shortcut-host").stdin(Stdio::piped()).spawn();
    match spawned {
        Ok(mut child) => {
            let stdin = child.stdin.take()?;
            Some((child, stdin))
        }
        Err(e) => {
            eprintln!("tabletd: cannot start the on-screen shortcuts: {e}");
            None
        }
    }
}

/// Starts the shortcut host ahead of time when a button opens a shortcut,
/// so even the first one shows at once.
fn prepare_shortcuts(shared: &Shared) {
    let config = shared.config.read().unwrap();
    let used = [&config.pen.upper_button, &config.pen.lower_button, &config.pen.tip].iter().any(|a| a.shortcut().is_some())
        || config.express_keys.iter().any(|a| a.shortcut().is_some())
        || config.apps.iter().any(|app| {
            [&app.pen.upper_button, &app.pen.lower_button, &app.pen.tip].iter().any(|a| a.shortcut().is_some())
                || app.express_keys.iter().any(|a| a.shortcut().is_some())
        });
    // The shell extension draws them; the window only starts if it fails.
    let used = used && !config.shortcuts.in_shell;
    drop(config);
    let mut host = shared.shortcut_host.lock().unwrap();
    if used && host.is_none() {
        *host = spawn_shortcut_host();
    }
}

/// The installed app, unless `TABLET_CENTER` points at another build.
fn tablet_center() -> PathBuf {
    if let Some(path) = std::env::var_os("TABLET_CENTER") {
        return path.into();
    }
    let installed = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(".local/bin/tablet-center");
    if installed.exists() { installed } else { "tablet-center".into() }
}

/// Runs an action picked from an on-screen shortcut.
fn run_action(shared: &Shared, action: &KeyAction) -> io::Result<()> {
    let out = pad::tap(action, *shared.pen_pos.lock().unwrap());
    emit_pad(shared, &out)?;
    if out.toggle_display {
        toggle_display(shared);
    }
    if let Some(shortcut) = out.shortcut {
        show_shortcut(shared, shortcut, true);
    }
    Ok(())
}

/// GNOME shows the mouse pointer and the pen cursor side by side when both
/// are in use. When the real mouse moves while the pen merely hovers, take
/// the virtual pen out of proximity so only the mouse pointer remains.
fn mouse_moved(shared: &Shared) {
    if shared.mouse_owns_cursor.load(Ordering::Acquire) {
        return;
    }
    // The pen already gave way to the pointer; the mouse just takes it.
    if shared.following.swap(false, Ordering::AcqRel) {
        *shared.hidden_at.lock().unwrap() = *shared.pen_pos.lock().unwrap();
        shared.mouse_owns_cursor.store(true, Ordering::Release);
        return;
    }
    if shared.pen_active.lock().unwrap().is_some_and(|t| t.elapsed() < PEN_IN_USE) {
        return;
    }
    let pen = shared.pen.lock().unwrap().clone();
    let pen_mode = !shared.mouse_mode.load(Ordering::Acquire);
    if !pen_mode || !pen.in_proximity || pen.tip_down || pen.panning {
        return;
    }
    let mut outputs = shared.outputs.lock().unwrap();
    let Some(outputs) = outputs.as_mut() else { return };
    let key = |code: KeyCode| InputEvent::new(EventType::KEY.0, code.0, 0);
    if outputs.pen.emit(&[key(KeyCode::BTN_TOUCH), key(KeyCode::BTN_TOOL_PEN)]).is_ok() {
        *shared.hidden_at.lock().unwrap() = *shared.pen_pos.lock().unwrap();
        shared.mouse_owns_cursor.store(true, Ordering::Release);
    }
}

/// Mutter shows the mouse pointer for at least 300 ms after any event of the
/// virtual mouse (clicks, scrolling), and the pen's own cursor stays next to
/// it. So when the mouse is used while the pen hovers, the pen leaves
/// proximity and drives the mouse pointer instead: one cursor, which even
/// takes the shape apps ask for. Once the mouse has been idle for
/// [`POINTER_HIDE_DELAY`] (and the tip is up), [`hide_pointer`] brings the
/// pen back. Meanwhile the tip clicks through the pointer, so picking from a
/// context menu shows one cursor too.
fn pen_gives_way(shared: &Shared, out: &mut pen::Output, hovering: bool, touching: bool) {
    let at_edge = hovering && !touching && moves(&out.pen) && pen_at_screen_edge(shared);
    let following = shared.following.load(Ordering::Acquire);
    let pad_clicked = shared.pad_clicked.swap(false, Ordering::AcqRel);
    if !hovering || shared.mouse_owns_cursor.load(Ordering::Acquire) {
        if following {
            // Let go of a click the tip may still hold on the pointer.
            out.mouse.push(vec![InputEvent::new(EventType::KEY.0, KeyCode::BTN_LEFT.0, 0)]);
        }
        shared.following.store(false, Ordering::Release);
        return;
    }
    let key = |code: KeyCode, v| InputEvent::new(EventType::KEY.0, code.0, v);
    if !following {
        if touching || (out.mouse.is_empty() && !pad_clicked && !at_edge) {
            return;
        }
        out.pen = vec![key(KeyCode::BTN_TOOL_PEN, 0)];
        if at_edge {
            // The pointer appears where the pen is and carries on from there.
            let (x, y) = *shared.pen_pos.lock().unwrap();
            let abs = |code: evdev::AbsoluteAxisCode, v| InputEvent::new(EventType::ABSOLUTE.0, code.0, v);
            out.mouse.push(vec![abs(evdev::AbsoluteAxisCode::ABS_X, x), abs(evdev::AbsoluteAxisCode::ABS_Y, y)]);
        }
        shared.following.store(true, Ordering::Release);
        return;
    }
    // Motion moves the pointer and the tip clicks with it, as in a context
    // menu that just opened; the rest (pressure, tilt) has no pointer use.
    let frame: Vec<InputEvent> = out
        .pen
        .drain(..)
        .filter_map(|e| match (e.event_type(), e.code()) {
            (EventType::ABSOLUTE, c) if c == evdev::AbsoluteAxisCode::ABS_X.0 || c == evdev::AbsoluteAxisCode::ABS_Y.0 => {
                Some(e)
            }
            (EventType::KEY, c) if c == KeyCode::BTN_TOUCH.0 => Some(key(KeyCode::BTN_LEFT, e.value())),
            _ => None,
        })
        .collect();
    if !frame.is_empty() {
        out.mouse.insert(0, frame);
    }
}

/// Distance (logical px) from a screen edge at which the pen hands over to
/// the pointer.
const EDGE_MARGIN: f64 = 24.0;

/// Dash to Dock's reveal, hot corners and other edge triggers only follow
/// the mouse pointer, never a tablet tool, so near a screen edge the pen
/// gives way to the pointer as for a click (see [`pen_gives_way`]).
fn pen_at_screen_edge(shared: &Shared) -> bool {
    let pos = *shared.pen_pos.lock().unwrap();
    let Some(tablet) = *shared.tablet.lock().unwrap() else { return false };
    let displays = shared.displays.lock().unwrap();
    mapping::desktop_point(&displays, tablet, pos).is_some_and(|p| mapping::near_screen_edge(&displays, p, EDGE_MARGIN))
}

fn moves(events: &[InputEvent]) -> bool {
    events.iter().any(|e| {
        e.event_type() == EventType::ABSOLUTE
            && (e.code() == evdev::AbsoluteAxisCode::ABS_X.0 || e.code() == evdev::AbsoluteAxisCode::ABS_Y.0)
    })
}

/// The pen counts as in use this long after it last moved.
const PEN_IN_USE: Duration = Duration::from_millis(200);

/// When the pen leaves, put the mouse pointer where the pen was, so picking
/// up the mouse carries on from there instead of jumping back. One event,
/// and the pen's cursor is gone by then, so nothing shows twice.
fn mouse_to_pen(shared: &Shared) {
    if shared.mouse_owns_cursor.load(Ordering::Acquire) {
        return;
    }
    let pos = *shared.pen_pos.lock().unwrap();
    if let Some(outputs) = shared.outputs.lock().unwrap().as_mut() {
        let abs = |code: evdev::AbsoluteAxisCode, v| InputEvent::new(EventType::ABSOLUTE.0, code.0, v);
        let _ = outputs.pointer.emit(&[abs(evdev::AbsoluteAxisCode::ABS_X, pos.0), abs(evdev::AbsoluteAxisCode::ABS_Y, pos.1)]);
    }
}

/// Pen travel (mm) that brings the pen cursor back after [`mouse_moved`];
/// less would let hover jitter steal it from the mouse.
const TAKE_BACK_MM: f64 = 3.0;

/// While the mouse owns the cursor, hold back the pen's events; bring the
/// pen back into proximity once it touches or clearly moves.
fn pen_takes_cursor_back(shared: &Shared, events: &mut Vec<InputEvent>, geometry: pen::Geometry) {
    if !shared.mouse_owns_cursor.load(Ordering::Acquire) {
        return;
    }
    let key_value = |code: KeyCode| {
        events.iter().rev().find(|e| e.event_type() == EventType::KEY && e.code() == code.0).map(|e| e.value())
    };
    if key_value(KeyCode::BTN_TOOL_PEN) == Some(0) {
        // The pen left on its own; the next approach starts afresh.
        events.clear();
        shared.mouse_owns_cursor.store(false, Ordering::Release);
        return;
    }
    let pos = *shared.pen_pos.lock().unwrap();
    let from = *shared.hidden_at.lock().unwrap();
    let travel = f64::from(pos.0 - from.0).hypot(f64::from(pos.1 - from.1));
    if key_value(KeyCode::BTN_TOUCH) != Some(1) && travel < TAKE_BACK_MM * geometry.x_units_per_mm {
        events.clear();
        return;
    }
    let abs = |code: evdev::AbsoluteAxisCode, v| InputEvent::new(EventType::ABSOLUTE.0, code.0, v);
    let back = [
        InputEvent::new(EventType::KEY.0, KeyCode::BTN_TOOL_PEN.0, 1),
        abs(evdev::AbsoluteAxisCode::ABS_X, pos.0),
        abs(evdev::AbsoluteAxisCode::ABS_Y, pos.1),
    ];
    events.splice(0..0, back);
    shared.mouse_owns_cursor.store(false, Ordering::Release);
}

fn remember_position(shared: &Shared, events: &[InputEvent]) {
    let mut pos = shared.pen_pos.lock().unwrap();
    for ev in events.iter().filter(|e| e.event_type() == EventType::ABSOLUTE) {
        match evdev::AbsoluteAxisCode(ev.code()) {
            evdev::AbsoluteAxisCode::ABS_X => pos.0 = ev.value(),
            evdev::AbsoluteAxisCode::ABS_Y => pos.1 = ev.value(),
            _ => {}
        }
    }
}

fn pad_loop(shared: &Shared) {
    let mut last_error = String::new();
    loop {
        if let Some(pad) = devices::find_pad() {
            match run_pad(pad, shared) {
                Err(e) if e.to_string() != last_error => {
                    eprintln!("tabletd: ExpressKeys stopped: {e}; retrying every second");
                    last_error = e.to_string();
                }
                _ => {}
            }
        }
        wait_for_hotplug();
    }
}

fn run_pad(mut pad: evdev::Device, shared: &Shared) -> io::Result<()> {
    pad.grab()?;
    {
        let mut keyboard = shared.keyboard.lock().unwrap();
        if keyboard.is_none() { *keyboard = Some(devices::virtual_keyboard()?); }
    }
    eprintln!("tabletd: using {}", pad.name().unwrap_or("tablet pad"));
    let mut generation = shared.generation.load(Ordering::Acquire);
    let mut processor = pad::PadProcessor::new(effective_config(shared).express_keys);
    let mut frame = Vec::new();
    let result = (|| loop {
        for ev in pad.fetch_events()? {
            if ev.event_type() != EventType::SYNCHRONIZATION {
                frame.push(ev);
                continue;
            }
            let current = shared.generation.load(Ordering::Acquire);
            if current != generation {
                generation = current;
                processor.set_actions(effective_config(shared).express_keys);
            }
            let pos = *shared.pen_pos.lock().unwrap();
            remember_keys(shared, &frame);
            let out = processor.process_frame(&frame, pos);
            frame.clear();
            shared.pad_gesture.store(processor.gesture().map_or(0, Gesture::id), Ordering::Release);
            shared.pad_effects.store(processor.effects(), Ordering::Release);
            emit_pad(shared, &out)?;
            if out.toggle_display {
                toggle_display(shared);
            }
            if let Some(shortcut) = out.shortcut {
                show_shortcut(shared, shortcut, true);
            }
        }
    })();
    shared.pad_gesture.store(0, Ordering::Release);
    shared.pad_effects.store(0, Ordering::Release);
    shared.pad_keys.lock().unwrap().clear();
    let _ = emit_pad(shared, &processor.release_all());
    result
}

fn remember_keys(shared: &Shared, frame: &[InputEvent]) {
    let mut keys = shared.pad_keys.lock().unwrap();
    for ev in frame.iter().filter(|e| e.event_type() == EventType::KEY) {
        let Some(i) = ev.code().checked_sub(KeyCode::BTN_0.0).map(usize::from).filter(|&i| i < 10) else { continue };
        if keys.len() <= i {
            keys.resize(i + 1, false);
        }
        keys[i] = ev.value() != 0;
    }
}

fn emit_pad(shared: &Shared, out: &pad::PadOutput) -> io::Result<()> {
    action_effects(out.run.as_deref(), out.on_screen_keyboard);
    if out.toggle_mode { shared.mode_toggles.fetch_add(1, Ordering::AcqRel); }
    if let Some(keyboard) = shared.keyboard.lock().unwrap().as_mut() {
        for batch in &out.keyboard {
            keyboard.emit(batch)?;
        }
    }
    if let Some(outputs) = shared.outputs.lock().unwrap().as_mut() {
        for batch in &out.pointer {
            if shared.relative_mode.load(Ordering::Acquire) {
                let events: Vec<_> = batch.iter().copied().filter(|e| e.event_type() != EventType::ABSOLUTE).collect();
                outputs.pointer.emit(&events)?;
            } else { outputs.pointer.emit(batch)?; }
        }
        if !out.pointer.is_empty() {
            *shared.pointer_used.lock().unwrap() = Some(Instant::now());
            shared.pad_clicked.store(true, Ordering::Release);
            let buttons = [KeyCode::BTN_LEFT, KeyCode::BTN_RIGHT, KeyCode::BTN_MIDDLE, KeyCode::BTN_SIDE, KeyCode::BTN_EXTRA];
            let last = out.pointer.iter().flatten().rev().find(|e| {
                e.event_type() == EventType::KEY && buttons.iter().any(|b| b.0 == e.code())
            });
            if let Some(ev) = last {
                shared.pad_button_held.store(ev.value() != 0, Ordering::Release);
            }
        }
    }
    if let Some(keyboard) = shared.keyboard.lock().unwrap().as_mut() {
        for batch in &out.keyboard_after { keyboard.emit(batch)?; }
    }
    Ok(())
}

/// Moves absolute positions into the configured screen area.
fn remap(map: &mapping::Mapping, events: &mut [InputEvent]) {
    let position = |axis: evdev::AbsoluteAxisCode| {
        events.iter().find(|e| e.event_type() == EventType::ABSOLUTE && e.code() == axis.0).map(|e| e.value())
    };
    let (Some(x), Some(y)) = (position(evdev::AbsoluteAxisCode::ABS_X), position(evdev::AbsoluteAxisCode::ABS_Y)) else {
        return;
    };
    let (x, y) = map.map(x, y);
    for ev in events.iter_mut().filter(|e| e.event_type() == EventType::ABSOLUTE) {
        let value = match evdev::AbsoluteAxisCode(ev.code()) {
            evdev::AbsoluteAxisCode::ABS_X => x,
            evdev::AbsoluteAxisCode::ABS_Y => y,
            _ => continue,
        };
        *ev = InputEvent::new(EventType::ABSOLUTE.0, ev.code(), value);
    }
}

/// Keeps `shared.displays` in sync with GNOME and re-applies the mapping
/// whenever monitors are added, removed or rearranged.
fn track_displays(shared: &Shared) -> zbus::Result<()> {
    let conn = zbus::blocking::Connection::session()?;
    let update = |displays: Vec<Display>| {
        *shared.displays.lock().unwrap() = displays;
        shared.generation.fetch_add(1, Ordering::AcqRel);
    };
    update(displays::current(&conn)?);
    displays::watch(&conn, update)
}

fn shutdown_on_signal(shared: &Shared) {
    let Ok(mut signals) = Signals::new([SIGINT, SIGTERM, SIGHUP]) else {
        eprintln!("tabletd: cannot install signal handlers");
        return;
    };
    if signals.forever().next().is_some() {
        eprintln!("tabletd: shutting down");
        if let Some(mut keyboard) = shared.keyboard.lock().unwrap().take() {
            // Let go of any shortcut whose ExpressKey is still held.
            let modifiers = [
                KeyCode::KEY_LEFTCTRL, KeyCode::KEY_RIGHTCTRL, KeyCode::KEY_LEFTSHIFT, KeyCode::KEY_RIGHTSHIFT,
                KeyCode::KEY_LEFTALT, KeyCode::KEY_RIGHTALT, KeyCode::KEY_LEFTMETA, KeyCode::KEY_RIGHTMETA,
            ];
            let _ = keyboard.emit(&modifiers.map(|k| InputEvent::new(EventType::KEY.0, k.0, 0)));
        }
        if let Some(outputs) = shared.outputs.lock().unwrap().take() {
            outputs.retire();
        }
        let _ = fs::remove_file(tablet_config::socket_path());
        // Exiting closes the physical pen and releases its grab.
        std::process::exit(0);
    }
}

fn serve(shared: &Arc<Shared>) -> io::Result<()> {
    let path = tablet_config::socket_path();
    let _ = fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    for stream in listener.incoming().flatten() {
        let shared = Arc::clone(shared);
        thread::spawn(move || {
            let _ = client(stream, &shared);
        });
    }
    Ok(())
}

fn client(stream: UnixStream, shared: &Shared) -> io::Result<()> {
    let mut writer = stream.try_clone()?;
    for line in BufReader::new(stream).lines() {
        let response = match serde_json::from_str::<Request>(&line?) {
            Ok(Request::Reload) => {
                *shared.config.write().unwrap() = tablet_config::load();
                *shared.display_override.lock().unwrap() = None;
                shared.generation.fetch_add(1, Ordering::AcqRel);
                prepare_shortcuts(shared);
                Response { ok: true, ..Default::default() }
            }
            Ok(Request::RunningApps) => match shell::running_apps() {
                Ok(apps) => Response { ok: true, apps, ..Default::default() },
                Err(e) => Response { error: Some(e.to_string()), ..Default::default() },
            },
            Ok(Request::Status) => Response {
                focused_app: shared.focused_app.lock().unwrap().clone(),
                ok: true,
                device: shared.device.lock().unwrap().clone(),
                pen: Some(shared.pen.lock().unwrap().clone()),
                displays: shared.displays.lock().unwrap().clone(),
                toggled_display: shared.display_override.lock().unwrap().clone(),
                express_keys: shared.pad_keys.lock().unwrap().clone(),
                ..Default::default()
            },
            Ok(Request::Run { action }) => match run_action(shared, &action) {
                Ok(()) => Response { ok: true, ..Default::default() },
                Err(e) => Response { error: Some(e.to_string()), ..Default::default() },
            },
            Err(e) => Response { error: Some(e.to_string()), ..Default::default() },
        };
        serde_json::to_writer(&mut writer, &response)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}
