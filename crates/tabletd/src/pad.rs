//! ExpressKeys: turns pad button frames into keyboard and pointer frames.
//! No I/O here; `main.rs` owns the devices.

use evdev::{AbsoluteAxisCode as Abs, EventType, InputEvent, KeyCode};
use std::str::FromStr;
use tablet_config::{Gesture, KeyAction, Shortcut};

#[derive(Debug, Default)]
pub struct PadOutput {
    pub keyboard: Vec<Vec<InputEvent>>,
    pub pointer: Vec<Vec<InputEvent>>,
    /// A key assigned to Display toggle was pressed.
    pub toggle_display: bool,
    /// A key assigned to an on-screen shortcut was pressed.
    pub shortcut: Option<Shortcut>,
}

/// What a held ExpressKey is doing, so its release undoes exactly that.
#[derive(Debug, Clone, Default, PartialEq)]
enum Held {
    #[default]
    Nothing,
    Keys(Vec<KeyCode>),
    Mouse(KeyCode),
    Gesture(Gesture),
}

pub struct PadProcessor {
    actions: Vec<KeyAction>,
    held: Vec<Held>,
}

impl PadProcessor {
    pub fn new(actions: Vec<KeyAction>) -> Self {
        Self { held: vec![Held::Nothing; actions.len().max(10)], actions }
    }

    pub fn set_actions(&mut self, actions: Vec<KeyAction>) {
        self.actions = actions;
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
            if ev.value() != 0 {
                let action = self.actions.get(index).cloned().unwrap_or_default();
                self.held[index] = press(&action, pen_pos, &mut out);
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
        out
    }
}

/// Presses and releases `action` once, as when it is picked from an
/// on-screen shortcut. Gestures need a held key, so they do nothing.
pub fn tap(action: &KeyAction, pen_pos: (i32, i32)) -> PadOutput {
    let mut out = PadOutput::default();
    let held = press(action, pen_pos, &mut out);
    release(held, &mut out);
    out
}

fn press(action: &KeyAction, (x, y): (i32, i32), out: &mut PadOutput) -> Held {
    let click = |code: KeyCode| vec![abs(Abs::ABS_X, x), abs(Abs::ABS_Y, y), key(code, true)];
    if let Some(gesture) = action.gesture() {
        return Held::Gesture(gesture);
    }
    if let Some(shortcut) = action.shortcut() {
        out.shortcut = Some(shortcut);
        return Held::Nothing;
    }
    let mouse = match action {
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

fn release(held: Held, out: &mut PadOutput) {
    match held {
        Held::Keys(codes) => out.keyboard.push(codes.iter().rev().map(|&k| key(k, false)).collect()),
        Held::Mouse(code) => out.pointer.push(vec![key(code, false)]),
        Held::Nothing | Held::Gesture(_) => {}
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
