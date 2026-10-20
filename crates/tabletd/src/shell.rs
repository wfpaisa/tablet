//! Feedback drawn by the Tablet Center GNOME Shell extension
//! (`gnome-extension/`), reached over D-Bus. Calls run on their own thread
//! so the pen loop never waits on the bus; without the extension they fail
//! quietly.

use std::{
    sync::mpsc::{self, Sender},
    thread,
};

pub enum Feedback {
    Precision { x: i32, y: i32, w: i32, h: i32 },
    HidePrecision,
    /// A ring at a desktop point that fills in over the given milliseconds.
    HoldRing { x: i32, y: i32, ms: u32 },
    Hide,
    /// An on-screen shortcut by name, at a desktop point if known; `shown`
    /// learns whether the extension took it.
    Menu { name: &'static str, at: Option<(i32, i32)>, centered: bool, shown: Sender<bool> },
}

const BUS_NAME: &str = "dev.tabletcenter.Shell";
const PATH: &str = "/dev/tabletcenter/Shell";

pub fn spawn() -> Sender<Feedback> {
    let (tx, rx) = mpsc::channel::<Feedback>();
    thread::spawn(move || {
        let Ok(conn) = zbus::blocking::Connection::session() else { return };
        for feedback in rx {
            let _ = match feedback {
                Feedback::HoldRing { x, y, ms } => {
                    conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowHoldRing", &(x, y, ms))
                }
                Feedback::Precision { x, y, w, h } => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowPrecision", &(x, y, w, h)),
                Feedback::HidePrecision => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "HidePrecision", &()),
                Feedback::Hide => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "HideHoldRing", &()),
                Feedback::Menu { name, at, centered, shown } => {
                    let (x, y) = at.unwrap_or((0, 0));
                    let args = (name, at.is_some(), x, y, centered);
                    let reply = conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowMenu", &args);
                    let _ = shown.send(reply.is_ok());
                    continue;
                }
            };
        }
    });
    tx
}
