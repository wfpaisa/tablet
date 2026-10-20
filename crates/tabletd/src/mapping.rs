//! Tablet-to-screen mapping done in the daemon. The compositor maps our
//! virtual pen to the whole desktop, so choosing one monitor (or trimming the
//! tablet area) remaps the rotated absolute coordinates on each output axis.

use tablet_config::{Display, MappingSettings, Orientation};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Axis {
    min: f64,
    max: f64,
    /// Used part of the tablet, as fractions of the axis.
    from: (f64, f64),
    /// Where it lands, as fractions of the desktop.
    to: (f64, f64),
}

impl Axis {
    fn full(min: i32, max: i32) -> Self {
        Self { min: min.into(), max: max.into(), from: (0.0, 1.0), to: (0.0, 1.0) }
    }

    fn apply(&self, n: f64) -> i32 {
        let span = self.max - self.min;
        let n = ((n - self.from.0) / (self.from.1 - self.from.0)).clamp(0.0, 1.0);
        let out = self.to.0 + n * (self.to.1 - self.to.0);
        (self.min + out * span).round() as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mapping {
    x: Axis,
    y: Axis,
    orientation: Orientation,
}

/// Physical tablet facts needed to build a mapping.
#[derive(Debug, Clone, Copy)]
pub struct Tablet {
    pub x_range: (i32, i32),
    pub y_range: (i32, i32),
    pub width_mm: f64,
    pub height_mm: f64,
}

impl Mapping {
    pub fn new(settings: &MappingSettings, displays: &[Display], tablet: Tablet) -> Self {
        let mut map = Self {
            orientation: settings.orientation,
            x: Axis::full(tablet.x_range.0, tablet.x_range.1),
            y: Axis::full(tablet.y_range.0, tablet.y_range.1),
        };
        let Some(desktop) = bounds(displays) else { return map };
        let target = settings
            .display
            .as_deref()
            .and_then(|id| displays.iter().find(|d| d.id == id))
            .map_or(desktop, |d| (d.x, d.y, d.width, d.height));

        map.x.to = ((target.0 - desktop.0) / desktop.2, (target.0 + target.2 - desktop.0) / desktop.2);
        map.y.to = ((target.1 - desktop.1) / desktop.3, (target.1 + target.3 - desktop.1) / desktop.3);

        if settings.force_proportions && tablet.width_mm > 0.0 && tablet.height_mm > 0.0 && target.3 > 0.0 {
            let tablet_aspect = if settings.orientation.is_portrait() {
                tablet.height_mm / tablet.width_mm
            } else {
                tablet.width_mm / tablet.height_mm
            };
            let screen_aspect = target.2 / target.3;
            if tablet_aspect > screen_aspect {
                map.x.from.1 = screen_aspect / tablet_aspect;
            } else {
                map.y.from.1 = tablet_aspect / screen_aspect;
            }
        }
        map
    }

    pub fn map(&self, x: i32, y: i32) -> (i32, i32) {
        let x = (f64::from(x) - self.x.min) / (self.x.max - self.x.min);
        let y = (f64::from(y) - self.y.min) / (self.y.max - self.y.min);
        let (u, v) = self.orientation.position(x, y);
        (self.x.apply(u), self.y.apply(v))
    }
}

/// Where a mapped pen position is on the desktop, in logical pixels.
pub fn desktop_point(displays: &[Display], tablet: Tablet, (x, y): (i32, i32)) -> Option<(f64, f64)> {
    let (x0, y0, w, h) = bounds(displays)?;
    let fraction = |v: i32, (min, max): (i32, i32)| f64::from(v - min) / f64::from((max - min).max(1));
    Some((x0 + fraction(x, tablet.x_range) * w, y0 + fraction(y, tablet.y_range) * h))
}

/// Whether a desktop point is within `margin` pixels of a screen edge with
/// no monitor beyond it, where the dock, hot corners and edge barriers live.
pub fn near_screen_edge(displays: &[Display], (x, y): (f64, f64), margin: f64) -> bool {
    let on_screen = |px: f64, py: f64| {
        displays.iter().any(|d| px >= d.x && px < d.x + d.width && py >= d.y && py < d.y + d.height)
    };
    let Some(d) = displays.iter().find(|d| x >= d.x && x < d.x + d.width && y >= d.y && y < d.y + d.height) else {
        return false;
    };
    let (left, right, top, bottom) = (d.x, d.x + d.width, d.y, d.y + d.height);
    (x - left < margin && !on_screen(left - 1.0, y))
        || (right - x <= margin && !on_screen(right, y))
        || (y - top < margin && !on_screen(x, top - 1.0))
        || (bottom - y <= margin && !on_screen(x, bottom))
}

/// Bounding box of all monitors: (x, y, width, height).
fn bounds(displays: &[Display]) -> Option<(f64, f64, f64, f64)> {
    let x0 = displays.iter().map(|d| d.x).reduce(f64::min)?;
    let y0 = displays.iter().map(|d| d.y).reduce(f64::min)?;
    let x1 = displays.iter().map(|d| d.x + d.width).reduce(f64::max)?;
    let y1 = displays.iter().map(|d| d.y + d.height).reduce(f64::max)?;
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLET: Tablet = Tablet { x_range: (0, 15200), y_range: (0, 9500), width_mm: 152.0, height_mm: 95.0 };

    fn display(id: &str, x: f64, w: f64, h: f64) -> Display {
        Display { id: id.into(), x, y: 0.0, width: w, height: h, ..Default::default() }
    }

    #[test]
    fn screen_edges_skip_shared_borders() {
        let displays = two_monitors();
        let right = displays[1].x + displays[1].width;
        let h = displays[0].height;
        assert!(near_screen_edge(&displays, (500.0, h - 5.0), 24.0));
        assert!(near_screen_edge(&displays, (right - 3.0, 400.0), 24.0));
        assert!(!near_screen_edge(&displays, (500.0, h / 2.0), 24.0));
        // Between the two monitors there is no edge to reach.
        assert!(!near_screen_edge(&displays, (displays[1].x - 3.0, 400.0), 24.0));
    }

    fn two_monitors() -> Vec<Display> {
        vec![display("left", 0.0, 1920.0, 1080.0), display("right", 1920.0, 1920.0, 1080.0)]
    }

    #[test]
    fn all_displays_is_identity() {
        let m = Mapping::new(&MappingSettings::default(), &two_monitors(), TABLET);
        assert_eq!(m.map(7600, 4750), (7600, 4750));
    }

    #[test]
    fn one_monitor_uses_its_half_of_the_desktop() {
        let settings = MappingSettings { display: Some("right".into()), ..Default::default() };
        let m = Mapping::new(&settings, &two_monitors(), TABLET);
        assert_eq!(m.map(0, 0).0, 7600, "tablet left edge -> right monitor's left edge");
        assert_eq!(m.map(15200, 0).0, 15200);
        assert_eq!(m.map(0, 9500).1, 9500);
    }

    #[test]
    fn force_proportions_trims_the_bottom_for_wide_screens() {
        // 16:10 tablet onto a 16:9 monitor: full width, top 90% of the height.
        let settings = MappingSettings { display: Some("left".into()), force_proportions: true, ..Default::default() };
        let m = Mapping::new(&settings, &two_monitors(), TABLET);
        assert_eq!(m.map(15200, 0).0, 7600);
        let used: f64 = 9500.0 * (152.0 / 95.0) / (1920.0 / 1080.0);
        assert_eq!(m.map(0, used.round() as i32).1, 9500, "bottom of the used area -> bottom of screen");
        assert_eq!(m.map(0, 9500).1, 9500, "trimmed strip clamps to the edge");
    }

    #[test]
    fn orientations_map_all_corners() {
        let tablet = Tablet { x_range: (100, 15300), y_range: (200, 9700), ..TABLET };
        let corners = [(100, 200), (15300, 200), (15300, 9700), (100, 9700)];
        for (orientation, order) in [
            (Orientation::Landscape, [0, 1, 2, 3]),
            (Orientation::Portrait, [1, 2, 3, 0]),
            (Orientation::LandscapeFlipped, [2, 3, 0, 1]),
            (Orientation::PortraitFlipped, [3, 0, 1, 2]),
        ] {
            let settings = MappingSettings { orientation, ..Default::default() };
            let m = Mapping::new(&settings, &two_monitors(), tablet);
            for (physical, expected) in corners.iter().zip(order) {
                assert_eq!(m.map(physical.0, physical.1), corners[expected], "{orientation:?}");
            }
        }
    }

    #[test]
    fn portrait_proportions_use_rotated_dimensions() {
        let settings = MappingSettings {
            orientation: Orientation::Portrait,
            force_proportions: true,
            ..Default::default()
        };
        let m = Mapping::new(&settings, &[display("screen", 0.0, 1000.0, 1600.0)], TABLET);
        assert_eq!(m.map(15200, 0), (15200, 9500));
        assert_eq!(m.y.from, (0.0, 1.0));
        assert_eq!(m.x.from, (0.0, 1.0));
        let m = Mapping::new(&settings, &[display("screen", 0.0, 1920.0, 1080.0)], TABLET);
        assert!((m.y.from.1 - (95.0 / 152.0) / (1920.0 / 1080.0)).abs() < 1e-10);
    }

    #[test]
    fn desktop_point_scales_to_the_layout() {
        let p = desktop_point(&two_monitors(), TABLET, (11400, 4750)).unwrap();
        assert_eq!(p, (2880.0, 540.0));
    }

    #[test]
    fn unknown_display_falls_back_to_all() {
        let settings = MappingSettings { display: Some("unplugged".into()), ..Default::default() };
        let m = Mapping::new(&settings, &two_monitors(), TABLET);
        assert_eq!(m.map(3800, 0).0, 3800);
    }
}
