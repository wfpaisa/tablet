//! Classic CSS tokens and layout measurements.
use egui::{Color32, FontFamily, FontId};
pub const BG: Color32 = Color32::from_gray(36);
pub const BAR: Color32 = Color32::from_gray(48);
pub const LINE: Color32 = Color32::from_gray(58);
pub const PANEL: Color32 = Color32::from_gray(46);
pub const VIEW: Color32 = Color32::from_gray(30);
pub const RAISED: Color32 = Color32::from_gray(56);
pub const TRACK: Color32 = Color32::from_gray(74);
pub const TEXT: Color32 = Color32::WHITE;
pub const MUTED: Color32 = Color32::from_gray(168);
pub const FAINT: Color32 = Color32::from_gray(112);
pub const ACCENT: Color32 = Color32::from_rgb(53, 132, 228);
pub const THUMB: Color32 = Color32::from_gray(246);
pub const RING: Color32 = Color32::from_rgba_premultiplied(89, 89, 89, 89);
pub const GREEN: Color32 = Color32::from_rgb(51, 209, 122);
pub const CHIP: Color32 = Color32::from_rgb(47, 66, 87);
pub const HIGHLIGHT: Color32 = Color32::from_gray(78);
pub const CLEAR: Color32 = Color32::TRANSPARENT;
/// Light under the dark pen and tablet pictures in dark mode.
pub const DEVICE_GLOW: Color32 = Color32::from_rgba_premultiplied(26, 26, 26, 26);
/// Farthest the device glow reaches past the picture.
pub const GLOW_SPREAD: f32 = 56.;
pub const TITLE_HEIGHT: f32 = 40.;
/// Page coordinate origin: pages keep the original 116 px header layout.
pub const HEADER: f32 = 116.;
/// Height of the title bar, the only header left above the pages.
pub const TITLE_BAR: f32 = 44.;
/// Pages are drawn shifted up by this, so they start under the title bar.
pub const PAGE_SHIFT: f32 = HEADER - TITLE_BAR;
pub const SIDEBAR: f32 = 200.;
pub const CONTENT_PAD: f32 = 28.;
pub const CARD_WIDTH: f32 = 320.;
pub const CARD_RADIUS: u8 = 12;
pub const PEN_MAX: f32 = 460.;
pub fn font(size: f32) -> FontId {
    FontId::proportional(size)
}
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
pub const DEVICE: Color32 = Color32::from_gray(62);

pub const STROKE: Color32 = Color32::from_gray(232);

// Shared CSS spacing and small control dimensions.
pub const SPACE_2: f32 = 2.0;
pub const SPACE_3: f32 = 3.0;
pub const SPACE_4: f32 = 4.0;
pub const SPACE_5: f32 = 5.0;
pub const SPACE_6: f32 = 6.0;
pub const SPACE_8: f32 = 8.0;
pub const SPACE_9: f32 = 9.0;
pub const SPACE_10: f32 = 10.0;
pub const SPACE_11: f32 = 11.0;
pub const SPACE_12: f32 = 12.0;
pub const SPACE_13: f32 = 13.0;
pub const SPACE_14: f32 = 14.0;
pub const SPACE_15: f32 = 15.0;
pub const SPACE_16: f32 = 16.0;
pub const SPACE_17: f32 = 17.0;
pub const SPACE_18: f32 = 18.0;
pub const SPACE_20: f32 = 20.0;
pub const SPACE_21: f32 = 21.0;
pub const SPACE_22: f32 = 22.0;
pub const SPACE_24: f32 = 24.0;
pub const SPACE_25: f32 = 25.0;
pub const SPACE_26: f32 = 26.0;
pub const SPACE_32: f32 = 32.0;
pub const SPACE_36: f32 = 36.0;
pub const SPACE_39: f32 = 39.0;
pub const SPACE_40: f32 = 40.0;
pub const SPACE_44: f32 = 44.0;
pub const SPACE_45: f32 = 45.0;
pub const SPACE_49: f32 = 49.0;
pub const SPACE_55_HALF: f32 = 55.5;
pub const SPACE_56: f32 = 56.0;
pub const SPACE_60: f32 = 60.0;
pub const SPACE_66: f32 = 66.0;
pub const SPACE_76: f32 = 76.0;
pub const SPACE_88: f32 = 88.0;
pub const SPACE_91: f32 = 91.0;

/// Dimensions from the original frame CSS.
pub mod frame_layout {
    pub const BAR_SQUARE_TOP: f32 = 25.0;
    pub const LOGO_LEFT: f32 = 17.0;
    pub const INNER_RADIUS: f32 = 11.0;
    pub const LOGO_SIZE: f32 = 20.0;
    pub const TITLE_LEFT: f32 = 45.0;
    pub const CAPTION_RADIUS: f32 = 13.0;
    pub const CAPTION_RESERVED: f32 = 105.0;
    pub const CAPTION_RIGHT: f32 = 86.0;
    pub const CAPTION_STEP: f32 = 32.0;
    pub const CAPTION_Y: f32 = 21.0;
    pub const CAPTION_SIZE: f32 = 26.0;
    pub const GLYPH_STROKE: f32 = 1.4;
    pub const MAXIMIZE_TOP: f32 = 18.0;
    pub const CLOSE_HALF: f32 = 2.5;
    /// Resize grip along each window edge.
    pub const EDGE: f32 = 8.0;
    /// Resize grip at each window corner.
    pub const CORNER: f32 = 16.0;
    pub const TITLE_LINE_Y: f32 = super::TITLE_BAR - 0.5;
    pub const PLACEHOLDER_LEFT: f32 = 260.0;
    pub const PLACEHOLDER_TOP: f32 = 240.0;
    pub const RELEASE_DISTANCE: f32 = 10000.0;
}

/// Dimensions from the original sidebar CSS.
pub mod sidebar_layout {
    pub const BORDER_X: f32 = 200.5;
    pub const LEFT: f32 = 13.0;
    pub const DEVICE_TOP: f32 = 126.0;
    pub const ITEM_WIDTH: f32 = 175.0;
    pub const DEVICE_HEIGHT: f32 = 44.0;
    pub const DEVICE_IMAGE_LEFT: f32 = 21.0;
    pub const DEVICE_IMAGE_TOP: f32 = 132.0;
    pub const DEVICE_IMAGE_WIDTH: f32 = 40.0;
    pub const DEVICE_IMAGE_HEIGHT: f32 = 32.0;
    pub const DEVICE_LABEL_LEFT: f32 = 71.0;
    pub const DEVICE_LABEL_TOP: f32 = 137.0;
    pub const DEVICE_FONT: f32 = 17.0;
    pub const ITEM_TOP: f32 = 178.0;
    pub const ITEM_STEP: f32 = 42.4;
    pub const ITEM_HEIGHT: f32 = 41.0;
    pub const ICON_LEFT: f32 = 25.0;
    pub const ICON_SIZE: f32 = 22.0;
    pub const LABEL_LEFT: f32 = 59.0;
    /// Space kept between a label and the item's right edge.
    pub const LABEL_PAD: f32 = 8.0;
    pub const FONT: f32 = 16.0;
    /// Devices | Shortcuts switch above the navigation.
    pub const SWITCH_TOP: f32 = 54.0;
    pub const SWITCH_HEIGHT: f32 = 62.0;
    pub const SWITCH_RADIUS: u8 = 10;
    pub const SWITCH_PAD: f32 = 3.0;
    pub const SWITCH_GAP: f32 = 4.0;
    pub const SWITCH_ICON: f32 = 22.0;
    pub const SWITCH_ICON_TOP: f32 = 7.0;
    pub const SWITCH_LABEL_TOP: f32 = 42.0;
    pub const SWITCH_FONT: f32 = 12.5;
    /// Settings row pinned to the bottom, under a divider.
    pub const FOOTER: f32 = 60.0;
    pub const FOOTER_ITEM_TOP: f32 = 10.0;
}

/// Dimensions from the original control CSS.
pub mod control_layout {
    pub const HEADING_TRACKING: f32 = 0.72;
    /// Font scale for labels wrapped to two lines because they do not fit.
    pub const WRAP_SCALE: f32 = 0.875;
    /// Gap between a label and its trailing help icon or chip edge.
    pub const LABEL_GAP: f32 = 10.0;
    pub const LABEL_FONT: f32 = 15.0;
    pub const HELP_GAP: f32 = 26.5;
    pub const HELP_CENTER_Y: f32 = 10.0;
    pub const HELP_RADIUS: f32 = 9.0;
    pub const HELP_STROKE: f32 = 1.5;
    pub const HELP_TARGET: f32 = 20.0;
    pub const HELP_WIDTH: f32 = 260.0;
    pub const THUMB_INSET: f32 = 13.0;
    pub const TRACK_TOP: f32 = 36.5;
    pub const SLIDER_TARGET_TOP: f32 = 26.0;
    pub const SLIDER_TARGET_HEIGHT: f32 = 25.0;
    pub const THUMB_DIAMETER: f32 = 22.0;
    pub const THUMB_CENTER_Y: f32 = 39.0;
    pub const RING_RADIUS: f32 = 9.0;
    pub const DOT_RADIUS: f32 = 6.0;
    pub const TICK_INSET: f32 = 10.5;
    pub const TICK_INSETS: f32 = 21.0;
    pub const TICKS_Y: f32 = 55.5;
    pub const SCALE_TOP: f32 = 66.0;
    /// Single-line text boxes: one look for every dialog and page.
    pub const FIELD_FONT: f32 = 15.0;
    pub const FIELD_PAD_X: f32 = 12.0;
    pub const FIELD_RADIUS: u8 = 8;
}

/// Dimensions from the original pen CSS.
pub mod pen_layout {
    pub const STRIP_TOP: f32 = 144.0;
    /// Hold time card: shared slider, then its description.
    pub const HOLD_SLIDER_HEIGHT: f32 = 82.0;
    pub const HOLD_NOTE_TOP: f32 = 120.0;
    /// Actions panel on the right while a pen button is selected.
    pub const PANEL_WIDTH: f32 = 300.0;
    /// ExpressKeys page: tablet art beside its key callouts.
    pub const KEYS_ART_WIDTH: f32 = 340.0;
    pub const KEYS_ART_MIN_WIDTH: f32 = 180.0;
    pub const INTUOS_ART_RATIO: f32 = 351.17 / 440.0;
    pub const KEYS_ART_RATIO: f32 = 360.0 / 560.0;
    pub const KEYS_DEFAULT_GAP: f32 = 24.0;
    pub const PANEL_INSET: f32 = 18.0;
    pub const PANEL_TITLE_TOP: f32 = 16.0;
    pub const PANEL_SUBTITLE_TOP: f32 = 42.0;
    pub const PANEL_HEADER: f32 = 70.0;
    pub const PANEL_CLOSE: f32 = 28.0;
    pub const PANEL_HEADING_STEP: f32 = 32.0;
    /// Space above every group heading but the first.
    pub const PANEL_GROUP_GAP: f32 = 16.0;
    pub const ROW_INSET: f32 = 8.0;
    pub const CALLOUT_REACH: f32 = 72.0;
    pub const CALLOUT_BEND: f32 = 24.0;
    pub const CALLOUT_WIDTH: f32 = 200.0;
    pub const CALLOUT_BOTTOM: f32 = 12.0;
    pub const CALLOUT_GAP: f32 = 10.0;
    pub const CALLOUT_PAD: f32 = 14.0;
    pub const CALLOUT_NAME_TOP: f32 = 10.0;
    pub const CALLOUT_ACTION_TOP: f32 = 32.0;
    pub const CALLOUT_ICON: f32 = 20.0;
    pub const CALLOUT_CHEVRON_ROOM: f32 = 30.0;
    pub const SPOT_RADIUS: f32 = 7.0;
    pub const SPOT_STROKE: f32 = 1.5;
    pub const CONTENT_RIGHT: f32 = 28.0;
    pub const STRIP_HEIGHT: f32 = 54.0;
    pub const STRIP_HEADING_TOP: f32 = 158.0;
    pub const CHIP_LEFT: f32 = 119.0;
    pub const CHIP_TOP: f32 = 147.0;
    pub const CHIP_WIDTH: f32 = 118.0;
    pub const CHIP_HEIGHT: f32 = 36.0;
    pub const ART_SOURCE_WIDTH: f32 = 131.0;
    pub const BUTTON_LEAD_MAX: f32 = 140.0;
    pub const CHIP_GRID_BOTTOM: f32 = 167.0;
    pub const STRIP_LABEL_TOP: f32 = 156.0;
    pub const ADD_RIGHT: f32 = 159.0;
    pub const STRIP_BUTTON_TOP: f32 = 154.0;
    pub const REMOVE_RIGHT: f32 = 120.0;
    pub const ACTIVE_BORDER_RIGHT: f32 = 93.5;
    pub const ACTIVE_BORDER_BOTTOM: f32 = 174.0;
    pub const ACTIVE_DOT_RIGHT: f32 = 79.5;
    pub const ACTIVE_DOT_Y: f32 = 165.5;
    pub const LEAD_DOT_RADIUS: f32 = 3.5;
    pub const ACTIVE_LABEL_RIGHT: f32 = 69.0;
    pub const ART_RESERVED_HEIGHT: f32 = 270.0;
    pub const PAGE_TOP: f32 = 210.0;
    pub const PAGE_BOTTOM_RESERVED: f32 = 211.0;
    pub const CANVAS_RESERVED_HEIGHT: f32 = 700.0;
    pub const CANVAS_MIN_HEIGHT: f32 = 88.0;
    pub const CANVAS_MAX_HEIGHT: f32 = 176.0;
    pub const COMPACT_CONTENT_HEIGHT: f32 = 819.0;
    pub const COLUMN_GAP: f32 = 44.0;
    pub const ART_SOURCE_HEIGHT: f32 = 1599.0;
    pub const ART_RIGHT_RATIO: f32 = 0.12;
    pub const UPPER_Y_RATIO: f32 = 0.645;
    pub const BUTTON_X_RATIO: f32 = 0.4;
    pub const LOWER_Y_RATIO: f32 = 0.735;
    pub const TIP_Y_RATIO: f32 = 0.975;
    pub const TIP_X_RATIO: f32 = 0.49;
    pub const LABEL_RESERVED: f32 = 170.0;
    pub const MIN_LEAD: f32 = 30.0;
    pub const TIP_SPOT_THRESHOLD: f32 = 0.9;
    pub const TIP_LEAD_MAX: f32 = 150.0;
    pub const TEST_CHROME_HEIGHT: f32 = 97.0;
    pub const CARD_INSET: f32 = 19.0;
    pub const CANVAS_TOP: f32 = 45.0;
    pub const CARD_INNER_WIDTH: f32 = 282.0;
    pub const STROKE_MIN: f32 = 1.5;
    pub const STROKE_RANGE: f32 = 26.5;
    pub const PRESSURE_SMOOTHING: f32 = 0.6;
    pub const CARD_GAP: f32 = 28.0;
    pub const TIP_HEIGHT: f32 = 341.0;
    pub const FIRST_SLIDER_TOP: f32 = 46.0;
    pub const SLIDER_HEIGHT: f32 = 83.0;
    pub const SECOND_SLIDER_TOP: f32 = 142.5;
    pub const THIRD_SLIDER_TOP: f32 = 239.0;
    pub const MODE_TOP: f32 = 369.0;
    pub const MODE_HEIGHT: f32 = 37.0;
    pub const MODE_ITEM_HEIGHT: f32 = 31.0;
    pub const HINT_LINE_HEIGHT: f32 = 18.85;
    pub const MODE_HINT_TOP: f32 = 92.0;
    pub const RESTORE_LEFT: f32 = 198.0;
    pub const RESTORE_TOP: f32 = 161.0;
    pub const RESTORE_WIDTH: f32 = 122.0;
    pub const RESTORE_HEIGHT: f32 = 32.0;
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}

/// Tall enough for the tablet, pen and ExpressKeys art without scrolling.
pub const WINDOW_SIZE: [f32; 2] = [1100., 880.];
/// Largest share of the monitor the first window may take.
pub const WINDOW_SCREEN_SHARE: f32 = 0.9;
pub const WINDOW_MIN: [f32; 2] = [800., 600.];
pub const TEXT_GAMMA: f32 = 0.86;
pub const RADIUS: u8 = 8;
pub const CONTROL_RADIUS: u8 = 6;
pub const PILL_RADIUS: u8 = 18;
pub const BORDER: f32 = 1.;

pub const SQUARE: u8 = 0;
pub const TRACK_RADIUS: u8 = 3;
pub const INNER_RADIUS: u8 = 11;
pub const WINDOW_RADIUS: u8 = 12;

// Device page colors from style.css and native browser meter defaults.
pub const KEY_RING: Color32 = Color32::from_gray(100);
pub const WATERMARK: Color32 = Color32::from_gray(43);
pub const LINK: Color32 = Color32::from_rgb(170, 170, 255);
pub const METER_BORDER: Color32 = Color32::from_gray(128);
pub const METER_FILL: Color32 = Color32::from_rgb(119, 181, 119);
pub const BEAM: Color32 = Color32::from_rgba_premultiplied(6, 16, 28, 31);
pub const BEAM_EDGE: Color32 = Color32::from_rgba_premultiplied(29, 72, 125, 140);
pub const AREA: Color32 = Color32::from_rgba_premultiplied(18, 46, 80, 89);
pub const SCREEN: Color32 = Color32::from_rgb(41, 66, 97);
pub const DIM: Color32 = Color32::from_black_alpha(140);
pub fn if_screen(on: bool) -> Color32 {
    if on {
        SCREEN
    } else {
        WATERMARK
    }
}

pub const GLOW: Color32 = Color32::from_rgba_premultiplied(42, 106, 182, 204);
pub const TOGGLE_OUTLINE: Color32 = Color32::from_rgb(54, 61, 68);
pub const TOGGLE_BEAM: Color32 = Color32::from_rgb(37, 40, 43);
pub const TOGGLE_DOT: Color32 = Color32::from_gray(49);

pub const SCREEN_SIZE: Color32 = Color32::from_rgb(201, 209, 219);

/// Device CSS pixel dimensions, shared by pages and modal controls.
pub mod devices_layout {
    pub const PX_1_3: f32 = 1.3;
    pub const PX_594: f32 = 594.0;
    pub const PX_0_9: f32 = 0.9;
    pub const PX_0_5: f32 = 0.5;
    pub const PX_636: f32 = 636.0;
    pub const PX_552: f32 = 552.0;
    pub const PX_678: f32 = 678.0;
    pub const PX_679: f32 = 679.0;
    pub const PX_637: f32 = 637.0;
    pub const PX_595: f32 = 595.0;
    pub const PX_553: f32 = 553.0;
    pub const PX_448: f32 = 448.0;
    pub const PX_422: f32 = 422.0;
    pub const PX_369: f32 = 369.0;
    pub const PX_358: f32 = 358.0;
    pub const PX_301: f32 = 301.0;
    pub const PX_248: f32 = 248.0;
    pub const PX_206: f32 = 206.0;
    pub const PX_0_075: f32 = 0.075;
    pub const PX_431: f32 = 431.;
    pub const PX_425: f32 = 425.;
    pub const PX_738: f32 = 738.;
    pub const PX_61: f32 = 61.;
    pub const PX_46: f32 = 46.;
    pub const PX_41: f32 = 41.;
    pub const PX_394: f32 = 394.;
    pub const PX_37: f32 = 37.;
    pub const PX_364: f32 = 364.;
    pub const PX_318: f32 = 318.;
    pub const PX_308: f32 = 308.;
    pub const PX_285: f32 = 285.;
    pub const PX_202: f32 = 202.;
    pub const PX_1_9: f32 = 1.9;
    pub const PX_1_5: f32 = 1.5;
    pub const PX_1_45: f32 = 1.45;
    pub const PX_1_15: f32 = 1.15;
    pub const PX_19_6: f32 = 19.6;
    pub const PX_165: f32 = 165.;
    pub const PX_13: f32 = 13.;
    pub const PX_126: f32 = 126.;
    pub const PX_102: f32 = 102.;
    pub const PX_1: f32 = 1.;
    pub const PX_0_22: f32 = 0.22;
    pub const PX_0_088: f32 = 0.088;
    pub const PX_198: f32 = 198.;
    pub const PX_143: f32 = 143.;
    pub const PX_482: f32 = 482.;
    pub const PX_484: f32 = 484.;
    pub const PX_315: f32 = 315.;
    pub const KEY_WIDTH_VH: f32 = 0.52;
    pub const KEY_SPOTS: [f32; 4] = [0.19, 0.396, 0.602, 0.808];
    pub const KEY_Y: f32 = 0.12;
    pub const KEY_STEP: f32 = 0.045;
    pub const KEY_LABEL_Y: f32 = 0.19;
    pub const KEY_RING_HALF_WIDTH: f32 = 0.085;
    pub const KEY_RING_HALF_HEIGHT: f32 = 0.038;
    pub const KEY_RING_WIDTH: f32 = 0.17;
    pub const KEY_RING_HEIGHT: f32 = 0.1;
    pub const LINE_HEIGHT: f32 = 1.4;
    pub const TOGGLE_LINE_HEIGHT: f32 = 1.35;
    pub const TABLET_ACTIVE_X: f32 = 0.114;
    pub const TABLET_ACTIVE_Y: f32 = 0.304;
    pub const TABLET_ACTIVE_WIDTH: f32 = 0.79;
    pub const TABLET_ACTIVE_HEIGHT: f32 = 0.622;
    pub const PORTRAIT_SCALE: f32 = 0.79;
    pub const THIN_STROKE: f32 = 1.5;
    pub const PORTION_INSET: f64 = 0.1;
    pub const PORTION_SIZE: f64 = 0.8;
    pub const PORTION_MIN: f64 = 0.05;
    pub const TOGGLE_HEIGHT_VH: f32 = 0.34;

    pub const PX_515: f32 = 515.;
    pub const PX_197: f32 = 197.;
    pub const PX_456: f32 = 456.;
    pub const PX_332: f32 = 332.;
    pub const PX_53: f32 = 53.;
    pub const PX_2: f32 = 2.;
    pub const PX_3: f32 = 3.;
    pub const PX_3_5: f32 = 3.5;
    pub const PX_4: f32 = 4.;
    pub const PX_5: f32 = 5.;
    pub const PX_6: f32 = 6.;
    pub const PX_7: f32 = 7.;
    pub const PX_8: f32 = 8.;
    pub const PX_9: f32 = 9.;
    pub const PX_10: f32 = 10.;
    pub const PX_11: f32 = 11.;
    pub const PX_12: f32 = 12.;
    pub const PX_14: f32 = 14.;
    pub const PX_15: f32 = 15.;
    pub const PX_16: f32 = 16.;
    pub const PX_17: f32 = 17.;
    pub const PX_18: f32 = 18.;
    pub const PX_19: f32 = 19.;
    pub const PX_20: f32 = 20.;
    pub const PX_21: f32 = 21.;
    pub const PX_22: f32 = 22.;
    pub const PX_23: f32 = 23.;
    pub const PX_24: f32 = 24.;
    pub const PX_25: f32 = 25.;
    pub const PX_26: f32 = 26.;
    pub const PX_27: f32 = 27.;
    pub const PX_28: f32 = 28.;
    pub const PX_29: f32 = 29.;
    pub const PX_30: f32 = 30.;
    pub const PX_31: f32 = 31.;
    pub const PX_32: f32 = 32.;
    pub const PX_33: f32 = 33.;
    pub const PX_34: f32 = 34.;
    pub const PX_35: f32 = 35.;
    pub const PX_36: f32 = 36.;
    pub const PX_38: f32 = 38.;
    pub const PX_39: f32 = 39.;
    pub const PX_40: f32 = 40.;
    pub const PX_44: f32 = 44.;
    pub const PX_45: f32 = 45.;
    pub const PX_48: f32 = 48.;
    pub const PX_50: f32 = 50.;
    pub const PX_52: f32 = 52.;
    pub const PX_54: f32 = 54.;
    pub const PX_55: f32 = 55.;
    pub const PX_56: f32 = 56.;
    pub const PX_57: f32 = 57.;
    pub const PX_59: f32 = 59.;
    pub const PX_60: f32 = 60.;
    pub const PX_63: f32 = 63.;
    pub const PX_64: f32 = 64.;
    pub const PX_65: f32 = 65.;
    pub const PX_68: f32 = 68.;
    pub const PX_69: f32 = 69.;
    pub const PX_70: f32 = 70.;
    pub const PX_71: f32 = 71.;
    pub const PX_72: f32 = 72.;
    pub const PX_73: f32 = 73.;
    pub const PX_75: f32 = 75.;
    pub const PX_79_5: f32 = 79.5;
    pub const PX_80: f32 = 80.;
    pub const PX_85: f32 = 85.;
    pub const PX_88: f32 = 88.;
    pub const PX_90: f32 = 90.;
    pub const PX_92_5: f32 = 92.5;
    pub const PX_92: f32 = 92.;
    pub const PX_93_5: f32 = 93.5;
    pub const PX_95: f32 = 95.;
    pub const PX_96: f32 = 96.;
    pub const PX_100: f32 = 100.;
    pub const PX_110: f32 = 110.;
    pub const PX_113: f32 = 113.;
    pub const PX_116: f32 = 116.;
    pub const PX_117: f32 = 117.;
    pub const PX_118: f32 = 118.;
    pub const PX_120: f32 = 120.;
    pub const PX_122: f32 = 122.;
    pub const PX_123: f32 = 123.;
    pub const PX_124: f32 = 124.;
    pub const PX_125: f32 = 125.;
    pub const PX_129: f32 = 129.;
    pub const PX_130: f32 = 130.;
    pub const PX_132: f32 = 132.;
    pub const PX_133: f32 = 133.;
    pub const PX_134: f32 = 134.;
    pub const PX_138: f32 = 138.;
    pub const PX_140: f32 = 140.;
    pub const PX_145: f32 = 145.;
    pub const PX_147: f32 = 147.;
    pub const PX_150: f32 = 150.;
    pub const PX_151: f32 = 151.;
    pub const PX_154: f32 = 154.;
    pub const PX_156: f32 = 156.;
    pub const PX_158: f32 = 158.;
    pub const PX_159: f32 = 159.;
    pub const PX_160: f32 = 160.;
    pub const PX_165_5: f32 = 165.5;
    pub const PX_166: f32 = 166.;
    pub const PX_167: f32 = 167.;
    pub const PX_168: f32 = 168.;
    pub const PX_170: f32 = 170.;
    pub const PX_171: f32 = 171.;
    pub const PX_174: f32 = 174.;
    pub const PX_175: f32 = 175.;
    pub const PX_176: f32 = 176.;
    pub const PX_178: f32 = 178.;
    pub const PX_180: f32 = 180.;
    pub const PX_187: f32 = 187.;
    pub const PX_190: f32 = 190.;
    pub const PX_196: f32 = 196.;
    pub const PX_200: f32 = 200.;
    pub const PX_201: f32 = 201.;
    pub const PX_205: f32 = 205.;
    pub const PX_207: f32 = 207.;
    pub const PX_208: f32 = 208.;
    pub const PX_209: f32 = 209.;
    pub const PX_210: f32 = 210.;
    pub const PX_211: f32 = 211.;
    pub const PX_214: f32 = 214.;
    pub const PX_215: f32 = 215.;
    pub const PX_217: f32 = 217.;
    pub const PX_218: f32 = 218.;
    pub const PX_220: f32 = 220.;
    pub const PX_221: f32 = 221.;
    pub const PX_222: f32 = 222.;
    pub const PX_225: f32 = 225.;
    pub const PX_229: f32 = 229.;
    pub const PX_230: f32 = 230.;
    pub const PX_231: f32 = 231.;
    pub const PX_234: f32 = 234.;
    pub const PX_235: f32 = 235.;
    pub const PX_236: f32 = 236.;
    pub const PX_240: f32 = 240.;
    pub const PX_241: f32 = 241.;
    pub const PX_242: f32 = 242.;
    pub const PX_243: f32 = 243.;
    pub const PX_245: f32 = 245.;
    pub const PX_251: f32 = 251.;
    pub const PX_257: f32 = 257.;
    pub const PX_258: f32 = 258.;
    pub const PX_260: f32 = 260.;
    pub const PX_263: f32 = 263.;
    pub const PX_265: f32 = 265.;
    pub const PX_270: f32 = 270.;
    pub const PX_275: f32 = 275.;
    pub const PX_277: f32 = 277.;
    pub const PX_280: f32 = 280.;
    pub const PX_288: f32 = 288.;
    pub const PX_289: f32 = 289.;
    pub const PX_290: f32 = 290.;
    pub const PX_300: f32 = 300.;
    pub const PX_323: f32 = 323.;
    pub const PX_327: f32 = 327.;
    pub const PX_329: f32 = 329.;
    pub const PX_330: f32 = 330.;
    pub const PX_340: f32 = 340.;
    pub const PX_348: f32 = 348.;
    pub const PX_356: f32 = 356.;
    pub const PX_360: f32 = 360.;
    pub const PX_363: f32 = 363.;
    pub const PX_370: f32 = 370.;
    pub const PX_372: f32 = 372.;
    pub const PX_380: f32 = 380.;
    pub const PX_384: f32 = 384.;
    pub const PX_387: f32 = 387.;
    pub const PX_400: f32 = 400.;
    pub const PX_405: f32 = 405.;
    pub const PX_409: f32 = 409.;
    pub const PX_428: f32 = 428.;
    pub const PX_430: f32 = 430.;
    pub const PX_432: f32 = 432.;
    pub const PX_436: f32 = 436.;
    pub const PX_440: f32 = 440.;
    pub const PX_471: f32 = 471.;
    pub const PX_472: f32 = 472.;
    pub const PX_473: f32 = 473.;
    pub const PX_478: f32 = 478.;
    pub const PX_480: f32 = 480.;
    pub const PX_485: f32 = 485.;
    pub const PX_487: f32 = 487.;
    pub const PX_490: f32 = 490.;
    pub const PX_492: f32 = 492.;
    pub const PX_500: f32 = 500.;
    pub const PX_502: f32 = 502.;
    pub const PX_520: f32 = 520.;
    pub const PX_540: f32 = 540.;
    pub const PX_560: f32 = 560.;
    pub const PX_568: f32 = 568.;
    pub const PX_600: f32 = 600.;
    pub const PX_650: f32 = 650.;
    pub const PX_669: f32 = 669.;
    pub const PX_670: f32 = 670.;
    pub const PX_769: f32 = 769.;
    pub const PX_845: f32 = 845.;
    pub const PX_892: f32 = 892.;
}
pub const DISABLED_ACCENT: Color32 = Color32::from_rgb(50, 89, 137);

pub const TOGGLE_BEAM_ACTIVE: Color32 = Color32::from_rgba_premultiplied(9, 24, 41, 46);

pub const FULL_RADIUS: u8 = 255;

pub const UNUSED_AREA: Color32 = Color32::from_black_alpha(89);

pub const DIALOG_SHADOW: Color32 = Color32::from_black_alpha(128);

/// On-screen shortcut dimensions copied from style.css.
pub mod shortcuts_layout {
    pub const SIDEBAR_CONTENT_HEIGHT: f32 = 720.;
    pub const ACTION_GROUP_HEIGHT: f32 = 38.4;
    use egui::Color32;
    pub const RADIAL_SIZES: [f32; 5] = [240., 290., 340., 390., 440.];
    pub const CELL_SIZES: [f32; 5] = [60., 70., 80., 92., 104.];
    pub const TEXT_SIZES: [f32; 5] = [11., 12., 13., 15., 17.];
    pub const GESTURE: Color32 = Color32::from_gray(110);
    pub const HOVER: Color32 = Color32::from_gray(68);
    pub const WATERMARK: Color32 = Color32::from_gray(43);
    pub const SHADOW: Color32 = Color32::from_black_alpha(115);
    pub const RADIAL_UNITS: f32 = 204.;
    pub const LABEL_RADIUS: f32 = 67.;
    pub const PREVIEW_WIDTH: f32 = 460.;
    pub const OPTIONS_WIDTH: f32 = 240.;
    pub const GAP: f32 = 50.;
    pub const TOP: f32 = 148.;
    pub const MENU_TOP: f32 = 215.;
    pub const OPTIONS_TOP: f32 = 245.;
    pub const SLIDER_STEP: f32 = 118.;
    pub const TOGGLE_TOP: f32 = 483.;
    pub const GESTURE_TOP: f32 = 213.;
    pub const GESTURE_SIZE: f32 = 260.;
    pub const SPEED_TOP: f32 = 499.;
    pub const SPEED_WIDTH: f32 = 420.;
    pub const FOLLOW_TOP: f32 = 619.;
    pub const HINT_TOP: f32 = 673.;
    pub const HINT_PLAIN_TOP: f32 = 614.;
    pub const LINE_HEIGHT: f32 = 21.75;
    pub const CELL_GAP: f32 = 6.;
    pub const GRID_PAD: f32 = 13.;
    pub const GRID_HEADER: f32 = 47.;
    pub const GRID_BOTTOM: f32 = 13.;
    pub const EDGE: f32 = 8.;
}

// Settings CSS status and modal tokens.
pub mod settings_layout {
    use egui::Color32;
    pub const BACKUP_TITLE_WIDTH: f32 = 208.;
    pub const BACKUP_LINE_HEIGHT: f32 = 21.;
    pub const HINT_WIDTH: f32 = 720.;
    pub const CONTENT_TOP: f32 = 187.;
    /// Single page: a heading over each section card, stacked down the page.
    pub const SECTION_TOP: f32 = 138.;
    pub const SECTION_FONT: f32 = 20.;
    pub const SECTION_CARD: f32 = 38.;
    pub const SECTION_GAP: f32 = 34.;
    pub const BOTTOM_MARGIN: f32 = 24.;
    pub const GENERAL_HEIGHT: f32 = 542.;
    pub const DEVICE_HEIGHT: f32 = 158.;
    pub const NO_DEVICE_HEIGHT: f32 = 62.;
    pub const BACKUP_HEIGHT: f32 = 190.;
    /// Backups card without the empty-table row, before the rows' heights.
    pub const BACKUP_ROWS_HEIGHT: f32 = 146.;
    /// General rows below the language field.
    pub const AUTOSTART_Y: f32 = 556.;
    pub const AUTOSTART_RULE: f32 = 616.5;
    pub const SHELL_Y: f32 = 647.;
    pub const HINT_Y: f32 = 686.;
    pub const CJK_OFFSET: f32 = -2.;
    pub const INPUT_BORDER: Color32 = Color32::from_gray(118);
    pub const INPUT: Color32 = Color32::from_gray(59);
    pub const FOCUS: Color32 = Color32::from_rgba_premultiplied(16, 40, 69, 77);
    pub const ACTIVE: Color32 = Color32::from_gray(68);
    pub const RUNNING: Color32 = Color32::from_rgb(48, 164, 108);
    pub const STOPPED: Color32 = Color32::from_rgb(229, 72, 77);
    pub const DIALOG_SHADOW: Color32 = Color32::from_black_alpha(128);
    pub const SELECT_SHADOW: Color32 = Color32::from_black_alpha(115);
    pub const TOAST_SHADOW: Color32 = Color32::from_black_alpha(102);
    pub const BACKDROP: Color32 = Color32::from_black_alpha(115);
    pub const SWITCH_RADIUS: u8 = 15;
    pub const MENU_RADIUS: u8 = 10;
    pub const DIALOG_SHADOW_Y: i8 = 20;
    pub const DIALOG_SHADOW_BLUR: u8 = 50;
    pub const SELECT_SHADOW_Y: i8 = 10;
    pub const SELECT_SHADOW_BLUR: u8 = 30;
    pub const TOAST_SHADOW_Y: i8 = 6;
    pub const TOAST_SHADOW_BLUR: u8 = 20;
    pub const TOAST_MS: u64 = 3500;
    pub const PX_0: f32 = 0.;
    pub const PX_0_04: f32 = 0.04;
    pub const PX_0_1: f32 = 0.1;
    pub const PX_0_13: f32 = 0.13;
    pub const PX_0_16: f32 = 0.16;
    pub const PX_0_18: f32 = 0.18;
    pub const PX_0_23: f32 = 0.23;
    pub const PX_0_68: f32 = 0.68;
    pub const PX_1: f32 = 1.;
    pub const PX_1_45: f32 = 1.45;
    pub const PX_1_5: f32 = 1.5;
    pub const PX_1_6: f32 = 1.6;
    pub const PX_2: f32 = 2.;
    pub const PX_3: f32 = 3.;
    pub const PX_4: f32 = 4.;
    pub const PX_4_5: f32 = 4.5;
    pub const PX_5: f32 = 5.;
    pub const PX_6: f32 = 6.;
    pub const PX_7: f32 = 7.;
    pub const PX_8: f32 = 8.;
    pub const PX_9: f32 = 9.;
    pub const PX_10: f32 = 10.;
    pub const PX_12: f32 = 12.;
    pub const PX_13: f32 = 13.;
    pub const PX_14: f32 = 14.;
    pub const PX_15: f32 = 15.;
    pub const PX_16: f32 = 16.;
    pub const PX_17_55: f32 = 17.55;
    pub const PX_18: f32 = 18.;
    pub const PX_19: f32 = 19.;
    pub const PX_20: f32 = 20.;
    pub const PX_22: f32 = 22.;
    pub const PX_24: f32 = 24.;
    pub const PX_24_57: f32 = 24.57;
    pub const PX_29: f32 = 29.;
    pub const PX_30: f32 = 30.;
    pub const PX_32: f32 = 32.;
    pub const PX_33: f32 = 33.;
    pub const PX_33_6: f32 = 33.6;
    pub const PX_35: f32 = 35.;
    pub const PX_36: f32 = 36.;
    pub const PX_38: f32 = 38.;
    pub const PX_39: f32 = 39.;
    pub const PX_40: f32 = 40.;
    pub const PX_48: f32 = 48.;
    pub const PX_49: f32 = 49.;
    pub const PX_52: f32 = 52.;
    pub const PX_52_5: f32 = 52.5;
    pub const PX_55: f32 = 55.;
    pub const PX_56: f32 = 56.;
    pub const PX_58: f32 = 58.;
    pub const PX_61: f32 = 61.;
    pub const PX_63: f32 = 63.;
    pub const PX_65: f32 = 65.;
    pub const PX_77: f32 = 77.;
    pub const PX_82: f32 = 82.;
    pub const PX_88: f32 = 88.;
    pub const PX_95: f32 = 95.;
    pub const PX_96: f32 = 96.;
    pub const PX_100: f32 = 100.;
    pub const PX_138: f32 = 138.;
    pub const PX_141: f32 = 141.;
    pub const PX_146: f32 = 146.;
    pub const PX_160: f32 = 160.;
    pub const PX_180: f32 = 180.;
    pub const PX_181: f32 = 181.;
    pub const PX_187: f32 = 187.;
    pub const PX_190: f32 = 190.;
    pub const PX_200: f32 = 200.;
    pub const PX_204: f32 = 204.;
    pub const PX_206: f32 = 206.;
    pub const PX_210: f32 = 210.;
    pub const PX_225: f32 = 225.;
    pub const PX_230: f32 = 230.;
    pub const PX_235: f32 = 235.;
    pub const PX_244: f32 = 244.;
    pub const PX_246: f32 = 246.;
    pub const PX_254_5: f32 = 254.5;
    pub const PX_272: f32 = 272.;
    pub const PX_274: f32 = 274.;
    pub const PX_295_5: f32 = 295.5;
    pub const PX_306_5: f32 = 306.5;
    pub const PX_320: f32 = 320.;
    pub const PX_321: f32 = 321.;
    pub const PX_326: f32 = 326.;
    pub const PX_332: f32 = 332.;
    pub const PX_352_5: f32 = 352.5;
    pub const PX_360: f32 = 360.;
    pub const PX_380: f32 = 380.;
    pub const PX_389_5: f32 = 389.5;
    pub const PX_422: f32 = 422.;
    pub const PX_460: f32 = 460.;
    pub const PX_525_5: f32 = 525.5;
    pub const PX_558: f32 = 558.;
    pub const PX_560: f32 = 560.;
    pub const PX_595: f32 = 595.;
    pub const PX_661_5: f32 = 661.5;
    pub const PX_678: f32 = 678.;
    pub const PX_692: f32 = 692.;
    pub const PX_745: f32 = 745.;
    pub const PX_752_5: f32 = 752.5;
    pub const PX_783: f32 = 783.;
    pub const PX_808: f32 = 808.;
    pub const PX_822: f32 = 822.;
    pub const PX_842: f32 = 842.;
}

/// ExpressKey spots on the Intuos S artwork, as fractions of the image.
pub mod device_art {
    pub const INTUOS_KEY_X: [f32; 4] = [0.189, 0.395, 0.602, 0.808];
    pub const INTUOS_KEY_Y: f32 = 0.15;
}

/// Interaction values copied from style.css, select.css and frame.css.
/// Application strip: app pills, the Add pill, hover remove and scroll arrows.
/// Overview hero: connection status beside the model, then the info button.
pub mod overview_layout {
    pub const STATUS_GAP: f32 = 16.;
    pub const STATUS_ICON: f32 = 16.;
    pub const STATUS_TEXT_GAP: f32 = 6.;
    pub const BUTTON_TOP: f32 = 246.;
    pub const BUTTON_HEIGHT: f32 = 34.;
    pub const BUTTON_PAD: f32 = 10.;
    pub const BUTTON_ICON: f32 = 16.;
    pub const BUTTON_GAP: f32 = 8.;
    pub const CHEVRON: f32 = 14.;
}
/// About this device: details and live pen input cards.
pub mod about_layout {
    pub const LEFT: f32 = 289.;
    pub const TOP: f32 = 236.;
    pub const MAX_WIDTH: f32 = 600.;
    pub const MIN_COLUMN: f32 = 340.;
    pub const MAX_COLUMN: f32 = 440.;
    pub const GAP: f32 = 16.;
    pub const PAD: f32 = 20.;
    pub const ICON: f32 = 16.;
    pub const TITLE_LEFT: f32 = 26.;
    pub const TITLE: f32 = 16.;
    pub const NAME_TOP: f32 = 54.;
    pub const KIND_TOP: f32 = 76.;
    pub const DIVIDER_TOP: f32 = 106.;
    pub const ROWS_TOP: f32 = 122.;
    pub const ROW_STEP: f32 = 30.;
    pub const LABEL: f32 = 14.;
    pub const VALUE: f32 = 15.;
    pub const VALUE_COL: f32 = 170.;
    pub const PILL_HEIGHT: f32 = 26.;
    pub const PILL_PAD: f32 = 12.;
    pub const DOT: f32 = 3.;
    pub const METERS_TOP: f32 = 56.;
    pub const METER_STEP: f32 = 54.;
    pub const BAR_TOP: f32 = 24.;
    pub const BAR: f32 = 6.;
}
/// Display toggle help dialog, showing the old Display toggle page.
pub mod toggle_dialog {
    pub const WIDTH: f32 = 600.;
    pub const HEIGHT: f32 = 520.;
    pub const ART_SCALE: f32 = 0.85;
    pub const ART_TOP: f32 = 64.;
    pub const CLOSE: f32 = 28.;
    /// Help mark drawn at the right of the Display toggle action row.
    pub const MARK_RIGHT: f32 = 20.;
}
/// Open/Run dialog: command field over the installed-apps list.
pub mod run_dialog {
    pub const WIDTH: f32 = 460.;
    pub const HEIGHT: f32 = 500.;
    pub const PAD: f32 = 24.;
    pub const TITLE: f32 = 18.;
    pub const LABEL: f32 = 13.;
    pub const FIELD_LABEL_TOP: f32 = 44.;
    pub const FIELD_TOP: f32 = 64.;
    pub const FIELD_HEIGHT: f32 = 40.;
    pub const LIST_LABEL_TOP: f32 = 122.;
    pub const LIST_TOP: f32 = 142.;
    /// Space between the list and the buttons row.
    pub const LIST_BOTTOM: f32 = 76.;
    pub const ROW_HEIGHT: f32 = 44.;
    pub const ICON: f32 = 28.;
    pub const ICON_LEFT: f32 = 10.;
    pub const NAME_LEFT: f32 = 48.;
    pub const LIST_INSET: f32 = 4.;
}
pub mod app_strip {
    /// Pushes the strip down so its top gap matches the content padding (28).
    pub const TOP_SHIFT: f32 = 6.0;
    /// Width of a scroll arrow button.
    pub const ARROW: f32 = 28.;
    pub const ARROW_ICON: f32 = 16.;
    pub const GAP: f32 = 6.;
    /// Room between the last control and the card's right edge.
    pub const PAD: f32 = 12.;
    /// Extra pill width an app reserves for its remove button.
    pub const CLOSE_ROOM: f32 = 30.;
    pub const CLOSE: f32 = 18.;
    pub const CLOSE_ICON: f32 = 12.;
    /// Remove button center, from the pill's right edge.
    pub const CLOSE_INSET: f32 = 17.;
    pub const ADD_PAD_LEFT: f32 = 12.;
    pub const ADD_ICON: f32 = 16.;
    pub const ADD_GAP: f32 = 8.;
    pub const ADD_PAD_RIGHT: f32 = 16.;
    pub const ADD_TEXT_Y: f32 = 9.;
    /// Active-profile badge on a pill's top right corner.
    pub const BADGE: f32 = 4.;
    pub const BADGE_RING: f32 = 2.;
    pub const BADGE_X: f32 = 12.;
    pub const BADGE_Y: f32 = 4.;
    /// App icon at the start of a pill.
    pub const PILL_ICON: f32 = 20.;
    /// Arrow click scroll distance.
    pub const STEP: f32 = 220.;
    pub const SCROLL_TIME: f32 = 0.18;
}

/// The shared scroll thumb (no track), the same on every scrolling surface.
pub mod scroll {
    use egui::Color32;
    pub const WIDTH: f32 = 4.0;
    pub const INSET: f32 = 4.0;
    pub const MIN_THUMB: f32 = 24.0;
    /// Extra grab width on each side of the thin thumb.
    pub const HIT_PAD: f32 = 4.0;
    pub const RADIUS: u8 = 2;
    pub const THUMB: Color32 = super::FAINT;
    pub const ACTIVE: Color32 = super::MUTED;
}

pub mod interaction {
    use egui::Color32 as C;
    pub const RAISED_HOVER: C = C::from_gray(68);
    pub const WHITE_6: C = C::from_rgba_premultiplied(15, 15, 15, 15);
    pub const WHITE_8: C = C::from_rgba_premultiplied(20, 20, 20, 20);
    /// ACCENT at 12% and 18%, for selected pen callouts.
    pub const ACCENT_TINT: C = C::from_rgba_premultiplied(6, 16, 27, 31);
    pub const ACCENT_TINT_HOVER: C = C::from_rgba_premultiplied(10, 24, 41, 46);
    /// Text and border of accent-tinted buttons.
    pub const ACCENT_SOFT: C = C::from_rgb(143, 186, 240);
    pub const ACCENT_TINT_LINE: C = C::from_rgba_premultiplied(19, 46, 80, 89);
    pub const RING_HOVER: C = C::from_rgba_premultiplied(179, 179, 179, 179);
    pub const FRAME_HOVER: C = C::from_rgba_premultiplied(31, 31, 31, 31);
    pub const CLOSE: C = C::from_rgb(224, 27, 36);
    pub const MONITOR_HOVER: C = C::from_gray(138);
    pub const MONITOR_ON_HOVER: C = C::from_rgb(120, 174, 240);
    pub const SWITCH_TIME: f32 = 0.15;
    /// Pen actions panel sliding in, and the option cards sliding out.
    pub const PANEL_SLIDE_TIME: f32 = 0.3;
    pub const FAST_SWITCH_TIME: f32 = 0.12;
    pub const MONITOR_TIME: f32 = 0.15;
    pub const PRESSURE_TIME: f32 = 0.05;
    pub const OVERLAY_HIDE_TIME: f32 = 0.12;
    pub const OVERLAY_SHOW_TIME: f32 = 0.14;
    pub const OVERLAY_SCALE_TIME: f32 = 0.18;
    pub const OVERLAY_MAP_DELAY: f32 = 0.16;
    pub const OVERLAY_SCALE_START: f32 = 0.9;
    pub const EASE_IN: [f32; 4] = [0.42, 0., 1., 1.];
    pub const EASE_OUT: [f32; 4] = [0., 0., 0.58, 1.];
    pub const OVERLAY_EASE: [f32; 4] = [0.2, 0.8, 0.2, 1.];
    pub const FOCUS_WIDTH: f32 = 2.;
    pub const FOCUS_OFFSET: f32 = 3.;
}
