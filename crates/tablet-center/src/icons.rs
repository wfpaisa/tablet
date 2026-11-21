//! App icons by role, embedded from `assets/icons/app` (one file per role).

/// Icon role for an action id from the classic catalog or shortcut items.
pub fn action(id: &str) -> &'static str {
    match id {
        "left_click" | "click" | "click_lock" | "pressure_hold" => "left-click",
        "right_click" | "right_click_lock" => "right-click",
        "middle_click" | "middle_click_lock" | "right_or_hold_middle" | "pen_mouse_toggle" => {
            "middle-click"
        }
        "double_click" => "double-click",
        "pan_scroll" | "pan_scroll_legacy" => "pan",
        "pan_zoom" | "zoom" => "zoom",
        "scroll" => "scroll",
        "wheel" => "wheel",
        "back" => "back",
        "forward" => "forward",
        "undo" => "undo",
        "redo" => "redo",
        "copy" => "copy",
        "paste" => "paste",
        "cut" => "cut",
        "save" => "save",
        "open" => "open",
        "select_all" => "select-all",
        "type" => "type",
        "modifier" => "modifier",
        "run" => "run",
        "disabled" => "disabled",
        "default" => "default",
        "mode_toggle" => "mode-toggle",
        "precision_mode" => "precision",
        "display_toggle" | "show_desktop" => "display",
        "overview" => "overview",
        "switch_application" => "switch-app",
        "notification_center" => "notifications",
        "express_menu" | "radial_menu" => "on-screen-menu",
        "app_shortcuts" => "app-shortcuts",
        "keypad" => "keypad",
        _ => "keystroke",
    }
}

/// Generates `svg()` from role names; each maps to `assets/icons/app/<role>.svg`.
macro_rules! roles {
    ($($role:literal),* $(,)?) => {
        /// SVG source of an icon role; unknown roles fall back to the keystroke icon.
        pub fn svg(role: &str) -> &'static str {
            match role {
                $($role => include_str!(concat!("../../../assets/icons/app/", $role, ".svg")),)*
                _ => include_str!("../../../assets/icons/app/keystroke.svg"),
            }
        }
    };
}

roles!(
    "add",
    "app-shortcuts",
    "back",
    "close",
    "copy",
    "cut",
    "default",
    "disabled",
    "display",
    "double-click",
    "forward",
    "gesture",
    "home",
    "info",
    "keypad",
    "keystroke",
    "left-click",
    "middle-click",
    "mode-arrow",
    "mode-mouse",
    "mode-pen",
    "mode-relative",
    "mode-toggle",
    "modifier",
    "nav-devices",
    "nav-display-toggle",
    "nav-express-keys",
    "nav-mapping",
    "nav-pen",
    "nav-settings",
    "nav-shortcuts",
    "nav-touch",
    "next",
    "notifications",
    "on-screen-menu",
    "open",
    "overview",
    "pan",
    "paste",
    "precision",
    "previous",
    "redo",
    "right-click",
    "run",
    "save",
    "scroll",
    "select-all",
    "select-check",
    "switch-app",
    "type",
    "undo",
    "usb",
    "wheel",
    "window-close",
    "window-maximize",
    "window-minimize",
    "zoom",
);

/// An icon drawn in white, ready to tint with `egui::Image::tint`.
pub fn source(role: &str) -> egui::ImageSource<'static> {
    egui::ImageSource::Bytes {
        uri: format!("bytes://icon-{role}.svg").into(),
        bytes: svg(role)
            .replace("currentColor", "#fff")
            .into_bytes()
            .into(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_role_file_is_embedded() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icons/app");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "svg") {
                let role = path.file_stem().unwrap().to_str().unwrap();
                let embedded = super::svg(role);
                let on_disk = std::fs::read_to_string(&path).unwrap();
                assert_eq!(embedded, on_disk, "{role} is not listed in roles!");
                assert!(embedded.contains("currentColor"), "{role} is not tintable");
            }
        }
    }
}
