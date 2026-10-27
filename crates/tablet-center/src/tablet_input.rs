//! Wayland tablet-v2 pen input for the main window (winit has no tablet support).
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use wayland_client::{
    backend::{Backend, ObjectId},
    event_created_child,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_registry, wl_seat::WlSeat, wl_surface::WlSurface},
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::wp::cursor_shape::v1::client::{
    wp_cursor_shape_device_v1::{Shape, WpCursorShapeDeviceV1},
    wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
};
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2,
    zwp_tablet_pad_group_v2::{self, ZwpTabletPadGroupV2},
    zwp_tablet_pad_ring_v2::ZwpTabletPadRingV2,
    zwp_tablet_pad_strip_v2::ZwpTabletPadStripV2,
    zwp_tablet_pad_v2::{self, ZwpTabletPadV2},
    zwp_tablet_seat_v2::{self, ZwpTabletSeatV2},
    zwp_tablet_tool_v2::{self, ButtonState, ZwpTabletToolV2},
    zwp_tablet_v2::ZwpTabletV2,
};
use wayland_protocols::xdg::shell::client::xdg_toplevel::{ResizeEdge, XdgToplevel};

const BTN_STYLUS: u32 = 0x14b;
const BTN_STYLUS2: u32 = 0x14c;

/// Pen events in logical surface pixels, converted to points in the input hook.
enum PenEvent {
    Move(egui::Pos2),
    Button(egui::PointerButton, bool),
    /// Tip pressure, 0 to 1.
    Pressure(f32),
    Gone,
}

struct State {
    surface: ObjectId,
    shapes: Option<WpCursorShapeManagerV1>,
    events: Arc<Mutex<Vec<PenEvent>>>,
    /// Serial of the last tip press, which window move and resize need.
    serial: Arc<AtomicU32>,
    ctx: egui::Context,
    /// Tools currently over the main window, with their cursor device.
    inside: Vec<(ObjectId, Option<WpCursorShapeDeviceV1>)>,
}

pub struct TabletInput {
    events: Arc<Mutex<Vec<PenEvent>>>,
    /// Last pen position in points, where button events land.
    pos: egui::Pos2,
    conn: Connection,
    seat: WlSeat,
    serial: Arc<AtomicU32>,
    /// winit's toplevel, found on the first frame; it moves the window for the pen.
    toplevel: Option<XdgToplevel>,
}

/// Set while the pen tip is down, so window grabs go through [`TabletInput`].
fn pen_down_id() -> egui::Id {
    egui::Id::new("tablet-pen-down")
}
/// Window move or resize asked while the pen held the tip down.
fn pen_grab_id() -> egui::Id {
    egui::Id::new("tablet-pen-grab")
}

/// Latest tip pressure from the compositor, read in step with pen motion.
fn pen_pressure_id() -> egui::Id {
    egui::Id::new("tablet-pen-pressure")
}

/// Tip pressure (0 to 1) while a tablet-v2 pen is over the window.
pub fn pressure(ctx: &egui::Context) -> Option<f32> {
    ctx.data(|d| d.get_temp::<f32>(pen_pressure_id()))
}

/// Starts a window move or resize from the pen when it is the active pointer.
/// Returns false when the mouse should use winit's own command.
pub fn grab(ctx: &egui::Context, command: &egui::ViewportCommand) -> bool {
    if !ctx.data(|d| d.get_temp::<bool>(pen_down_id()).unwrap_or(false)) {
        return false;
    }
    ctx.data_mut(|d| d.insert_temp(pen_grab_id(), Some(command.clone())));
    ctx.request_repaint();
    true
}

impl TabletInput {
    /// Returns None on X11 (XWayland already maps the pen) or without tablet-v2.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Option<Self> {
        let RawDisplayHandle::Wayland(display) = cc.display_handle().ok()?.as_raw() else {
            return None;
        };
        let RawWindowHandle::Wayland(surface) = cc.window_handle().ok()?.as_raw() else {
            return None;
        };
        // SAFETY: winit owns these handles and keeps them alive for the whole run.
        let conn = Connection::from_backend(unsafe {
            Backend::from_foreign_display(display.display.as_ptr().cast())
        });
        let surface =
            unsafe { ObjectId::from_ptr(WlSurface::interface(), surface.surface.as_ptr().cast()) }
                .ok()?;
        let (globals, mut queue) = registry_queue_init::<State>(&conn).ok()?;
        let qh = queue.handle();
        let seat: WlSeat = globals.bind(&qh, 1..=1, ()).ok()?;
        let manager: ZwpTabletManagerV2 = globals.bind(&qh, 1..=1, ()).ok()?;
        manager.get_tablet_seat(&seat, &qh, ());
        let events = Arc::new(Mutex::new(Vec::new()));
        let serial = Arc::new(AtomicU32::new(0));
        let mut state = State {
            surface,
            shapes: globals.bind(&qh, 1..=1, ()).ok(),
            events: events.clone(),
            serial: serial.clone(),
            ctx: cc.egui_ctx.clone(),
            inside: Vec::new(),
        };
        // A dedicated reader thread wakes egui; winit never sees these events.
        std::thread::Builder::new()
            .name("tablet-input".into())
            .spawn(move || while queue.blocking_dispatch(&mut state).is_ok() {})
            .ok()?;
        Some(Self {
            events,
            pos: egui::Pos2::ZERO,
            conn,
            seat,
            serial,
            toplevel: None,
        })
    }

    /// Runs a pending pen window grab on winit's toplevel.
    pub fn run_grab(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let Some(command) = ctx
            .data_mut(|d| d.remove_temp::<Option<egui::ViewportCommand>>(pen_grab_id()))
            .flatten()
        else {
            return;
        };
        if self.toplevel.is_none() {
            use winit::platform::wayland::WindowExtWayland;
            let ptr = frame.winit_window().and_then(|w| w.xdg_toplevel());
            // SAFETY: winit keeps its xdg_toplevel alive as long as the window.
            self.toplevel = ptr
                .and_then(|p| {
                    unsafe { ObjectId::from_ptr(XdgToplevel::interface(), p.as_ptr().cast()) }.ok()
                })
                .and_then(|id| XdgToplevel::from_id(&self.conn, id).ok());
        }
        let Some(toplevel) = &self.toplevel else {
            return;
        };
        let serial = self.serial.load(Ordering::Relaxed);
        match command {
            egui::ViewportCommand::StartDrag => toplevel._move(&self.seat, serial),
            egui::ViewportCommand::BeginResize(direction) => {
                use egui::viewport::ResizeDirection as D;
                let edge = match direction {
                    D::North => ResizeEdge::Top,
                    D::South => ResizeEdge::Bottom,
                    D::East => ResizeEdge::Right,
                    D::West => ResizeEdge::Left,
                    D::NorthEast => ResizeEdge::TopRight,
                    D::SouthEast => ResizeEdge::BottomRight,
                    D::NorthWest => ResizeEdge::TopLeft,
                    D::SouthWest => ResizeEdge::BottomLeft,
                };
                toplevel.resize(&self.seat, serial, edge);
            }
            _ => return,
        }
        let _ = self.conn.flush();
        // The compositor keeps the tip release, as it does for the mouse.
        ctx.data_mut(|d| d.insert_temp(pen_down_id(), false));
    }

    /// Appends queued pen events to the root viewport input as pointer events.
    pub fn inject(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if raw.viewport_id != egui::ViewportId::ROOT {
            return;
        }
        let Ok(mut events) = self.events.lock() else {
            return;
        };
        let zoom = ctx.zoom_factor();
        for event in events.drain(..) {
            raw.events.push(match event {
                PenEvent::Move(p) => {
                    self.pos = egui::pos2(p.x / zoom, p.y / zoom);
                    egui::Event::PointerMoved(self.pos)
                }
                PenEvent::Button(button, pressed) => {
                    if button == egui::PointerButton::Primary {
                        ctx.data_mut(|d| d.insert_temp(pen_down_id(), pressed));
                    }
                    egui::Event::PointerButton {
                        pos: self.pos,
                        button,
                        pressed,
                        modifiers: ctx.input(|i| i.modifiers),
                    }
                }
                PenEvent::Pressure(p) => {
                    ctx.data_mut(|d| d.insert_temp(pen_pressure_id(), p));
                    continue;
                }
                PenEvent::Gone => {
                    ctx.data_mut(|d| {
                        d.insert_temp(pen_down_id(), false);
                        d.remove_temp::<f32>(pen_pressure_id());
                    });
                    egui::Event::PointerGone
                }
            });
        }
    }
}

impl State {
    fn push(&self, event: PenEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
        self.ctx.request_repaint();
    }
    fn is_inside(&self, tool: &ZwpTabletToolV2) -> bool {
        self.inside.iter().any(|(id, _)| *id == tool.id())
    }
}

impl Dispatch<ZwpTabletToolV2, ()> for State {
    fn event(
        state: &mut Self,
        tool: &ZwpTabletToolV2,
        event: zwp_tablet_tool_v2::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use zwp_tablet_tool_v2::Event;
        match event {
            Event::ProximityIn {
                serial, surface, ..
            } => {
                if surface.id() != state.surface {
                    return;
                }
                // Without a cursor the compositor shows none while the pen hovers.
                let device = state.shapes.as_ref().map(|m| {
                    let device = m.get_tablet_tool_v2(tool, qh, ());
                    device.set_shape(serial, Shape::Default);
                    device
                });
                state.inside.push((tool.id(), device));
            }
            Event::ProximityOut => {
                if let Some(at) = state.inside.iter().position(|(id, _)| *id == tool.id()) {
                    if let (_, Some(device)) = state.inside.remove(at) {
                        device.destroy();
                    }
                    state.push(PenEvent::Gone);
                }
            }
            _ if !state.is_inside(tool) => {}
            Event::Motion { x, y } => {
                state.push(PenEvent::Move(egui::pos2(x as f32, y as f32)));
            }
            Event::Pressure { pressure } => {
                state.push(PenEvent::Pressure(pressure as f32 / 65535.));
            }
            Event::Down { serial } => {
                state.serial.store(serial, Ordering::Relaxed);
                state.push(PenEvent::Button(egui::PointerButton::Primary, true));
            }
            Event::Up => state.push(PenEvent::Button(egui::PointerButton::Primary, false)),
            Event::Button {
                button,
                state: pressed,
                ..
            } => {
                let button = match button {
                    BTN_STYLUS => egui::PointerButton::Secondary,
                    BTN_STYLUS2 => egui::PointerButton::Middle,
                    _ => return,
                };
                let pressed = matches!(pressed, WEnum::Value(ButtonState::Pressed));
                state.push(PenEvent::Button(button, pressed));
            }
            Event::Removed => {
                state.inside.retain(|(id, _)| *id != tool.id());
                tool.destroy();
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletSeatV2,
        _: zwp_tablet_seat_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
    event_created_child!(State, ZwpTabletSeatV2, [
        zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (ZwpTabletV2, ()),
        zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (ZwpTabletToolV2, ()),
        zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (ZwpTabletPadV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadV2,
        _: zwp_tablet_pad_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
    event_created_child!(State, ZwpTabletPadV2, [
        zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (ZwpTabletPadGroupV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadGroupV2, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpTabletPadGroupV2,
        _: zwp_tablet_pad_group_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
    event_created_child!(State, ZwpTabletPadGroupV2, [
        zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (ZwpTabletPadRingV2, ()),
        zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (ZwpTabletPadStripV2, ()),
    ]);
}

/// Objects whose events the pen mapping does not need.
macro_rules! ignore {
    ($($proxy:ty, $event:ty, $data:ty;)*) => {$(
        impl Dispatch<$proxy, $data> for State {
            fn event(_: &mut Self, _: &$proxy, _: $event, _: &$data, _: &Connection, _: &QueueHandle<Self>) {}
        }
    )*};
}
ignore! {
    wl_registry::WlRegistry, wl_registry::Event, GlobalListContents;
    WlSeat, wayland_client::protocol::wl_seat::Event, ();
    ZwpTabletManagerV2, wayland_protocols::wp::tablet::zv2::client::zwp_tablet_manager_v2::Event, ();
    ZwpTabletV2, wayland_protocols::wp::tablet::zv2::client::zwp_tablet_v2::Event, ();
    ZwpTabletPadRingV2, wayland_protocols::wp::tablet::zv2::client::zwp_tablet_pad_ring_v2::Event, ();
    ZwpTabletPadStripV2, wayland_protocols::wp::tablet::zv2::client::zwp_tablet_pad_strip_v2::Event, ();
    WpCursorShapeManagerV1, wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::Event, ();
    WpCursorShapeDeviceV1, wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::Event, ();
    XdgToplevel, wayland_protocols::xdg::shell::client::xdg_toplevel::Event, ();
}
