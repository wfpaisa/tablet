//! ExpressKeys: turns pad button frames into keyboard and pointer frames.
//! No I/O here; `main.rs` owns the devices.

use evdev::{AbsoluteAxisCode as Abs, EventType, InputEvent, KeyCode};
use std::str::FromStr;
use tablet_config::{Click, Gesture, KeyAction, Shortcut};

#[derive(Debug, Default)]
pub struct PadOutput {
    pub run: Option<String>,
    pub on_screen_keyboard: bool,
    pub toggle_mode: bool,
    pub keyboard: Vec<Vec<InputEvent>>,
    pub keyboard_after: Vec<Vec<InputEvent>>,
    pub pointer: Vec<Vec<InputEvent>>,
    /// A key assigned to Display toggle was pressed.
    pub toggle_display: bool,
    /// A key assigned to an on-screen shortcut was pressed.
    pub shortcut: Option<Shortcut>,
}

/// What a held ExpressKey is doing, so its release undoes exactly that.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Held {
    #[default]
    Nothing,
    Keys(Vec<KeyCode>),
    Mouse(KeyCode),
    Gesture(Gesture),
    Modifier(Vec<KeyCode>, Option<KeyCode>),
    Pressure,
    Precision,
}

pub struct PadProcessor {
    actions: Vec<KeyAction>,
    held: Vec<Held>,
    locks: Vec<Option<KeyCode>>,
    down: Vec<bool>,
}

impl PadProcessor {
    pub fn new(actions: Vec<KeyAction>) -> Self {
        Self { held: vec![Held::Nothing; actions.len().max(10)], locks: vec![None; actions.len().max(10)], down: vec![false; actions.len().max(10)], actions }
    }

    pub fn set_actions(&mut self, actions: Vec<KeyAction>) {
        self.actions = actions;
    }

    pub fn effects(&self) -> u8 {
        u8::from(self.held.iter().any(|h| matches!(h, Held::Pressure)))
            | (u8::from(self.held.iter().any(|h| matches!(h, Held::Precision))) << 1)
    }

    /// The pen gesture of a held key, if any.
    pub fn gesture(&self) -> Option<Gesture> {
        self.held.iter().find_map(|h| match h {
            Held::Gesture(g) => Some(*g),
            _ => None,
        })
    }

    /// `pen_pos` is where the pointer must be for clicks (already mapped).
    pub fn process_frame(&mut self, frame: &[InputEvent], pen_pos: (i32, i32)) -> PadOutput {
        let mut out = PadOutput::default();
        for ev in frame.iter().filter(|e| e.event_type() == EventType::KEY) {
            let Some(index) = ev.code().checked_sub(KeyCode::BTN_0.0).map(usize::from) else { continue };
            if index >= self.held.len() {
                continue;
            }
            if ev.value() == 2 || self.down[index] == (ev.value() != 0) { continue; }
            self.down[index] = ev.value() != 0;
            if ev.value() != 0 {
                let action = self.actions.get(index).cloned().unwrap_or_default();
                if let Some(code) = self.locks[index].take() {
                    out.pointer.push(vec![key(code, false)]);
                } else if let Some(code) = lock_key(&action) {
                    let down = self.locks[index].is_none();
                    out.pointer.push(vec![key(code, down)]);
                    self.locks[index] = down.then_some(code);
                } else {
                    self.held[index] = press(&action, pen_pos, &mut out);
                }
            } else {
                release(std::mem::take(&mut self.held[index]), &mut out);
            }
        }
        out
    }

    /// Releases everything still held (device going away).
    pub fn release_all(&mut self) -> PadOutput {
        let mut out = PadOutput::default();
        for held in &mut self.held {
            release(std::mem::take(held), &mut out);
        }
        for lock in &mut self.locks {
            if let Some(code) = lock.take() { out.pointer.push(vec![key(code, false)]); }
        }
        self.down.fill(false);
        out
    }
}

pub fn lock_key(action: &KeyAction) -> Option<KeyCode> {
    Some(match action {
        KeyAction::ClickLock => KeyCode::BTN_LEFT,
        KeyAction::MiddleClickLock => KeyCode::BTN_MIDDLE,
        KeyAction::RightClickLock => KeyCode::BTN_RIGHT,
        _ => return None,
    })
}

/// Presses and releases `action` once, as when it is picked from an
/// on-screen shortcut. Gestures need a held key, so they do nothing.
pub fn tap(action: &KeyAction, pen_pos: (i32, i32)) -> PadOutput {
    let mut out = PadOutput::default();
    let held = press(action, pen_pos, &mut out);
    release(held, &mut out);
    out
}

pub fn press(action: &KeyAction, (x, y): (i32, i32), out: &mut PadOutput) -> Held {
    let click = |code: KeyCode| vec![abs(Abs::ABS_X, x), abs(Abs::ABS_Y, y), key(code, true)];
    if let Some(gesture) = action.gesture() {
        return Held::Gesture(gesture);
    }
    if let Some(shortcut) = action.shortcut() {
        out.shortcut = Some(shortcut);
        return Held::Nothing;
    }
    let mouse = match action {
        KeyAction::PressureHold => return Held::Pressure,
        KeyAction::PrecisionMode => return Held::Precision,
        KeyAction::ModeToggle => { out.toggle_mode = true; return Held::Nothing; }
        KeyAction::Run { command, .. } => { out.run = Some(command.clone()); return Held::Nothing; }
        KeyAction::OnScreenKeyboard => { out.on_screen_keyboard = true; return Held::Nothing; }
        KeyAction::Modifier { shift, ctrl, alt, super_key, click: button } => {
            let codes: Vec<_> = [(*shift, KeyCode::KEY_LEFTSHIFT), (*ctrl, KeyCode::KEY_LEFTCTRL),
                (*alt, KeyCode::KEY_LEFTALT), (*super_key, KeyCode::KEY_LEFTMETA)]
                .into_iter().filter_map(|(on, code)| on.then_some(code)).collect();
            if !codes.is_empty() { out.keyboard.push(codes.iter().map(|&k| key(k, true)).collect()); }
            let button = button.map(|c| match c { Click::Left => KeyCode::BTN_LEFT, Click::Right => KeyCode::BTN_RIGHT, Click::Middle => KeyCode::BTN_MIDDLE });
            if let Some(code) = button { out.pointer.push(click(code)); }
            return Held::Modifier(codes, button);
        }
        KeyAction::ShowDesktop | KeyAction::Overview | KeyAction::NotificationCenter | KeyAction::SwitchApplication => {
            let codes = match action {
                KeyAction::ShowDesktop => vec![KeyCode::KEY_LEFTMETA, KeyCode::KEY_D],
                KeyAction::Overview => vec![KeyCode::KEY_LEFTMETA],
                KeyAction::NotificationCenter => vec![KeyCode::KEY_LEFTMETA, KeyCode::KEY_V],
                _ => vec![KeyCode::KEY_LEFTALT, KeyCode::KEY_TAB],
            };
            out.keyboard.push(codes.iter().map(|&k| key(k, true)).collect());
            if matches!(action, KeyAction::SwitchApplication) { return Held::Keys(codes); }
            out.keyboard.push(codes.iter().rev().map(|&k| key(k, false)).collect());
            return Held::Nothing;
        }
        KeyAction::ClickLock | KeyAction::MiddleClickLock | KeyAction::RightClickLock => return Held::Nothing,
        KeyAction::Disabled => return Held::Nothing,
        KeyAction::DisplayToggle => {
            out.toggle_display = true;
            return Held::Nothing;
        }
        KeyAction::Keystroke { keys, .. } => {
            let codes: Vec<KeyCode> = keys.iter().filter_map(|k| KeyCode::from_str(k).ok()).collect();
            if codes.is_empty() {
                return Held::Nothing;
            }
            out.keyboard.push(codes.iter().map(|&k| key(k, true)).collect());
            return Held::Keys(codes);
        }
        KeyAction::DoubleClick => {
            out.pointer.push(click(KeyCode::BTN_LEFT));
            for pressed in [false, true, false] {
                out.pointer.push(vec![key(KeyCode::BTN_LEFT, pressed)]);
            }
            return Held::Nothing;
        }
        KeyAction::LeftClick => KeyCode::BTN_LEFT,
        KeyAction::RightClick => KeyCode::BTN_RIGHT,
        KeyAction::MiddleClick => KeyCode::BTN_MIDDLE,
        KeyAction::Back => KeyCode::BTN_SIDE,
        KeyAction::Forward => KeyCode::BTN_EXTRA,
        KeyAction::PanScroll
        | KeyAction::PanScrollLegacy
        | KeyAction::Scroll
        | KeyAction::Wheel
        | KeyAction::Zoom
        | KeyAction::PanZoom
        | KeyAction::ExpressMenu
        | KeyAction::RadialMenu
        | KeyAction::AppShortcuts
        | KeyAction::Keypad => unreachable!("handled above"),
    };
    out.pointer.push(click(mouse));
    Held::Mouse(mouse)
}

pub fn release(held: Held, out: &mut PadOutput) {
    match held {
        Held::Keys(codes) => out.keyboard.push(codes.iter().rev().map(|&k| key(k, false)).collect()),
        Held::Mouse(code) => out.pointer.push(vec![key(code, false)]),
        Held::Modifier(codes, button) => {
            if let Some(code) = button { out.pointer.push(vec![key(code, false)]); }
            if !codes.is_empty() { out.keyboard_after.push(codes.iter().rev().map(|&k| key(k, false)).collect()); }
        }
        Held::Nothing | Held::Gesture(_) | Held::Pressure | Held::Precision => {}
    }
}

/// Drops pad buttons below `first` (e.g. a touch ring's centre button) and
/// shifts the rest down so ExpressKey 1 always arrives as BTN_0.
pub fn renumber(frame: &mut Vec<InputEvent>, first: u16) {
    if first == 0 {
        return;
    }
    frame.retain(|e| e.event_type() != EventType::KEY || !(KeyCode::BTN_0.0..KeyCode::BTN_0.0 + first).contains(&e.code()));
    for e in frame.iter_mut() {
        if e.event_type() == EventType::KEY && (KeyCode::BTN_0.0..=KeyCode::BTN_9.0).contains(&e.code()) {
            *e = InputEvent::new(EventType::KEY.0, e.code() - first, e.value());
        }
    }
}

fn key(code: KeyCode, down: bool) -> InputEvent {
    InputEvent::new(EventType::KEY.0, code.0, i32::from(down))
}

fn abs(code: Abs, value: i32) -> InputEvent {
    InputEvent::new(EventType::ABSOLUTE.0, code.0, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(n: u16, down: bool) -> Vec<InputEvent> {
        vec![key(KeyCode(KeyCode::BTN_0.0 + n), down)]
    }

    fn undo() -> KeyAction {
        KeyAction::Keystroke { keys: vec!["KEY_LEFTCTRL".into(), "KEY_Z".into()], label: "Ctrl+Z".into() }
    }

    #[test]
    fn renumber_skips_the_ring_button_and_shifts_express_keys() {
        let mut frame = [button(0, true), button(1, true), button(6, false)].concat();
        frame.push(abs(Abs::ABS_WHEEL, 12));
        renumber(&mut frame, 1);
        let codes: Vec<_> = frame.iter().map(|e| (e.event_type(), e.code(), e.value())).collect();
        assert_eq!(
            codes,
            [
                (EventType::KEY, KeyCode::BTN_0.0, 1),
                (EventType::KEY, KeyCode::BTN_5.0, 0),
                (EventType::ABSOLUTE, Abs::ABS_WHEEL.0, 12),
            ]
        );
        let mut same = button(0, true);
        renumber(&mut same, 0);
        assert_eq!(same[0].code(), KeyCode::BTN_0.0, "other tablets keep their numbering");
    }

    #[test]
    fn click_lock_ignores_repeat_and_releases_on_second_press_or_disconnect() {
        let mut p = PadProcessor::new(vec![KeyAction::MiddleClickLock]);
        let out = p.process_frame(&button(0, true), (0, 0));
        assert_eq!(out.pointer, vec![vec![key(KeyCode::BTN_MIDDLE, true)]]);
        assert!(p.process_frame(&button(0, true), (0, 0)).pointer.is_empty());
        assert!(p.process_frame(&button(0, false), (0, 0)).pointer.is_empty());
        assert_eq!(p.process_frame(&button(0, true), (0, 0)).pointer, vec![vec![key(KeyCode::BTN_MIDDLE, false)]]);
        p.process_frame(&button(0, false), (0, 0));
        p.process_frame(&button(0, true), (0, 0));
        assert_eq!(p.release_all().pointer, vec![vec![key(KeyCode::BTN_MIDDLE, false)]]);
    }

    #[test]
    fn modifier_presses_before_click_and_releases_after_click_in_reverse_order() {
        let mut p = PadProcessor::new(vec![KeyAction::Modifier { shift: true, ctrl: true, alt: false, super_key: false, click: Some(Click::Right) }]);
        let out = p.process_frame(&button(0, true), (10, 20));
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTSHIFT, true), key(KeyCode::KEY_LEFTCTRL, true)]]);
        assert!(out.keyboard_after.is_empty());
        assert_eq!(out.pointer[0].last(), Some(&key(KeyCode::BTN_RIGHT, true)));
        p.set_actions(vec![KeyAction::Disabled]);
        let out = p.process_frame(&button(0, false), (10, 20));
        assert!(out.keyboard.is_empty());
        assert_eq!(out.pointer, vec![vec![key(KeyCode::BTN_RIGHT, false)]]);
        assert_eq!(out.keyboard_after, vec![vec![key(KeyCode::KEY_LEFTCTRL, false), key(KeyCode::KEY_LEFTSHIFT, false)]]);
    }

    #[test]
    fn keystroke_presses_in_order_and_releases_in_reverse() {
        let mut p = PadProcessor::new(vec![undo(), KeyAction::Disabled]);
        let out = p.process_frame(&button(0, true), (0, 0));
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_LEFTCTRL, true), key(KeyCode::KEY_Z, true)]]);
        let out = p.process_frame(&button(0, false), (0, 0));
        assert_eq!(out.keyboard, vec![vec![key(KeyCode::KEY_Z, false), key(KeyCode::KEY_LEFTCTRL, false)]]);
    }

    #[test]
    fn click_happens_under_the_pen() {
        let mut p = PadProcessor::new(vec![KeyAction::Disabled, KeyAction::RightClick]);
        let out = p.process_frame(&button(1, true), (40, 50));
        assert_eq!(out.pointer, vec![vec![abs(Abs::ABS_X, 40), abs(Abs::ABS_Y, 50), key(KeyCode::BTN_RIGHT, true)]]);
    }

    #[test]
    fn pan_is_reported_while_held() {
        let mut p = PadProcessor::new(vec![KeyAction::Disabled, KeyAction::Disabled, KeyAction::PanScroll]);
        p.process_frame(&button(2, true), (0, 0));
        assert_eq!(p.gesture(), Some(Gesture::PanScroll));
        p.process_frame(&button(2, false), (0, 0));
        assert_eq!(p.gesture(), None);
    }

    #[test]
    fn release_survives_a_settings_change() {
        let mut p = PadProcessor::new(vec![undo()]);
        p.process_frame(&button(0, true), (0, 0));
        p.set_actions(vec![KeyAction::Disabled]);
        let out = p.process_frame(&button(0, false), (0, 0));
        assert_eq!(out.keyboard.len(), 1, "Ctrl+Z still gets released");
    }

    #[test]
    fn display_toggle_fires_once_per_press() {
        let mut p = PadProcessor::new(vec![KeyAction::DisplayToggle]);
        assert!(p.process_frame(&button(0, true), (0, 0)).toggle_display);
        assert!(!p.process_frame(&button(0, false), (0, 0)).toggle_display);
    }

    #[test]
    fn tap_presses_and_releases() {
        let out = tap(&undo(), (0, 0));
        assert_eq!(out.keyboard.len(), 2);
        let out = tap(&KeyAction::LeftClick, (5, 6));
        assert_eq!(out.pointer.len(), 2);
    }

    #[test]
    fn shortcut_keys_open_their_menu() {
        let mut p = PadProcessor::new(vec![KeyAction::RadialMenu]);
        assert_eq!(p.process_frame(&button(0, true), (0, 0)).shortcut, Some(Shortcut::RadialMenu));
    }

    #[test]
    fn release_all_lets_go_of_held_keys() {
        let mut p = PadProcessor::new(vec![undo()]);
        p.process_frame(&button(0, true), (0, 0));
        assert_eq!(p.release_all().keyboard.len(), 1);
    }
}
