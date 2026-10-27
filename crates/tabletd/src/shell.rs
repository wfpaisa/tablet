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
        // Connected on demand and dropped after a failed call: tabletd may
        // start before the session bus is usable, and giving up then would
        // silence the ring and menus until tabletd restarts.
        let mut conn: Option<zbus::blocking::Connection> = None;
        for feedback in rx {
            if conn.is_none() {
                conn = zbus::blocking::Connection::session().ok();
            }
            let reply = match conn.as_ref() {
                None => Err(zbus::Error::Failure("no session bus".into())),
                Some(conn) => match &feedback {
                    Feedback::HoldRing { x, y, ms } => {
                        conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowHoldRing", &(x, y, ms))
                    }
                    Feedback::Precision { x, y, w, h } => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowPrecision", &(x, y, w, h)),
                    Feedback::HidePrecision => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "HidePrecision", &()),
                    Feedback::Hide => conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "HideHoldRing", &()),
                    Feedback::Menu { name, at, centered, .. } => {
                        let (x, y) = at.unwrap_or((0, 0));
                        conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ShowMenu", &(*name, at.is_some(), x, y, *centered))
                    }
                },
            };
            if reply.is_err() {
                conn = None;
            }
            if let Feedback::Menu { shown, .. } = feedback {
                let _ = shown.send(reply.is_ok());
            }
        }
    });
    tx
}

pub fn running_apps() -> zbus::Result<Vec<tablet_config::AppInfo>> {
    let conn = zbus::blocking::Connection::session()?;
    let reply = conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "ListRunningApps", &())?;
    let apps: Vec<(String, String, String)> = reply.body().deserialize()?;
    Ok(apps.into_iter().map(|(id, name, icon)| tablet_config::AppInfo {
        id, name, icon: if icon.is_empty() { None } else { Some(icon) },
    }).collect())
}

/// Follows the focused application. Only our own signal is matched (a
/// catch-all rule would wake us for every broadcast on the session bus); the
/// extension emits the current focus right after it starts, so a restarted
/// extension resyncs without watching the name owner.
pub fn watch_focus(mut changed: impl FnMut(Option<String>) + Send + 'static) {
    thread::spawn(move || loop {
        let mut track = || -> zbus::Result<()> {
            let conn = zbus::blocking::Connection::session()?;
            let rule = zbus::MatchRule::builder()
                .msg_type(zbus::message::Type::Signal)
                .interface(BUS_NAME)?
                .member("FocusedAppChanged")?
                .build();
            let messages = zbus::blocking::MessageIterator::for_match_rule(rule, &conn, Some(64))?;
            // Subscribed first, so a change between the two is not lost.
            if let Ok(reply) = conn.call_method(Some(BUS_NAME), PATH, Some(BUS_NAME), "GetFocusedApp", &()) {
                if let Ok((id, _)) = reply.body().deserialize::<(String, String)>() {
                    changed((!id.is_empty()).then_some(id));
                }
            }
            for message in messages {
                if let Ok((id, _)) = message?.body().deserialize::<(String, String)>() {
                    changed((!id.is_empty()).then_some(id));
                }
            }
            Ok(())
        };
        let _ = track();
        thread::sleep(std::time::Duration::from_secs(2));
    });
}
