//! Settings and IPC protocol shared by `tabletd` and Tablet Center.
//!
//! Settings live in `$XDG_CONFIG_HOME/tablet-center/config.json`. The UI writes
//! the file and then sends [`Request::Reload`] over the daemon socket at
//! `$XDG_RUNTIME_DIR/tabletd.sock` (one JSON object per line, each answered by
//! one [`Response`] line).

use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};

/// What a pen side button does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonAction {
    /// Hold the button and drag the tip to scroll in any direction.
    PanScroll,
    /// The other pen gestures; see [`Gesture`].
    PanScrollLegacy,
    Scroll,
    Wheel,
    Zoom,
    PanZoom,
    Keystroke { #[serde(default)] keys: Vec<String>, #[serde(default)] label: String },
    ClickLock,
    MiddleClickLock,
    RightClickLock,
    PressureHold,
    ModeToggle,
    Modifier {
        #[serde(default)] shift: bool,
        #[serde(default)] ctrl: bool,
        #[serde(default)] alt: bool,
        #[serde(default, rename = "super")] super_key: bool,
        #[serde(default)] click: Option<Click>,
    },
    Run {
        #[serde(default)] command: String,
        #[serde(default)] label: String,
    },
    PrecisionMode,
    ShowDesktop,
    Overview,
    SwitchApplication,
    NotificationCenter,
    OnScreenKeyboard,
    LeftClick,
    RightClick,
    MiddleClick,
    /// A tap is a right click (sent on release); holding the button still,
    /// or moving the pen while it is held, presses the middle button until
    /// release, e.g. to drag with it.
    RightOrHoldMiddle,
    DoubleClick,
    Back,
    Forward,
    /// Cycle the mapping: all displays, then each monitor in turn.
    DisplayToggle,
    /// Open an on-screen shortcut; see [`Shortcut`].
    ExpressMenu,
    RadialMenu,
    AppShortcuts,
    Keypad,
    /// Forward the raw stylus button and let the desktop decide.
    Default,
    Disabled,
}

/// How the pen reaches the desktop.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PenMode {
    /// A tablet with pressure and its own cursor, which apps may not shape.
    #[default]
    Pen,
    /// Moves the mouse pointer: one cursor that changes over text and
    /// borders, the tip clicks, no pressure.
    Mouse,
    Relative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PenSettings {
    pub tip: ButtonAction,
    pub relative: RelativeSettings,
    pub mode: PenMode,
    /// Button nearest the eraser end (BTN_STYLUS2).
    pub upper_button: ButtonAction,
    /// Button nearest the tip (BTN_STYLUS).
    pub lower_button: ButtonAction,
    /// 1 (soft) ..= 7 (firm); 4 leaves the pressure untouched.
    pub tip_sensitivity: u8,
    /// Tip double-click assist radius: 0 (off), 1, 2, 3, 4 or 6 mm.
    pub double_click_mm: u8,
    /// 1 (light: the tablet's own touch threshold, as before) ..= 7 (firm),
    /// applied before the sensitivity curve.
    pub click_threshold: u8,
    /// Right-or-middle: how long a held button waits before it becomes a
    /// middle click, 1 (short) ..= 7 (long); see [`hold_middle_time`].
    pub hold_time: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Click { Left, Right, Middle }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RelativeSettings {
    pub speed: u8,
    pub acceleration: u8,
}

impl Default for RelativeSettings {
    fn default() -> Self { Self { speed: 4, acceleration: 4 } }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrecisionSettings {
    pub scale: u8,
    pub overlay: bool,
}

impl Default for PrecisionSettings {
    fn default() -> Self { Self { scale: 2, overlay: true } }
}

/// Wait of [`ButtonAction::RightOrHoldMiddle`] for a [`PenSettings::hold_time`].
pub fn hold_middle_time(level: u8) -> std::time::Duration {
    const MS: [u64; 7] = [300, 400, 550, 700, 900, 1100, 1400];
    std::time::Duration::from_millis(MS[usize::from(level.clamp(1, 7)) - 1])
}

impl Default for PenSettings {
    fn default() -> Self {
        Self {
            tip: ButtonAction::LeftClick,
            relative: RelativeSettings::default(),
            mode: PenMode::Pen,
            upper_button: ButtonAction::PanScroll,
            lower_button: ButtonAction::RightClick,
            tip_sensitivity: 4,
            double_click_mm: 2,
            click_threshold: 1,
            hold_time: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanScrollSettings {
    /// 1 (slow) ..= 5 (fast), same scale as the macOS slider.
    pub speed: u8,
    /// Content moves with the pen, like grabbing the page.
    pub content_follows_pen: bool,
}

impl Default for PanScrollSettings {
    fn default() -> Self {
        Self { speed: 3, content_follows_pen: true }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
    LandscapeFlipped,
    PortraitFlipped,
}

impl Orientation {
    pub fn position(self, x: f64, y: f64) -> (f64, f64) {
        match self {
            Self::Landscape => (x, y),
            Self::Portrait => (1.0 - y, x),
            Self::LandscapeFlipped => (1.0 - x, 1.0 - y),
            Self::PortraitFlipped => (y, 1.0 - x),
        }
    }

    pub fn delta(self, dx: f64, dy: f64) -> (f64, f64) {
        match self {
            Self::Landscape => (dx, dy),
            Self::Portrait => (-dy, dx),
            Self::LandscapeFlipped => (-dx, -dy),
            Self::PortraitFlipped => (dy, -dx),
        }
    }

    pub fn is_portrait(self) -> bool {
        matches!(self, Self::Portrait | Self::PortraitFlipped)
    }
}

/// A normalized rectangle in the target's coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(from = "RectFields")]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Deserialize)]
struct RectFields { x: f64, y: f64, w: f64, h: f64 }

impl From<RectFields> for Rect {
    fn from(r: RectFields) -> Self {
        Self { x: r.x, y: r.y, w: r.w, h: r.h }.clamped()
    }
}

impl Rect {
    pub fn clamped(self) -> Self {
        let finite = |n: f64, fallback| if n.is_finite() { n } else { fallback };
        let w = finite(self.w, 1.0).clamp(0.05, 1.0);
        let h = finite(self.h, 1.0).clamp(0.05, 1.0);
        Self { x: finite(self.x, 0.0).clamp(0.0, 1.0 - w), y: finite(self.y, 0.0).clamp(0.0, 1.0 - h), w, h }
    }
}

/// Which part of the desktop the tablet drives.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MappingSettings {
    pub orientation: Orientation,
    /// [`Display::id`] of one monitor, or `None` for all of them (for a pen
    /// display, its own screen when GNOME reports it; see [`own_screen`]).
    pub display: Option<String>,
    /// Trim the tablet area so its aspect ratio matches the target, keeping
    /// circles round. Trims from the bottom or right, like Wacom's driver.
    pub force_proportions: bool,
    /// Fractions of the selected monitor or whole desktop.
    pub screen_portion: Option<Rect>,
    /// Fractions of the tablet after rotation into the user orientation.
    pub tablet_portion: Option<Rect>,
}

/// What sliding a finger around the touch ring does in one mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RingAction {
    #[default]
    Disabled,
    Scroll,
    Zoom,
    BrushSize,
    Rotate,
    UndoRedo,
}

/// What an ExpressKey does.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeyAction {
    #[default]
    Disabled,
    /// evdev key names pressed in order and released in reverse, e.g.
    /// `["KEY_LEFTCTRL", "KEY_Z"]`. `label` is what the UI shows.
    Keystroke { keys: Vec<String>, label: String },
    ClickLock,
    MiddleClickLock,
    RightClickLock,
    PressureHold,
    ModeToggle,
    Modifier {
        #[serde(default)] shift: bool,
        #[serde(default)] ctrl: bool,
        #[serde(default)] alt: bool,
        #[serde(default, rename = "super")] super_key: bool,
        #[serde(default)] click: Option<Click>,
    },
    Run {
        #[serde(default)] command: String,
        #[serde(default)] label: String,
    },
    PrecisionMode,
    ShowDesktop,
    Overview,
    SwitchApplication,
    NotificationCenter,
    OnScreenKeyboard,
    LeftClick,
    RightClick,
    MiddleClick,
    DoubleClick,
    Back,
    Forward,
    /// Cycle the mapping: all displays, then each monitor in turn.
    DisplayToggle,
    ExpressMenu,
    RadialMenu,
    AppShortcuts,
    Keypad,
    /// Hold the key and drag the pen tip to scroll.
    PanScroll,
    PanScrollLegacy,
    Scroll,
    Wheel,
    Zoom,
    PanZoom,
}

/// What dragging the pen tip does while a button assigned to a pen gesture
/// is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// Content moves with the pen, in any direction.
    PanScroll,
    /// Joystick: the farther the pen from where it touched down, the faster
    /// it scrolls, along one axis at a time, even while the pen rests.
    PanScrollLegacy,
    /// Like Pan/Scroll, vertical only.
    Scroll,
    /// Joystick like the legacy mode, in whole wheel notches.
    Wheel,
    /// Joystick on the vertical axis sending Ctrl+wheel: up zooms in.
    Zoom,
    /// Dragging the tip pans like Pan/Scroll; moving the pen up or down in
    /// the air zooms (Ctrl+wheel), so one button does both.
    PanZoom,
}

impl Gesture {
    /// Stable small id, for passing the gesture between threads.
    pub fn id(self) -> u8 {
        self as u8 + 1
    }

    pub fn from_id(id: u8) -> Option<Self> {
        use Gesture::*;
        [PanScroll, PanScrollLegacy, Scroll, Wheel, Zoom, PanZoom].get(usize::from(id).checked_sub(1)?).copied()
    }
}

/// On-screen shortcuts: menus Tablet Center shows where the pen is, which
/// run the chosen action once the window closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shortcut {
    ExpressMenu,
    RadialMenu,
    AppShortcuts,
    Keypad,
}

impl Shortcut {
    pub fn name(self) -> &'static str {
        match self {
            Self::ExpressMenu => "express_menu",
            Self::RadialMenu => "radial_menu",
            Self::AppShortcuts => "app_shortcuts",
            Self::Keypad => "keypad",
        }
    }
}

impl ButtonAction {
    pub fn shortcut(&self) -> Option<Shortcut> {
        Some(match self {
            Self::ExpressMenu => Shortcut::ExpressMenu,
            Self::RadialMenu => Shortcut::RadialMenu,
            Self::AppShortcuts => Shortcut::AppShortcuts,
            Self::Keypad => Shortcut::Keypad,
            _ => return None,
        })
    }

    pub fn gesture(&self) -> Option<Gesture> {
        Some(match self {
            Self::PanScroll => Gesture::PanScroll,
            Self::PanScrollLegacy => Gesture::PanScrollLegacy,
            Self::Scroll => Gesture::Scroll,
            Self::Wheel => Gesture::Wheel,
            Self::Zoom => Gesture::Zoom,
            Self::PanZoom => Gesture::PanZoom,
            _ => return None,
        })
    }
}

impl ButtonAction {
    pub fn key_action(&self) -> Option<KeyAction> {
        let value = serde_json::to_value(self).ok()?;
        let value = match value {
            serde_json::Value::String(name) => serde_json::json!({"type": name}),
            serde_json::Value::Object(map) => {
                let (name, fields) = map.into_iter().next()?;
                let mut fields = fields.as_object()?.clone();
                fields.insert("type".into(), name.into());
                serde_json::Value::Object(fields)
            }
            _ => return None,
        };
        serde_json::from_value(value).ok()
    }
}

impl KeyAction {
    pub fn shortcut(&self) -> Option<Shortcut> {
        Some(match self {
            Self::ExpressMenu => Shortcut::ExpressMenu,
            Self::RadialMenu => Shortcut::RadialMenu,
            Self::AppShortcuts => Shortcut::AppShortcuts,
            Self::Keypad => Shortcut::Keypad,
            _ => return None,
        })
    }

    fn keys(label: &str, keys: &[&str]) -> Self {
        Self::Keystroke { keys: keys.iter().map(|k| k.to_string()).collect(), label: label.into() }
    }

    pub fn gesture(&self) -> Option<Gesture> {
        Some(match self {
            Self::PanScroll => Gesture::PanScroll,
            Self::PanScrollLegacy => Gesture::PanScrollLegacy,
            Self::Scroll => Gesture::Scroll,
            Self::Wheel => Gesture::Wheel,
            Self::Zoom => Gesture::Zoom,
            Self::PanZoom => Gesture::PanZoom,
            _ => return None,
        })
    }
}

/// One entry of a radial menu or grid panel.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuItem {
    /// Shown instead of the action's name when not empty.
    pub label: String,
    pub action: KeyAction,
}

impl MenuItem {
    fn new(action: KeyAction) -> Self {
        Self { label: String::new(), action }
    }
}

/// A radial menu or grid panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OnScreen {
    /// Radial menus have 8 slices, clockwise from the top; App shortcuts
    /// 12 cells. The keypad's keys are fixed.
    pub items: Vec<MenuItem>,
    /// 1 (small) ..= 5 (large).
    pub size: u8,
    /// 1 (small) ..= 5 (large).
    pub text_size: u8,
    /// Open where the pen is instead of centred on its monitor.
    pub at_cursor: bool,
}

impl Default for OnScreen {
    fn default() -> Self {
        Self::with(Vec::new())
    }
}

impl OnScreen {
    fn with(items: Vec<MenuItem>) -> Self {
        Self { items, size: 3, text_size: 3, at_cursor: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutSettings {
    pub express_menu: OnScreen,
    pub radial_menu: OnScreen,
    pub app_shortcuts: OnScreen,
    pub keypad: OnScreen,
    /// Drawn by the Tablet Center GNOME Shell extension instead of a
    /// Tablet Center window (which stays the fallback).
    pub in_shell: bool,
}

/// Menus resolved for the GNOME Shell extension (labels translated, icon
/// names picked), written by Tablet Center next to the settings file.
pub fn shell_menus_path() -> PathBuf {
    config_path().with_file_name("shell-menus.json")
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        use KeyAction as K;
        let item = MenuItem::new;
        let undo = || K::keys("Undo", &["KEY_LEFTCTRL", "KEY_Z"]);
        let redo = || K::keys("Redo", &["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_Z"]);
        let copy = || K::keys("Copy", &["KEY_LEFTCTRL", "KEY_C"]);
        let paste = || K::keys("Paste", &["KEY_LEFTCTRL", "KEY_V"]);
        let save = || K::keys("Save", &["KEY_LEFTCTRL", "KEY_S"]);
        Self {
            express_menu: OnScreen::with(vec![
                item(undo()),
                item(redo()),
                item(copy()),
                item(paste()),
                item(save()),
                item(K::DisplayToggle),
                item(K::RightClick),
                item(K::Disabled),
            ]),
            radial_menu: OnScreen::with(vec![
                item(undo()),
                item(redo()),
                item(K::keys("Select all", &["KEY_LEFTCTRL", "KEY_A"])),
                item(K::keys("Cut", &["KEY_LEFTCTRL", "KEY_X"])),
                item(copy()),
                item(paste()),
                item(K::Keypad),
                item(K::Disabled),
            ]),
            app_shortcuts: OnScreen::with(vec![
                item(undo()),
                item(redo()),
                item(K::keys("Cut", &["KEY_LEFTCTRL", "KEY_X"])),
                item(copy()),
                item(paste()),
                item(K::keys("Select all", &["KEY_LEFTCTRL", "KEY_A"])),
                item(save()),
                item(K::keys("Open", &["KEY_LEFTCTRL", "KEY_O"])),
                item(K::keys("New", &["KEY_LEFTCTRL", "KEY_N"])),
                item(K::keys("Find", &["KEY_LEFTCTRL", "KEY_F"])),
                item(K::keys("Zoom in", &["KEY_LEFTCTRL", "KEY_EQUAL"])),
                item(K::keys("Zoom out", &["KEY_LEFTCTRL", "KEY_MINUS"])),
            ]),
            keypad: OnScreen::default(),
            in_shell: false,
        }
    }
}

/// Speed sliders of the other pen gestures, 1 (slow) ..= 5 (fast).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GestureSettings {
    pub legacy_speed: u8,
    pub scroll_speed: u8,
    pub wheel_speed: u8,
    pub zoom_speed: u8,
    pub pan_zoom_speed: u8,
}

impl Default for GestureSettings {
    fn default() -> Self {
        Self { legacy_speed: 3, scroll_speed: 3, wheel_speed: 3, zoom_speed: 3, pan_zoom_speed: 3 }
    }
}

/// Pen travel (mm) per unit of joystick speed, from the macOS slider remap
/// (`kPenButtonPanScrollSpeedSliderValueRemapping`): slow 5 mm .. fast 1 mm.
pub fn joystick_mm(speed: u8) -> f64 {
    const MM: [f64; 5] = [5.0, 4.0, 3.0, 2.0, 1.0];
    MM[usize::from(speed.clamp(1, 5)) - 1]
}

/// Wacom's USB vendor ids: 056a, and 0531 for the Wacom One pen tablets
/// (CTC4110WL / CTC6110WL, from libwacom).
pub fn wacom_vendor(vendor: u16) -> bool {
    matches!(vendor, 0x056a | 0x0531)
}

/// USB and Bluetooth ids from Linux drivers/hid/wacom_wac.c (v6.12) and
/// libwacom. Generic HID Intuos USB ids 0374/0376/0378 use the four-key
/// fallback. The Wacom One ids (0531:0100..=0105, 056a:03cb, 056a:03ce)
/// do not collide with any 056a id listed here.
pub fn express_key_count(product: u16) -> usize {
    match product {
        0x0100..=0x0105 | 0x03cb | 0x03ce => 0,
        0x0392 | 0x0393 | 0x03dd | 0x0314 => 6,
        0x0357 | 0x0358 | 0x0360 | 0x0361 | 0x0315 | 0x0317 => 8,
        _ => 4,
    }
}

/// Intuos Pro 2013 (PTH-451/651/851): BTN_0 is the touch ring's centre
/// button (libwacom `Ring=A`), so ExpressKey 1 is BTN_1.
pub fn intuos_pro_2013(product: u16) -> bool {
    matches!(product, 0x0314 | 0x0315 | 0x0317)
}

/// Tablets with a touch ring (ABS_WHEEL) and four mode LEDs.
pub fn has_touch_ring(product: u16) -> bool {
    intuos_pro_2013(product)
}

/// Pad button code offset of ExpressKey 1, counted from BTN_0.
pub fn first_express_key(product: u16) -> u16 {
    u16::from(intuos_pro_2013(product))
}

/// Wacom One 12 / 13 touch: the pen draws on the tablet's own screen.
pub fn pen_display(product: u16) -> bool {
    matches!(product, 0x03cb | 0x03ce)
}

/// The screen of a Wacom pen display, whose EDID vendor is "WAC".
pub fn own_screen(displays: &[Display]) -> Option<&Display> {
    displays.iter().find(|d| d.id.starts_with("WAC|"))
}

pub fn wireless_model(product: u16) -> bool {
    matches!(product, 0x0376 | 0x0377 | 0x0378 | 0x0379 | 0x03c6 | 0x03c8
        | 0x0392 | 0x0393 | 0x03dd | 0x0357 | 0x0358 | 0x0360 | 0x0361
        | 0x0100..=0x0105)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub apps: Vec<AppProfile>,
    pub touch: TouchSettings,
    pub precision: PrecisionSettings,
    pub pen: PenSettings,
    pub pan_scroll: PanScrollSettings,
    pub mapping: MappingSettings,
    pub express_keys: Vec<KeyAction>,
    pub touch_ring: Vec<RingAction>,
    pub gestures: GestureSettings,
    pub shortcuts: ShortcutSettings,
}

/// Finger input of a touch display (Wacom One 13 touch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TouchSettings {
    pub enabled: bool,
}

impl Default for TouchSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppProfile {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub pen: PenSettings,
    pub express_keys: Vec<KeyAction>,
}

impl Default for AppProfile {
    fn default() -> Self {
        Self { id: String::new(), name: String::new(), icon: None,
            pen: PenSettings::default(), express_keys: vec![KeyAction::Disabled; express_key_count(0)] }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

impl Config {
    pub fn grow_express_keys(&mut self, count: usize) {
        if self.express_keys.len() < count { self.express_keys.resize(count, KeyAction::Disabled); }
        for app in &mut self.apps {
            if app.express_keys.len() < count { app.express_keys.resize(count, KeyAction::Disabled); }
        }
    }

    pub fn effective(&self, app_id: Option<&str>) -> Config {
        fn normalized(id: &str) -> String {
            let id = id.to_lowercase();
            id.strip_suffix(".desktop").unwrap_or(&id).to_string()
        }
        let mut config = self.clone();
        if let Some(id) = app_id.map(normalized) {
            let profile = self.apps.iter().find(|app| normalized(&app.id) == id)
                .or_else(|| self.apps.iter().find(|app| normalized(&app.id).rsplit('.').next() == Some(id.as_str())));
            if let Some(app) = profile {
                config.pen = app.pen.clone();
                config.express_keys = app.express_keys.clone();
            }
        }
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            apps: Vec::new(),
            touch: TouchSettings::default(),
            precision: PrecisionSettings::default(),
            pen: PenSettings::default(),
            pan_scroll: PanScrollSettings::default(),
            mapping: MappingSettings::default(),
            express_keys: vec![KeyAction::Disabled; express_key_count(0)],
            touch_ring: vec![RingAction::Scroll, RingAction::Zoom, RingAction::BrushSize, RingAction::Rotate],
            gestures: GestureSettings::default(),
            shortcuts: ShortcutSettings::default(),
        }
    }
}

impl PanScrollSettings {
    /// Hi-res wheel units (120 per notch) emitted per millimetre of pen travel.
    pub fn v120_per_mm(&self) -> f64 {
        v120_per_mm(self.speed)
    }
}

/// Drag gain shared by Pan/Scroll and Scroll.
pub fn v120_per_mm(speed: u8) -> f64 {
    const GAIN: [f64; 5] = [40.0, 70.0, 110.0, 170.0, 260.0];
    GAIN[usize::from(speed.clamp(1, 5)) - 1]
}

impl PenSettings {
    pub fn click_pressure_percent(&self) -> u8 {
        // libinput decides contact itself once pressure passes about 5%, so
        // steps below that would change nothing.
        [0, 8, 13, 19, 26, 34, 42][usize::from(self.click_threshold.clamp(1, 7)) - 1]
    }

    /// Maps normalised pen pressure (0..1) through the tip sensitivity
    /// curve. Like Wacom's, the soft steps reach full pressure before the
    /// pen is pressed all the way; the firm ones need a heavier hand to
    /// leave the low end.
    pub fn curve_pressure(&self, p: f64) -> f64 {
        // (pressure that already gives 100%, exponent; below 1 is softer)
        const CURVE: [(f64, f64); 7] =
            [(0.4, 0.6), (0.55, 0.7), (0.75, 0.85), (1.0, 1.0), (1.0, 1.4), (1.0, 1.9), (1.0, 2.6)];
        let (full_at, gamma) = CURVE[usize::from(self.tip_sensitivity.clamp(1, 7)) - 1];
        (p / full_at).clamp(0.0, 1.0).powf(gamma)
    }
}

pub fn config_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".config"));
    base.join("tablet-center").join("config.json")
}

pub fn socket_path() -> PathBuf {
    env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
        .join("tabletd.sock")
}

fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

/// Missing or unreadable settings fall back to defaults.
pub fn load() -> Config {
    fs::read(config_path())
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(config: &Config) -> io::Result<()> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(config)?)?;
    fs::rename(tmp, path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Reload,
    Status,
    RunningApps,
    /// Run an action picked from an on-screen shortcut (taps it once).
    Run { action: KeyAction },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    #[serde(default)]
    pub battery: Option<Battery>,
    #[serde(default)]
    pub firmware: Option<String>,
    #[serde(skip)]
    pub sysfs_path: Option<PathBuf>,
    pub name: String,
    pub vendor: u16,
    pub product: u16,
    /// Active area of the tablet.
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Battery {
    pub percent: u8,
    pub charging: bool,
}

/// A monitor in GNOME's layout, in logical pixels.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Display {
    /// "vendor|product|serial": stable across ports and reboots.
    pub id: String,
    pub connector: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub primary: bool,
}

/// Live pen state, for the test area in the UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PenState {
    #[serde(default)]
    pub tilt_x: i32,
    #[serde(default)]
    pub tilt_y: i32,
    #[serde(default)]
    pub serial: Option<u32>,
    #[serde(default)]
    pub tool_id: Option<u32>,
    pub in_proximity: bool,
    pub tip_down: bool,
    /// Pressure after the sensitivity curve, 0.0 ..= 1.0.
    pub pressure: f64,
    pub panning: bool,
    /// Side buttons held right now, so the UI can light up the one pressed.
    #[serde(default)]
    pub upper_button: bool,
    #[serde(default)]
    pub lower_button: bool,
    /// A side button holds a mouse button on the pointer (a drag).
    #[serde(default)]
    pub mouse_held: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Response {
    #[serde(default)]
    pub apps: Vec<AppInfo>,
    #[serde(default)]
    pub focused_app: Option<String>,
    /// A touch interface is connected.
    #[serde(default)]
    pub touch: bool,
    pub ok: bool,
    pub error: Option<String>,
    pub device: Option<DeviceInfo>,
    pub pen: Option<PenState>,
    #[serde(default)]
    pub displays: Vec<Display>,
    /// Monitor the pen currently drives after Display toggle (`None` = the
    /// configured mapping applies).
    #[serde(default)]
    pub toggled_display: Option<Option<String>>,
    /// ExpressKeys held right now, left to right.
    #[serde(default)]
    pub express_keys: Vec<bool>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_ring_defaults_and_actions_round_trip() {
        let mut config: Config = serde_json::from_str(r#"{"pen":{},"express_keys":[]}"#).unwrap();
        assert_eq!(config.touch_ring, vec![RingAction::Scroll, RingAction::Zoom, RingAction::BrushSize, RingAction::Rotate]);
        config.touch_ring = vec![RingAction::Disabled, RingAction::Scroll, RingAction::Zoom,
            RingAction::BrushSize, RingAction::Rotate, RingAction::UndoRedo];
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains(r#"{"type":"brush_size"}"#));
        assert_eq!(serde_json::from_str::<Config>(&json).unwrap(), config);
        for product in [0x0314, 0x0315, 0x0317] { assert!(has_touch_ring(product)); }
        assert!(!has_touch_ring(0x0374));
    }

    #[test]
    fn wacom_one_models_are_recognised_without_express_keys() {
        assert!(wacom_vendor(0x0531) && wacom_vendor(0x056a) && !wacom_vendor(0x256c));
        for product in [0x0100, 0x0103, 0x03cb, 0x03ce] {
            assert_eq!(express_key_count(product), 0);
        }
        assert!(wireless_model(0x0101) && !wireless_model(0x03ce));
        assert!(pen_display(0x03cb) && pen_display(0x03ce) && !pen_display(0x0100));
        let screen = |id: &str| Display { id: id.into(), ..Default::default() };
        let displays = [screen("DEL|DELL S2721QS|1"), screen("WAC|Wacom One 12|2")];
        assert_eq!(own_screen(&displays).map(|d| d.id.as_str()), Some("WAC|Wacom One 12|2"));
        assert!(own_screen(&displays[..1]).is_none());
        let old: Config = serde_json::from_str("{}").unwrap();
        assert!(old.touch.enabled, "touch stays on for old configs");
    }

    #[test]
    fn old_profiles_keep_defaults_and_grow_without_losing_actions() {
        let mut config: Config = serde_json::from_str(r#"{"pen":{},"express_keys":[{"type":"left_click"}],"apps":[{"id":"krita","pen":{},"express_keys":[] }]}"#).unwrap();
        assert_eq!(config.pen.double_click_mm, 2);
        assert_eq!(config.pen.click_threshold, 1);
        assert_eq!(config.apps[0].pen.double_click_mm, 2);
        config.grow_express_keys(express_key_count(0x0357));
        assert_eq!(config.express_keys.len(), 8);
        assert_eq!(config.express_keys[0], KeyAction::LeftClick);
        assert_eq!(config.apps[0].express_keys, vec![KeyAction::Disabled; 8]);
        config.apps[0].pen.double_click_mm = 6;
        config.apps[0].pen.click_threshold = 7;
        assert_eq!(config.effective(Some("krita")).pen.click_pressure_percent(), 42);
        assert_eq!(config.effective(Some("krita")).pen.double_click_mm, 6);
    }

    #[test]
    fn application_profiles_preserve_globals_and_old_configs() {
        let mut config: Config = serde_json::from_str("{}").unwrap();
        assert!(config.apps.is_empty());
        let partial: AppProfile = serde_json::from_str(r#"{"id":"code"}"#).unwrap();
        assert_eq!(partial.pen, config.pen);
        assert_eq!(partial.express_keys, config.express_keys);
        let mut app = AppProfile { id: "org.gnome.TextEditor.desktop".into(), ..Default::default() };
        app.pen.tip_sensitivity = 7;
        app.express_keys = vec![KeyAction::LeftClick];
        config.apps.push(app);
        for id in ["ORG.GNOME.TEXTEDITOR", "org.gnome.TextEditor.desktop", "TextEditor"] {
            let effective = config.effective(Some(id));
            assert_eq!(effective.pen.tip_sensitivity, 7);
            assert_eq!(effective.express_keys, vec![KeyAction::LeftClick]);
            assert_eq!(effective.mapping, config.mapping);
            assert_eq!(effective.shortcuts, config.shortcuts);
        }
        assert_eq!(config.effective(None), config);
        assert_eq!(config.effective(Some("other")), config);
        assert_eq!(serde_json::from_value::<Config>(serde_json::to_value(&config).unwrap()).unwrap(), config);
    }

    #[test]
    fn portions_default_and_validate_on_deserialization() {
        let old: MappingSettings = serde_json::from_str(r#"{"force_proportions":true}"#).unwrap();
        assert_eq!(old.screen_portion, None);
        assert_eq!(old.tablet_portion, None);
        let r: Rect = serde_json::from_str(r#"{"x":2,"y":-1,"w":0,"h":3}"#).unwrap();
        assert_eq!(r, Rect { x: 0.95, y: 0.0, w: 0.05, h: 1.0 });
        assert_eq!(serde_json::from_value::<Rect>(serde_json::to_value(r).unwrap()).unwrap(), r);
        assert_eq!(Rect { x: f64::NAN, y: f64::INFINITY, w: f64::NAN, h: -2.0 }.clamped(),
            Rect { x: 0.0, y: 0.0, w: 1.0, h: 0.05 });
    }

    #[test]
    fn new_fields_default_for_old_configs_and_parameter_actions_round_trip() {
        let config: Config = serde_json::from_str(r#"{"pen":{"lower_button":"right_click"}}"#).unwrap();
        assert_eq!(config.pen.tip, ButtonAction::LeftClick);
        assert_eq!(config.pen.relative, RelativeSettings { speed: 4, acceleration: 4 });
        assert_eq!(config.precision, PrecisionSettings { scale: 2, overlay: true });
        let modifier: ButtonAction = serde_json::from_str(r#"{"modifier":{"ctrl":true}}"#).unwrap();
        assert_eq!(modifier.key_action(), Some(KeyAction::Modifier { ctrl: true, shift: false, alt: false, super_key: false, click: None }));
        let run: ButtonAction = serde_json::from_str(r#"{"run":{"command":"desktop:org.gnome.Nautilus.desktop"}}"#).unwrap();
        assert_eq!(serde_json::from_value::<ButtonAction>(serde_json::to_value(&run).unwrap()).unwrap(), run);
    }

    #[test]
    fn mapping_orientation_defaults_for_old_configs_and_round_trips() {
        let old: Config = serde_json::from_str(r#"{"mapping":{"force_proportions":true}}"#).unwrap();
        assert_eq!(old.mapping.orientation, Orientation::Landscape);
        for (orientation, name) in [
            (Orientation::Landscape, "landscape"),
            (Orientation::Portrait, "portrait"),
            (Orientation::LandscapeFlipped, "landscape_flipped"),
            (Orientation::PortraitFlipped, "portrait_flipped"),
        ] {
            let mut config = old.clone();
            config.mapping.orientation = orientation;
            let value = serde_json::to_value(&config).unwrap();
            assert_eq!(value["mapping"]["orientation"], name);
            assert_eq!(serde_json::from_value::<Config>(value).unwrap(), config);
        }
    }
}
