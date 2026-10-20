//! Turns one evdev frame from the physical pen into frames for the virtual pen
//! and the virtual mouse. No I/O here, so the behaviour is unit-testable.
//!
//! Pan/Scroll (observed on macOS Tablet Center): while the button is held the
//! cursor stays put; touching the tip and dragging scrolls in any direction,
//! proportional to pen travel; letting go stops immediately (no momentum).
//!
//! Pan/Scroll (Legacy), Wheel and Zoom follow the macOS driver's
//! `CPenButton*GestureFromXY` classes: a joystick whose speed grows with the
//! distance from where the tip touched down, so they need [`PenProcessor::tick`].
//!
//! Pan/Zoom pans like Pan/Scroll while the tip drags, and zooms while the pen
//! moves up or down in the air with the button held.

use std::time::{Duration, Instant};

use evdev::{AbsoluteAxisCode as Abs, EventType, InputEvent, KeyCode, RelativeAxisCode as Rel};
use tablet_config::{hold_middle_time, joystick_mm, v120_per_mm, ButtonAction, Config, Gesture, PenMode, PenState, Shortcut};

/// Tablet properties needed to convert counts to millimetres.
#[derive(Debug, Clone, Copy)]
pub struct Geometry {
    pub x_units_per_mm: f64,
    pub y_units_per_mm: f64,
    pub pressure_max: i32,
}

#[derive(Debug, Default)]
pub struct Output {
    pub pen: Vec<InputEvent>,
    /// Each inner Vec is emitted as its own frame (a double click needs four).
    pub mouse: Vec<Vec<InputEvent>>,
    /// Frames for the virtual keyboard (Ctrl while zooming).
    pub keyboard: Vec<Vec<InputEvent>>,
    /// A button assigned to Display toggle was pressed.
    pub toggle_display: bool,
    /// A button assigned to an on-screen shortcut was pressed.
    pub shortcut: Option<Shortcut>,
}

/// What a held side button is currently doing, so its release undoes exactly
/// that even if the settings changed in between.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Held {
    Nothing,
    /// Right-or-middle, still undecided: pressed at `at`, at `since`.
    Pending { since: Instant, at: (i32, i32) },
    Gesture,
    Native(KeyCode),
    Mouse(KeyCode),
}

pub struct PenProcessor {
    config: Config,
    geometry: Geometry,
    x: i32,
    y: i32,
    pressure: i32,
    tip: bool,
    in_proximity: bool,
    /// Side buttons physically held.
    buttons_down: (bool, bool),
    upper: Held,
    lower: Held,
    /// A gesture is active while a side button or an ExpressKey holds it;
    /// the side button wins if both do.
    side_gesture: Option<Gesture>,
    pad_gesture: Option<Gesture>,
    /// Joystick gestures: set while the tip is down.
    stick: Option<Stick>,
    /// Ctrl is held on the virtual keyboard for Zoom.
    ctrl_down: bool,
    /// Where the cursor was frozen when the pan started.
    pan_anchor: (i32, i32),
    pan_last: Option<(i32, i32)>,
    /// Pan/Zoom: last hover position while zooming in the air.
    zoom_last: Option<(i32, i32)>,
    /// Fractional hi-res wheel units not yet emitted (horizontal, vertical).
    wheel_rest: (f64, f64),
    /// Hi-res units since the last legacy notch, per axis.
    notch_acc: (i32, i32),
    /// After a pan ends with the tip still down, keep the tip up until lifted
    /// so releasing the button does not start a click-drag.
    hold_tip_until_lift: bool,
    sent_touch: bool,
    sent_pressure: i32,
    /// Whether the virtual pen is in proximity, as far as the desktop knows.
    /// Mouse mode keeps it out.
    virtual_proximity: bool,
}

const WHEEL_NOTCH: i32 = 120;

/// Joystick state, in tablet counts since the tip touched down (or since the
/// last reversal).
#[derive(Debug, Clone, Copy, Default)]
struct Stick {
    last: (i32, i32),
    acc: (f64, f64),
    /// Fractional output not yet emitted (hi-res units or notches).
    rest: (f64, f64),
}

/// The macOS driver emits `accumulator / speed` steps per tablet packet; the
/// CTL-4100 reports at about this rate.
const PACKETS_PER_SEC: f64 = 133.0;
/// Reversals smaller than this fraction of the speed setting are tremor.
const DEADZONE: f64 = 0.05;
/// An axis dominates when the other moved less than this fraction of it.
const AXIS_DOMINANCE: f64 = 0.33;
/// Legacy: macOS scrolls 1 pixel per step; ~2.4 hi-res units per pixel.
const LEGACY_V120_PER_STEP: f64 = 2.4;
/// Wheel and Zoom notches per second per unit of joystick speed.
const WHEEL_NOTCHES_PER_SEC: f64 = 3.0;
const ZOOM_NOTCHES_PER_SEC: f64 = 1.0;
/// Zoom halves the travel per speed unit (`DAT_1009c4d10` = 0.5).
const ZOOM_TRAVEL: f64 = 0.5;
/// Right-or-middle: holding the button for `pen.hold_time`, or moving the
/// pen this far while it is held, makes it a middle button.
const HOLD_MIDDLE_MM: f64 = 3.0;
/// Pan/Zoom in the air: fraction of the pan gain, so one wheel notch takes a
/// few millimetres of travel instead of one.
const HOVER_ZOOM_GAIN: f64 = 0.25;

impl PenProcessor {
    pub fn new(config: Config, geometry: Geometry) -> Self {
        Self {
            config,
            geometry,
            x: 0,
            y: 0,
            pressure: 0,
            tip: false,
            in_proximity: false,
            buttons_down: (false, false),
            upper: Held::Nothing,
            lower: Held::Nothing,
            side_gesture: None,
            pad_gesture: None,
            stick: None,
            ctrl_down: false,
            pan_anchor: (0, 0),
            pan_last: None,
            zoom_last: None,
            wheel_rest: (0.0, 0.0),
            notch_acc: (0, 0),
            hold_tip_until_lift: false,
            sent_touch: false,
            sent_pressure: 0,
            virtual_proximity: false,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        if self.config.mapping.orientation != config.mapping.orientation {
            self.pan_last = None;
            self.zoom_last = None;
            self.stick = None;
            self.wheel_rest = (0.0, 0.0);
            self.notch_acc = (0, 0);
        }
        self.config = config;
    }

    pub fn state(&self) -> PenState {
        PenState {
            in_proximity: self.in_proximity,
            tip_down: self.tip,
            pressure: f64::from(self.curved_pressure()) / f64::from(self.geometry.pressure_max.max(1)),
            panning: self.panning(),
            upper_button: self.buttons_down.0,
            lower_button: self.buttons_down.1,
            mouse_held: [self.upper, self.lower].iter().any(|h| matches!(h, Held::Mouse(_))),
        }
    }

    fn gesture(&self) -> Option<Gesture> {
        self.side_gesture.or(self.pad_gesture)
    }

    fn panning(&self) -> bool {
        self.gesture().is_some()
    }

    /// Applies a change to a gesture source, starting or ending the gesture
    /// when the combined state changes.
    fn set_pan(&mut self, change: impl FnOnce(&mut Self)) {
        let before = self.gesture();
        change(self);
        let after = self.gesture();
        if before == after {
            return;
        }
        if before.is_none() {
            self.pan_anchor = (self.x, self.y);
        }
        self.pan_last = None;
        self.zoom_last = None;
        self.wheel_rest = (0.0, 0.0);
        self.stick = None;
        if after.is_none() {
            self.hold_tip_until_lift = self.tip;
        }
    }

    /// `pad_gesture` is the gesture of a held ExpressKey, if any.
    pub fn process_frame(&mut self, frame: &[InputEvent], pad_gesture: Option<Gesture>) -> Output {
        let mut out = Output::default();
        let was_panning = self.panning();
        if pad_gesture != self.pad_gesture {
            self.set_pan(|s| s.pad_gesture = pad_gesture);
        }
        let mut moved = false;
        // The Wacom driver reports x = y = 0 as the pen leaves; keep the
        // last real position (it is where the mouse pointer goes next).
        let leaving = frame
            .iter()
            .any(|e| e.event_type() == EventType::KEY && e.code() == KeyCode::BTN_TOOL_PEN.0 && e.value() == 0);

        for ev in frame {
            match ev.event_type() {
                EventType::KEY => self.key(*ev, &mut out),
                EventType::ABSOLUTE => match Abs(ev.code()) {
                    Abs::ABS_X | Abs::ABS_Y if leaving => {}
                    Abs::ABS_X => (self.x, moved) = (ev.value(), true),
                    Abs::ABS_Y => (self.y, moved) = (ev.value(), true),
                    Abs::ABS_PRESSURE => self.pressure = ev.value(),
                    _ => out.pen.push(*ev),
                },
                // The virtual pen has no MSC_SERIAL (see devices::virtual_pen).
                EventType::SYNCHRONIZATION | EventType::MISC => {}
                _ => out.pen.push(*ev),
            }
        }

        if !self.tip {
            self.hold_tip_until_lift = false;
        }
        self.promote_holds(&mut out);
        match self.gesture() {
            Some(g @ (Gesture::PanScroll | Gesture::Scroll | Gesture::PanZoom)) if self.tip => {
                self.zoom_last = None;
                self.scroll(g, &mut out);
            }
            Some(Gesture::PanZoom) => {
                self.pan_last = None;
                if self.in_proximity {
                    self.hover_zoom(&mut out);
                } else {
                    self.zoom_last = None;
                }
            }
            Some(g) if self.tip => self.stick_move(g),
            _ => {
                self.pan_last = None;
                self.stick = None;
            }
        }
        self.release_ctrl_unless_zooming(&mut out);
        if !self.panning() && (moved || was_panning) {
            out.pen.push(abs(Abs::ABS_X, self.x));
            out.pen.push(abs(Abs::ABS_Y, self.y));
        }

        // libinput decides tip contact from pressure on Wacom pens, so a
        // suppressed tip must also report zero pressure.
        let touch = self.tip && !self.panning() && !self.hold_tip_until_lift;
        let pressure = if touch { self.curved_pressure() } else { 0 };
        if pressure != self.sent_pressure {
            out.pen.push(abs(Abs::ABS_PRESSURE, pressure));
            self.sent_pressure = pressure;
        }
        if touch != self.sent_touch {
            out.pen.push(key(KeyCode::BTN_TOUCH, touch));
            self.sent_touch = touch;
        }
        if self.mouse_mode() {
            self.to_mouse(&mut out);
        } else {
            self.sync_proximity(&mut out);
        }
        out
    }

    pub fn mouse_mode(&self) -> bool {
        self.config.pen.mode == PenMode::Mouse
    }

    /// Mouse mode: the pen's position, tip and raw buttons go to the virtual
    /// mouse instead, and the virtual pen leaves proximity so its cursor
    /// does not show next to the mouse pointer.
    fn to_mouse(&mut self, out: &mut Output) {
        let mut frame = Vec::new();
        for ev in std::mem::take(&mut out.pen) {
            match (ev.event_type(), ev.code()) {
                (EventType::ABSOLUTE, c) if c == Abs::ABS_X.0 || c == Abs::ABS_Y.0 => frame.push(ev),
                (EventType::KEY, c) if c == KeyCode::BTN_TOUCH.0 => frame.push(key(KeyCode::BTN_LEFT, ev.value() != 0)),
                (EventType::KEY, c) if c == KeyCode::BTN_STYLUS.0 => frame.push(key(KeyCode::BTN_RIGHT, ev.value() != 0)),
                (EventType::KEY, c) if c == KeyCode::BTN_STYLUS2.0 => frame.push(key(KeyCode::BTN_MIDDLE, ev.value() != 0)),
                _ => {}
            }
        }
        if self.virtual_proximity {
            out.pen = vec![key(KeyCode::BTN_TOUCH, false), abs(Abs::ABS_PRESSURE, 0), key(KeyCode::BTN_TOOL_PEN, false)];
            self.virtual_proximity = false;
        }
        if !frame.is_empty() {
            out.mouse.insert(0, frame);
        }
    }

    /// Pen mode: forward proximity, and bring the virtual pen back in if
    /// the mode changed while the pen hovers.
    fn sync_proximity(&mut self, out: &mut Output) {
        let forwarded = out
            .pen
            .iter()
            .rev()
            .find(|e| e.event_type() == EventType::KEY && e.code() == KeyCode::BTN_TOOL_PEN.0)
            .map(|e| e.value() != 0);
        match forwarded {
            Some(state) => self.virtual_proximity = state,
            None if self.in_proximity && !self.virtual_proximity => {
                out.pen.insert(0, key(KeyCode::BTN_TOOL_PEN, true));
                out.pen.push(abs(Abs::ABS_X, self.x));
                out.pen.push(abs(Abs::ABS_Y, self.y));
                self.virtual_proximity = true;
            }
            None => {}
        }
    }

    fn key(&mut self, ev: InputEvent, out: &mut Output) {
        let code = KeyCode(ev.code());
        let down = ev.value() != 0;
        match code {
            KeyCode::BTN_TOUCH => self.tip = down,
            KeyCode::BTN_STYLUS2 => {
                self.buttons_down.0 = down;
                let action = self.config.pen.upper_button;
                self.upper = self.side_button(self.upper, action, code, down, out);
            }
            KeyCode::BTN_STYLUS => {
                self.buttons_down.1 = down;
                let action = self.config.pen.lower_button;
                self.lower = self.side_button(self.lower, action, code, down, out);
            }
            _ => {
                if code == KeyCode::BTN_TOOL_PEN {
                    self.in_proximity = down;
                }
                out.pen.push(ev);
            }
        }
    }

    fn side_button(
        &mut self,
        held: Held,
        action: ButtonAction,
        code: KeyCode,
        down: bool,
        out: &mut Output,
    ) -> Held {
        if !down {
            match held {
                Held::Gesture => self.set_pan(|s| s.side_gesture = None),
                Held::Native(k) => out.pen.push(key(k, false)),
                Held::Mouse(k) => out.mouse.push(vec![key(k, false)]),
                Held::Pending { at, .. } => {
                    out.mouse.push(vec![abs(Abs::ABS_X, at.0), abs(Abs::ABS_Y, at.1), key(KeyCode::BTN_RIGHT, true)]);
                    out.mouse.push(vec![key(KeyCode::BTN_RIGHT, false)]);
                }
                Held::Nothing => {}
            }
            return Held::Nothing;
        }
        if let Some(gesture) = action.gesture() {
            self.set_pan(|s| s.side_gesture = Some(gesture));
            return Held::Gesture;
        }
        if let Some(shortcut) = action.shortcut() {
            out.shortcut = Some(shortcut);
            return Held::Nothing;
        }
        let mouse_key = match action {
            ButtonAction::Default => {
                out.pen.push(key(code, true));
                return Held::Native(code);
            }
            ButtonAction::Disabled => return Held::Nothing,
            ButtonAction::DisplayToggle => {
                out.toggle_display = true;
                return Held::Nothing;
            }
            ButtonAction::DoubleClick => {
                out.mouse.push(self.at_pen(key(KeyCode::BTN_LEFT, true)));
                for pressed in [false, true, false] {
                    out.mouse.push(vec![key(KeyCode::BTN_LEFT, pressed)]);
                }
                return Held::Nothing;
            }
            // With the tip down there is nothing to drag with the middle
            // button, so it is a plain right click.
            ButtonAction::RightOrHoldMiddle if self.tip => KeyCode::BTN_RIGHT,
            ButtonAction::RightOrHoldMiddle => return Held::Pending { since: Instant::now(), at: (self.x, self.y) },
            ButtonAction::LeftClick => KeyCode::BTN_LEFT,
            ButtonAction::RightClick => KeyCode::BTN_RIGHT,
            ButtonAction::MiddleClick => KeyCode::BTN_MIDDLE,
            ButtonAction::Back => KeyCode::BTN_SIDE,
            ButtonAction::Forward => KeyCode::BTN_EXTRA,
            ButtonAction::PanScroll
            | ButtonAction::PanScrollLegacy
            | ButtonAction::Scroll
            | ButtonAction::Wheel
            | ButtonAction::Zoom
            | ButtonAction::PanZoom
            | ButtonAction::ExpressMenu
            | ButtonAction::RadialMenu
            | ButtonAction::AppShortcuts
            | ButtonAction::Keypad => unreachable!("handled above"),
        };
        out.mouse.push(self.at_pen(key(mouse_key, true)));
        Held::Mouse(mouse_key)
    }

    /// The pointer device keeps its own position in the compositor, so it
    /// must be moved under the pen before it clicks.
    fn at_pen(&self, ev: InputEvent) -> Vec<InputEvent> {
        vec![abs(Abs::ABS_X, self.x), abs(Abs::ABS_Y, self.y), ev]
    }

    fn delta_mm(&self, lx: i32, ly: i32) -> (f64, f64) {
        self.config.mapping.orientation.delta(
            f64::from(self.x - lx) / self.geometry.x_units_per_mm,
            f64::from(self.y - ly) / self.geometry.y_units_per_mm,
        )
    }

    /// Pan/Scroll and Scroll: the content moves with the pen.
    fn scroll(&mut self, gesture: Gesture, out: &mut Output) {
        let Some((lx, ly)) = self.pan_last.replace((self.x, self.y)) else {
            return;
        };
        let vertical_only = gesture == Gesture::Scroll;
        let speed = match gesture {
            Gesture::Scroll => self.config.gestures.scroll_speed,
            Gesture::PanZoom => self.config.gestures.pan_zoom_speed,
            _ => self.config.pan_scroll.speed,
        };
        let gain = v120_per_mm(speed);
        let sign = if self.config.pan_scroll.content_follows_pen { 1.0 } else { -1.0 };
        let (dx_mm, dy_mm) = self.delta_mm(lx, ly);
        let dx_mm = if vertical_only { 0.0 } else { dx_mm };
        // Wheel up (+) moves the view up, so content follows a pen moving
        // down; horizontal wheel right (+) needs the pen moving left.
        self.wheel_rest.0 -= dx_mm * gain * sign;
        self.wheel_rest.1 += dy_mm * gain * sign;

        let h = take_whole(&mut self.wheel_rest.0);
        let v = take_whole(&mut self.wheel_rest.1);
        self.push_wheel(h, v, out);
    }

    /// Pan/Zoom in the air: Ctrl+wheel proportional to vertical travel, pen
    /// up zooms in. Ctrl goes down with the first step, so merely holding the
    /// button does not leave a modifier pressed for long.
    fn hover_zoom(&mut self, out: &mut Output) {
        let Some((lx, ly)) = self.zoom_last.replace((self.x, self.y)) else {
            return;
        };
        let gain = v120_per_mm(self.config.gestures.pan_zoom_speed) * HOVER_ZOOM_GAIN;
        self.wheel_rest.1 -= self.delta_mm(lx, ly).1 * gain;
        let v = take_whole(&mut self.wheel_rest.1);
        if v == 0 {
            return;
        }
        if !self.ctrl_down {
            out.keyboard.push(vec![key(KeyCode::KEY_LEFTCTRL, true)]);
            self.ctrl_down = true;
        }
        self.push_wheel(0, v, out);
    }

    /// Emits hi-res wheel units (plus legacy notches) at the frozen cursor.
    fn push_wheel(&mut self, h: i32, v: i32, out: &mut Output) {
        // Scroll goes to the window under the pointer device, so park it where
        // the cursor froze (repeated values are dropped by the kernel).
        let mut frame = vec![abs(Abs::ABS_X, self.pan_anchor.0), abs(Abs::ABS_Y, self.pan_anchor.1)];
        if h != 0 {
            frame.push(rel(Rel::REL_HWHEEL_HI_RES, h));
            if let Some(n) = notches(&mut self.notch_acc.0, h) {
                frame.push(rel(Rel::REL_HWHEEL, n));
            }
        }
        if v != 0 {
            frame.push(rel(Rel::REL_WHEEL_HI_RES, v));
            if let Some(n) = notches(&mut self.notch_acc.1, v) {
                frame.push(rel(Rel::REL_WHEEL, n));
            }
        }
        if frame.len() > 2 {
            out.mouse.push(frame);
        }
    }

    /// Pen travel, in counts per axis, for one unit of joystick speed.
    fn stick_counts(&self, gesture: Gesture) -> (f64, f64) {
        let g = &self.config.gestures;
        let (speed, travel) = match gesture {
            Gesture::Wheel => (g.wheel_speed, 1.0),
            Gesture::Zoom => (g.zoom_speed, ZOOM_TRAVEL),
            _ => (g.legacy_speed, 1.0),
        };
        let mm = joystick_mm(speed) * travel;
        if self.config.mapping.orientation.is_portrait() {
            (mm * self.geometry.y_units_per_mm, mm * self.geometry.x_units_per_mm)
        } else {
            (mm * self.geometry.x_units_per_mm, mm * self.geometry.y_units_per_mm)
        }
    }

    /// Joystick gestures: accumulate travel since touch-down, one axis at a
    /// time, ignoring small reversals (as `SetSpeedEvent` does on macOS).
    fn stick_move(&mut self, gesture: Gesture) {
        let counts = self.stick_counts(gesture);
        let pos = (self.x, self.y);
        let st = self.stick.get_or_insert(Stick { last: pos, ..Stick::default() });
        let d = self.config.mapping.orientation.delta(f64::from(pos.0 - st.last.0), f64::from(pos.1 - st.last.1));
        st.last = pos;
        if d == (0.0, 0.0) {
            return;
        }
        let mut acc = st.acc;
        for (a, d, c) in [(&mut acc.0, d.0, counts.0), (&mut acc.1, d.1, counts.1)] {
            if d != 0.0 && *a != 0.0 && a.signum() != d.signum() {
                if d.abs() <= DEADZONE * c {
                    return; // tremor: drop the whole packet
                }
                *a = d;
            } else {
                *a += d;
            }
        }
        if gesture == Gesture::Zoom || d.0.abs() * AXIS_DOMINANCE <= d.1.abs() {
            acc.0 = 0.0;
        } else {
            acc.1 = 0.0;
        }
        st.acc = acc;
    }

    /// True while a joystick gesture needs [`Self::tick`] to keep going,
    /// or a held button waits to turn into a middle click.
    pub fn wants_ticks(&self) -> bool {
        self.stick.is_some() || [self.upper, self.lower].iter().any(|h| matches!(h, Held::Pending { .. }))
    }

    /// How long a right-or-middle button waits to become a middle click.
    pub fn hold_time(&self) -> Duration {
        hold_middle_time(self.config.pen.hold_time)
    }

    /// Whether a right-or-middle button is held and still undecided.
    pub fn hold_pending(&self) -> bool {
        [self.upper, self.lower].iter().any(|h| matches!(h, Held::Pending { .. }))
    }

    /// Turns a right-or-middle button held long enough, or moved with,
    /// into a middle button press where it was pressed.
    fn promote_holds(&mut self, out: &mut Output) {
        let travel = HOLD_MIDDLE_MM * self.geometry.x_units_per_mm;
        let wait = self.hold_time();
        let (x, y) = (self.x, self.y);
        for held in [&mut self.upper, &mut self.lower] {
            let Held::Pending { since, at } = *held else { continue };
            let moved = f64::from(x - at.0).hypot(f64::from(y - at.1)) >= travel;
            if moved || since.elapsed() >= wait {
                out.mouse.push(vec![abs(Abs::ABS_X, at.0), abs(Abs::ABS_Y, at.1), key(KeyCode::BTN_MIDDLE, true)]);
                *held = Held::Mouse(KeyCode::BTN_MIDDLE);
            }
        }
    }

    /// Emits joystick output for `dt` seconds, even if the pen is still.
    /// `pad_gesture` as in [`Self::process_frame`], so releasing an
    /// ExpressKey stops the gesture without waiting for a pen packet.
    pub fn tick(&mut self, dt: f64, pad_gesture: Option<Gesture>) -> Output {
        let mut out = Output::default();
        self.promote_holds(&mut out);
        if pad_gesture != self.pad_gesture {
            self.set_pan(|s| s.pad_gesture = pad_gesture);
            self.release_ctrl_unless_zooming(&mut out);
        }
        let (Some(gesture), Some(mut st)) = (self.gesture(), self.stick) else {
            return out;
        };
        let counts = self.stick_counts(gesture);
        // Speed units: how many speed settings away from touch-down.
        let units = (st.acc.0 / counts.0, st.acc.1 / counts.1);
        // Pen right/down scrolls right/down (hwheel +, wheel -); for Zoom,
        // pen up zooms in (Ctrl + wheel up).
        let (rate, scale) = match gesture {
            Gesture::Wheel => (WHEEL_NOTCHES_PER_SEC, WHEEL_NOTCH),
            Gesture::Zoom => (ZOOM_NOTCHES_PER_SEC, WHEEL_NOTCH),
            _ => (PACKETS_PER_SEC * LEGACY_V120_PER_STEP, 1),
        };
        st.rest.0 += units.0 * rate * dt;
        st.rest.1 -= units.1 * rate * dt;
        let h = take_whole(&mut st.rest.0) * scale;
        let v = take_whole(&mut st.rest.1) * scale;
        self.stick = Some(st);
        if (h, v) == (0, 0) {
            return out;
        }
        if gesture == Gesture::Zoom && !self.ctrl_down {
            out.keyboard.push(vec![key(KeyCode::KEY_LEFTCTRL, true)]);
            self.ctrl_down = true;
        }
        self.push_wheel(h, v, &mut out);
        out
    }

    fn release_ctrl_unless_zooming(&mut self, out: &mut Output) {
        let zooming = match self.gesture() {
            Some(Gesture::Zoom) => self.tip,
            Some(Gesture::PanZoom) => !self.tip && self.in_proximity,
            _ => false,
        };
        if self.ctrl_down && !zooming {
            out.keyboard.push(vec![key(KeyCode::KEY_LEFTCTRL, false)]);
            self.ctrl_down = false;
        }
    }

    fn curved_pressure(&self) -> i32 {
        let max = f64::from(self.geometry.pressure_max.max(1));
        let p = (f64::from(self.pressure) / max).clamp(0.0, 1.0);
        (p.powf(self.config.pen.pressure_gamma()) * max).round() as i32
    }
}

fn take_whole(rest: &mut f64) -> i32 {
    let whole = rest.trunc();
    *rest -= whole;
    whole as i32
}

/// Legacy REL_WHEEL clicks for clients that ignore hi-res scrolling.
fn notches(acc: &mut i32, delta: i32) -> Option<i32> {
    if acc.signum() * delta.signum() < 0 {
        *acc = 0;
    }
    *acc += delta;
    let n = *acc / WHEEL_NOTCH;
    *acc %= WHEEL_NOTCH;
    (n != 0).then_some(n)
}

fn key(code: KeyCode, down: bool) -> InputEvent {
    InputEvent::new(EventType::KEY.0, code.0, i32::from(down))
}

fn abs(code: Abs, value: i32) -> InputEvent {
    InputEvent::new(EventType::ABSOLUTE.0, code.0, value)
}

fn rel(code: Rel, value: i32) -> InputEvent {
    InputEvent::new(EventType::RELATIVE.0, code.0, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEO: Geometry = Geometry { x_units_per_mm: 100.0, y_units_per_mm: 100.0, pressure_max: 4095 };

    fn proc() -> PenProcessor {
        PenProcessor::new(Config::default(), GEO)
    }

    fn has(events: &[InputEvent], ty: EventType, code: u16) -> Option<i32> {
        events.iter().find(|e| e.event_type() == ty && e.code() == code).map(|e| e.value())
    }

    fn pos(x: i32, y: i32) -> Vec<InputEvent> {
        vec![abs(Abs::ABS_X, x), abs(Abs::ABS_Y, y)]
    }

    fn right_or_middle() -> PenProcessor {
        let mut config = Config::default();
        config.pen.lower_button = ButtonAction::RightOrHoldMiddle;
        let mut p = PenProcessor::new(config, GEO);
        p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, true), abs(Abs::ABS_X, 500), abs(Abs::ABS_Y, 500)], None);
        p
    }

    fn mouse_keys(out: &Output) -> Vec<(u16, i32)> {
        out.mouse.iter().flatten().filter(|e| e.event_type() == EventType::KEY).map(|e| (e.code(), e.value())).collect()
    }

    #[test]
    fn tap_is_a_right_click_on_release() {
        let mut p = right_or_middle();
        let down = p.process_frame(&[key(KeyCode::BTN_STYLUS, true)], None);
        assert!(down.mouse.is_empty(), "undecided until release");
        assert!(p.wants_ticks());
        let up = p.process_frame(&[key(KeyCode::BTN_STYLUS, false)], None);
        assert_eq!(mouse_keys(&up), vec![(KeyCode::BTN_RIGHT.0, 1), (KeyCode::BTN_RIGHT.0, 0)]);
        assert!(!p.wants_ticks());
    }

    #[test]
    fn moving_while_held_drags_with_the_middle_button() {
        let mut p = right_or_middle();
        p.process_frame(&[key(KeyCode::BTN_STYLUS, true)], None);
        assert!(p.process_frame(&pos(550, 500), None).mouse.is_empty(), "0.5 mm is hand jitter");
        let moved = p.process_frame(&pos(900, 500), None);
        assert_eq!(moved.mouse, vec![vec![abs(Abs::ABS_X, 500), abs(Abs::ABS_Y, 500), key(KeyCode::BTN_MIDDLE, true)]]);
        assert!(p.state().mouse_held);
        let up = p.process_frame(&[key(KeyCode::BTN_STYLUS, false)], None);
        assert_eq!(mouse_keys(&up), vec![(KeyCode::BTN_MIDDLE.0, 0)]);
        assert!(!p.state().mouse_held);
    }

    #[test]
    fn holding_still_turns_into_a_middle_press() {
        let mut p = right_or_middle();
        p.process_frame(&[key(KeyCode::BTN_STYLUS, true)], None);
        std::thread::sleep(p.hold_time());
        assert_eq!(mouse_keys(&p.tick(0.008, None)), vec![(KeyCode::BTN_MIDDLE.0, 1)]);
    }

    #[test]
    fn with_the_tip_down_it_is_a_plain_right_click() {
        let mut p = right_or_middle();
        p.process_frame(&[key(KeyCode::BTN_TOUCH, true)], None);
        let down = p.process_frame(&[key(KeyCode::BTN_STYLUS, true)], None);
        assert_eq!(mouse_keys(&down), vec![(KeyCode::BTN_RIGHT.0, 1)]);
    }

    fn wheel(out: &Output) -> (i32, i32) {
        let all: Vec<_> = out.mouse.iter().flatten().copied().collect();
        (
            has(&all, EventType::RELATIVE, Rel::REL_HWHEEL_HI_RES.0).unwrap_or(0),
            has(&all, EventType::RELATIVE, Rel::REL_WHEEL_HI_RES.0).unwrap_or(0),
        )
    }

    #[test]
    fn movement_passes_through_when_idle() {
        let mut p = proc();
        let out = p.process_frame(&pos(500, 600), None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), Some(500));
        assert!(out.mouse.is_empty());
    }

    #[test]
    fn pan_freezes_cursor_and_scrolls_both_axes() {
        let mut p = proc();
        p.process_frame(&pos(1000, 1000), None);
        p.process_frame(&[key(KeyCode::BTN_STYLUS2, true)], None);

        // Hovering with the button held: cursor frozen, nothing scrolls.
        let out = p.process_frame(&pos(1500, 1500), None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), None);
        assert!(out.mouse.is_empty());

        // Tip down, then drag 10 mm right and 10 mm down.
        let mut down = pos(1500, 1500);
        down.extend([key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 2000)]);
        let out = p.process_frame(&down, None);
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOUCH.0), None, "tip must not click");
        let out = p.process_frame(&pos(2500, 2500), None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), None);
        assert_eq!(wheel(&out), (-1100, 1100), "content follows pen at speed 3");
    }

    #[test]
    fn releasing_pan_keeps_tip_up_until_lift() {
        let mut p = proc();
        p.process_frame(&[key(KeyCode::BTN_STYLUS2, true)], None);
        let mut down = pos(0, 0);
        down.extend([key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 2000)]);
        p.process_frame(&down, None);

        let out = p.process_frame(&[key(KeyCode::BTN_STYLUS2, false)], None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), Some(0), "cursor resyncs");
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOUCH.0), None);

        let out = p.process_frame(&pos(300, 300), None);
        assert!(out.mouse.is_empty(), "no momentum after release");
        p.process_frame(&[key(KeyCode::BTN_TOUCH, false), abs(Abs::ABS_PRESSURE, 0)], None);
        let out = p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 2000)], None);
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOUCH.0), Some(1));
    }

    #[test]
    fn lower_button_right_clicks_on_mouse() {
        let mut p = proc();
        p.process_frame(&pos(700, 800), None);
        let out = p.process_frame(&[key(KeyCode::BTN_STYLUS, true)], None);
        assert_eq!(
            out.mouse,
            vec![vec![abs(Abs::ABS_X, 700), abs(Abs::ABS_Y, 800), key(KeyCode::BTN_RIGHT, true)]],
            "pointer moves under the pen before clicking"
        );
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_STYLUS.0), None);
        let out = p.process_frame(&[key(KeyCode::BTN_STYLUS, false)], None);
        assert_eq!(out.mouse, vec![vec![key(KeyCode::BTN_RIGHT, false)]]);
    }

    #[test]
    fn soft_sensitivity_raises_pressure() {
        let mut config = Config::default();
        config.pen.tip_sensitivity = 1;
        let mut p = PenProcessor::new(config, GEO);
        let out = p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 1000)], None);
        let sent = has(&out.pen, EventType::ABSOLUTE, Abs::ABS_PRESSURE.0).unwrap();
        assert!(sent > 1000, "soft curve should increase pressure, got {sent}");
    }

    #[test]
    fn expresskey_pan_scrolls_like_the_side_button() {
        let mut p = proc();
        p.process_frame(&pos(1000, 1000), None);
        let mut down = pos(1000, 1000);
        down.extend([key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 2000)]);
        p.process_frame(&down, Some(Gesture::PanScroll));
        let out = p.process_frame(&pos(1000, 2000), Some(Gesture::PanScroll));
        assert_eq!(wheel(&out).1, 1100);
        let out = p.process_frame(&[], None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), Some(1000), "cursor resyncs");
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOUCH.0), None, "tip stays up until lifted");
    }

    #[test]
    fn legacy_notch_every_120_units() {
        let mut acc = 0;
        assert_eq!(notches(&mut acc, 100), None);
        assert_eq!(notches(&mut acc, 30), Some(1));
        assert_eq!(notches(&mut acc, -50), None, "direction change resets");
    }

    fn legacy() -> PenProcessor {
        let mut config = Config::default();
        config.pen.upper_button = ButtonAction::PanScrollLegacy;
        PenProcessor::new(config, GEO)
    }

    fn wheel_v(out: &Output) -> i32 {
        out.mouse.iter().flatten()
            .filter(|e| e.event_type() == EventType::RELATIVE && e.code() == Rel::REL_WHEEL_HI_RES.0)
            .map(|e| e.value()).sum()
    }

    fn touch_with_button(p: &mut PenProcessor) {
        p.process_frame(&pos(1000, 1000), None);
        p.process_frame(&[key(KeyCode::BTN_STYLUS2, true)], None);
        p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 900)], None);
    }

    #[test]
    fn user_up_scrolls_the_same_when_flipped() {
        let mut values = Vec::new();
        for (orientation, y) in [
            (tablet_config::Orientation::Landscape, 500),
            (tablet_config::Orientation::LandscapeFlipped, 1500),
        ] {
            let mut p = proc();
            p.config.mapping.orientation = orientation;
            p.config.pen.upper_button = ButtonAction::PanScroll;
            touch_with_button(&mut p);
            values.push(wheel_v(&p.process_frame(&pos(1000, y), None)));
        }
        assert_ne!(values[0], 0);
        assert_eq!(values[0], values[1]);
    }

    #[test]
    fn gestures_follow_user_up_in_every_orientation() {
        use tablet_config::Orientation;
        for gesture in [
            Gesture::PanScroll, Gesture::Scroll, Gesture::PanZoom,
            Gesture::PanScrollLegacy, Gesture::Wheel, Gesture::Zoom,
        ] {
            let mut expected = None;
            for (orientation, end) in [
                (Orientation::Landscape, (1000, 0)),
                (Orientation::Portrait, (500, 1000)),
                (Orientation::LandscapeFlipped, (1000, 2000)),
                (Orientation::PortraitFlipped, (1500, 1000)),
            ] {
                let mut config = Config::default();
                config.mapping.orientation = orientation;
                let geometry = Geometry { y_units_per_mm: 200.0, ..GEO };
                let mut p = PenProcessor::new(config, geometry);
                p.process_frame(&pos(1000, 1000), Some(gesture));
                p.process_frame(&[key(KeyCode::BTN_TOUCH, true)], Some(gesture));
                let moved = p.process_frame(&pos(end.0, end.1), Some(gesture));
                let output = match gesture {
                    Gesture::PanScroll | Gesture::Scroll | Gesture::PanZoom => moved,
                    _ => p.tick(1.0, Some(gesture)),
                };
                let actual = wheel(&output);
                assert_eq!(actual.0, 0, "{gesture:?} {orientation:?}");
                assert_ne!(actual.1, 0, "{gesture:?} {orientation:?}");
                assert_eq!(actual, *expected.get_or_insert(actual), "{gesture:?} {orientation:?}");
            }
        }
    }

    #[test]
    fn hover_zoom_follows_user_up_in_every_orientation() {
        use tablet_config::Orientation;
        for (orientation, end) in [
            (Orientation::Landscape, (1000, 500)),
            (Orientation::Portrait, (500, 1000)),
            (Orientation::LandscapeFlipped, (1000, 1500)),
            (Orientation::PortraitFlipped, (1500, 1000)),
        ] {
            let mut p = pan_zoom();
            p.config.mapping.orientation = orientation;
            let out = p.process_frame(&pos(end.0, end.1), None);
            assert_eq!(wheel(&out), (0, 137), "{orientation:?}");
        }
    }

    #[test]
    fn legacy_keeps_scrolling_while_the_pen_rests() {
        let mut p = legacy();
        touch_with_button(&mut p);
        p.process_frame(&pos(1000, 1600), None); // 6 mm down = 2 speed units
        let first = wheel_v(&p.tick(0.1, None));
        let second = wheel_v(&p.tick(0.1, None));
        assert!(first < 0, "pen down scrolls down");
        assert!((first - second).abs() <= 1, "constant rate with the pen still");
    }

    #[test]
    fn legacy_ignores_tremor_reversals() {
        let mut p = legacy();
        touch_with_button(&mut p);
        p.process_frame(&pos(1000, 1600), None);
        p.process_frame(&pos(1000, 1590), None); // 0.1 mm back, inside the deadzone
        assert!(wheel_v(&p.tick(0.1, None)) < 0);
    }

    #[test]
    fn legacy_stops_when_the_tip_lifts() {
        let mut p = legacy();
        touch_with_button(&mut p);
        p.process_frame(&pos(1000, 1600), None);
        p.process_frame(&[key(KeyCode::BTN_TOUCH, false), abs(Abs::ABS_PRESSURE, 0)], None);
        assert!(!p.wants_ticks());
        assert!(p.tick(0.1, None).mouse.is_empty());
    }

    #[test]
    fn scroll_is_vertical_only() {
        let mut config = Config::default();
        config.pen.upper_button = ButtonAction::Scroll;
        let mut p = PenProcessor::new(config, GEO);
        touch_with_button(&mut p);
        let out = p.process_frame(&pos(3000, 1000), None);
        assert!(out.mouse.iter().flatten().all(|e| e.code() != Rel::REL_HWHEEL_HI_RES.0));
    }

    #[test]
    fn zoom_holds_ctrl_until_the_tip_lifts() {
        let mut config = Config::default();
        config.pen.upper_button = ButtonAction::Zoom;
        let mut p = PenProcessor::new(config, GEO);
        touch_with_button(&mut p);
        p.process_frame(&pos(1000, 400), None); // pen up = zoom in
        let out = p.tick(1.0, None);
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTCTRL, true)]]);
        assert!(wheel_v(&out) > 0);
        let out = p.process_frame(&[key(KeyCode::BTN_TOUCH, false), abs(Abs::ABS_PRESSURE, 0)], None);
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTCTRL, false)]]);
    }

    fn pan_zoom() -> PenProcessor {
        let mut config = Config::default();
        config.pen.upper_button = ButtonAction::PanZoom;
        let mut p = PenProcessor::new(config, GEO);
        p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, true), abs(Abs::ABS_X, 1000), abs(Abs::ABS_Y, 1000)], None);
        p.process_frame(&[key(KeyCode::BTN_STYLUS2, true)], None);
        p
    }

    #[test]
    fn pan_zoom_zooms_in_the_air() {
        let mut p = pan_zoom();
        p.process_frame(&pos(1000, 1000), None);
        let out = p.process_frame(&pos(1000, 600), None); // pen up = zoom in
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTCTRL, true)]]);
        assert!(wheel_v(&out) > 0);
        let out = p.process_frame(&[key(KeyCode::BTN_STYLUS2, false)], None);
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTCTRL, false)]]);
    }

    #[test]
    fn pan_zoom_pans_while_touching() {
        let mut p = pan_zoom();
        p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 900)], None);
        let out = p.process_frame(&pos(1600, 1600), None);
        assert!(out.keyboard.is_empty());
        let (h, v) = wheel(&out);
        assert!(h != 0 && v != 0);
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOUCH.0), None);
    }

    fn mouse_mode() -> PenProcessor {
        let mut config = Config::default();
        config.pen.mode = PenMode::Mouse;
        PenProcessor::new(config, GEO)
    }

    #[test]
    fn mouse_mode_moves_the_pointer_and_clicks_with_the_tip() {
        let mut p = mouse_mode();
        let out = p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, true), abs(Abs::ABS_X, 100), abs(Abs::ABS_Y, 200)], None);
        assert!(out.pen.is_empty(), "the tablet cursor stays away");
        assert_eq!(out.mouse, vec![vec![abs(Abs::ABS_X, 100), abs(Abs::ABS_Y, 200)]]);
        let out = p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 900)], None);
        assert_eq!(out.mouse, vec![vec![key(KeyCode::BTN_LEFT, true)]]);
    }

    #[test]
    fn switching_modes_moves_proximity_between_devices() {
        let mut p = proc();
        p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, true), abs(Abs::ABS_X, 100), abs(Abs::ABS_Y, 200)], None);
        let mut config = Config::default();
        config.pen.mode = PenMode::Mouse;
        p.set_config(config);
        let out = p.process_frame(&pos(110, 200), None);
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOOL_PEN.0), Some(0));
        p.set_config(Config::default());
        let out = p.process_frame(&pos(120, 200), None);
        assert_eq!(has(&out.pen, EventType::KEY, KeyCode::BTN_TOOL_PEN.0), Some(1));
    }

    #[test]
    fn leaving_keeps_the_last_position() {
        let mut p = proc();
        p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, true), abs(Abs::ABS_X, 500), abs(Abs::ABS_Y, 600)], None);
        let out = p.process_frame(&[key(KeyCode::BTN_TOOL_PEN, false), abs(Abs::ABS_X, 0), abs(Abs::ABS_Y, 0)], None);
        assert_eq!(has(&out.pen, EventType::ABSOLUTE, Abs::ABS_X.0), None);
        assert_eq!((p.x, p.y), (500, 600));
    }

    #[test]
    fn releasing_the_expresskey_stops_a_resting_joystick() {
        let mut p = proc();
        p.process_frame(&pos(1000, 1000), Some(Gesture::Wheel));
        p.process_frame(&[key(KeyCode::BTN_TOUCH, true), abs(Abs::ABS_PRESSURE, 900)], Some(Gesture::Wheel));
        p.process_frame(&pos(1000, 2000), Some(Gesture::Wheel));
        assert!(p.wants_ticks());
        p.tick(0.1, None);
        assert!(!p.wants_ticks());
    }
}
