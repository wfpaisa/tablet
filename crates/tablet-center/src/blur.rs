//! Blurs what is behind the Glass window, through the Wayland protocol
//! ext-background-effect-v1 (GNOME 51 and later). GTK 3 does not offer it,
//! so it is spoken on GTK's own Wayland connection, on its own queue.

use std::{cell::RefCell, ffi::c_void, rc::Rc};

use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use wayland_backend::client::{Backend, ObjectId};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_compositor::WlCompositor, wl_region::WlRegion, wl_registry::WlRegistry, wl_surface::WlSurface},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1::{Capability, Event, ExtBackgroundEffectManagerV1},
    ext_background_effect_surface_v1::ExtBackgroundEffectSurfaceV1,
};

/// Same radius as .wf-window in frame.css, so the blur stops at the
/// window's rounded corners.
const RADIUS: i32 = 12;

/// Added to <html> while the compositor blurs; the page then lets more of
/// the background through.
const CLASS: &str = "blur-behind";

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut gtk::gdk::ffi::GdkDisplay) -> *mut c_void;
    fn gdk_wayland_window_get_wl_surface(window: *mut gtk::gdk::ffi::GdkWindow) -> *mut c_void;
}

#[derive(Default)]
struct State {
    can_blur: bool,
}

pub struct Blur {
    conn: Connection,
    queue: EventQueue<State>,
    state: State,
    compositor: WlCompositor,
    manager: ExtBackgroundEffectManagerV1,
    /// The effect and the surface it is on. GTK 3 makes a new wl_surface
    /// every time the window is shown again.
    effect: Option<(*mut c_void, ExtBackgroundEffectSurfaceV1)>,
    /// The size the region was made for; None while there is no region.
    size: Option<(i32, i32)>,
    /// Only Glass is blurred; Tablet's window is opaque.
    enabled: bool,
}

thread_local! {
    /// The main window's blur, for set_enabled (GTK's main thread only).
    static MAIN: RefCell<Option<(Rc<RefCell<Blur>>, gtk::ApplicationWindow)>> = const { RefCell::new(None) };
}

impl Blur {
    /// Connects to the compositor, or None when it cannot blur (X11, older
    /// GNOME, or blur turned off).
    pub fn new() -> Option<Self> {
        let display = gtk::gdk::Display::default()?;
        if display.type_().name() != "GdkWaylandDisplay" {
            return None;
        }
        let ptr = unsafe { gdk_wayland_display_get_wl_display(display.to_glib_none().0) };
        let conn = Connection::from_backend(unsafe { Backend::from_foreign_display(ptr.cast()) });
        let (globals, mut queue) = registry_queue_init::<State>(&conn).ok()?;
        let qh = queue.handle();
        let compositor = globals.bind::<WlCompositor, _, _>(&qh, 1..=1, ()).ok()?;
        let manager = globals.bind::<ExtBackgroundEffectManagerV1, _, _>(&qh, 1..=1, ()).ok()?;
        let mut state = State::default();
        // The capabilities come right after the bind.
        queue.roundtrip(&mut state).ok()?;
        state.can_blur.then_some(Blur { conn, queue, state, compositor, manager, effect: None, size: None, enabled: false })
    }

    pub fn class_script() -> String {
        format!("document.documentElement.classList.add('{CLASS}')")
    }

    /// Keeps the blur on the window while it lives, on while `enabled`.
    pub fn attach(mut self, window: &tauri::WebviewWindow, enabled: bool) {
        let (Ok(gtk_window), webview) = (window.gtk_window(), window.clone()) else { return };
        self.enabled = enabled;
        let blur = Rc::new(RefCell::new(self));
        MAIN.with(|m| *m.borrow_mut() = Some((blur.clone(), gtk_window.clone())));
        let update = move |w: &gtk::ApplicationWindow| {
            let Some(gdk_window) = w.window() else { return };
            let size = (gdk_window.width(), gdk_window.height());
            let had_blur = blur.borrow().state.can_blur;
            let changed = blur.borrow_mut().apply(&gdk_window, size);
            let can_blur = blur.borrow().state.can_blur;
            if can_blur != had_blur {
                let op = if can_blur { "add" } else { "remove" };
                let _ = webview.eval(format!("document.documentElement.classList.{op}('{CLASS}')"));
            }
            // The region takes effect on GTK's next commit.
            if changed {
                w.queue_draw();
            }
        };
        let on_map = update.clone();
        gtk_window.connect_map_event(move |w, _| {
            on_map(w);
            gtk::glib::Propagation::Proceed
        });
        let on_configure = update.clone();
        gtk_window.connect_configure_event(move |w, _| {
            on_configure(w);
            false
        });
        if gtk_window.is_mapped() {
            update(&gtk_window);
        }
    }

    /// Turns the main window's blur on or off, when switching interfaces.
    /// Call on GTK's main thread.
    pub fn set_enabled(enabled: bool) {
        MAIN.with(|m| {
            let Some((blur, window)) = &*m.borrow() else { return };
            blur.borrow_mut().enabled = enabled;
            if let Some(gdk_window) = window.window() {
                if blur.borrow_mut().apply(&gdk_window, (gdk_window.width(), gdk_window.height())) {
                    window.queue_draw();
                }
            }
        });
    }

    /// Puts the effect on the window's current surface with a region for
    /// its size. Returns whether anything was sent.
    fn apply(&mut self, window: &gtk::gdk::Window, size: (i32, i32)) -> bool {
        let _ = self.queue.dispatch_pending(&mut self.state);
        let surface_ptr = unsafe { gdk_wayland_window_get_wl_surface(window.to_glib_none().0) };
        if surface_ptr.is_null() || size.0 <= 0 || size.1 <= 0 {
            return false;
        }
        let qh = self.queue.handle();
        if !self.enabled {
            let Some((_, effect)) = self.effect.as_ref().filter(|_| self.size.is_some()) else { return false };
            effect.set_blur_region(None);
            self.size = None;
            let _ = self.conn.flush();
            return true;
        }
        let new_surface = self.effect.as_ref().map(|(p, _)| *p) != Some(surface_ptr);
        if new_surface {
            if let Some((_, old)) = self.effect.take() {
                old.destroy();
            }
            let Ok(id) = (unsafe { ObjectId::from_ptr(WlSurface::interface(), surface_ptr.cast()) }) else { return false };
            let Ok(surface) = WlSurface::from_id(&self.conn, id) else { return false };
            let effect = self.manager.get_background_effect(&surface, &qh, ());
            self.effect = Some((surface_ptr, effect));
        } else if Some(size) == self.size {
            return false;
        }
        self.size = Some(size);
        let region = self.compositor.create_region(&qh, ());
        for (x, y, w, h) in rounded_rect(size.0, size.1, RADIUS) {
            region.add(x, y, w, h);
        }
        let (_, effect) = self.effect.as_ref().unwrap();
        effect.set_blur_region(Some(&region));
        region.destroy();
        let _ = self.conn.flush();
        true
    }
}

/// A rounded rectangle as rows of rectangles: one per pixel row in the
/// corners, and one for the straight middle.
fn rounded_rect(w: i32, h: i32, r: i32) -> Vec<(i32, i32, i32, i32)> {
    let r = r.min(w / 2).min(h / 2);
    let mut rects = vec![(0, r, w, h - 2 * r)];
    for y in 0..r {
        let dy = f64::from(r - y) - 0.5;
        let inset = (f64::from(r) - (f64::from(r * r) - dy * dy).sqrt()).round() as i32;
        rects.push((inset, y, w - 2 * inset, 1));
        rects.push((inset, h - 1 - y, w - 2 * inset, 1));
    }
    rects
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(_: &mut Self, _: &WlRegistry, _: <WlRegistry as Proxy>::Event, _: &GlobalListContents, _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ExtBackgroundEffectManagerV1, ()> for State {
    fn event(state: &mut Self, _: &ExtBackgroundEffectManagerV1, event: Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let Event::Capabilities { flags: WEnum::Value(flags) } = event {
            state.can_blur = flags.contains(Capability::Blur);
        }
    }
}

wayland_client::delegate_noop!(State: ignore WlCompositor);
wayland_client::delegate_noop!(State: ignore WlRegion);
wayland_client::delegate_noop!(State: ignore ExtBackgroundEffectSurfaceV1);

#[cfg(test)]
mod tests {
    use super::rounded_rect;

    #[test]
    fn rounded_rect_covers_the_middle_and_trims_the_corners() {
        let rects = rounded_rect(100, 60, 12);
        assert!(rects.contains(&(0, 12, 100, 36)));
        let top = rects.iter().find(|r| r.1 == 0).unwrap();
        let bottom = rects.iter().find(|r| r.1 == 59).unwrap();
        assert!(top.0 > 0 && top.0 + top.2 == 100 - top.0);
        assert_eq!(top, &(bottom.0, 0, bottom.2, 1));
        assert!(rects.iter().find(|r| r.1 == 11).unwrap().0 == 0);
    }
}
