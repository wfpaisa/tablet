//! Lucide line icons (vendored from lucide.dev, ISC).

/// Lucide glyph for an action id from the classic catalog or shortcut items.
pub fn action(id: &str) -> &'static str {
    match id {
        "left_click" | "click" | "click_lock" | "pressure_hold" => "mouse-left",
        "right_click" | "right_click_lock" => "mouse-right",
        "middle_click" | "middle_click_lock" | "right_or_hold_middle" => "mouse",
        "double_click" => "mouse-pointer-click",
        "pan_scroll" | "pan_scroll_legacy" => "move",
        "pan_zoom" | "zoom" => "zoom-in",
        "scroll" => "arrow-down-up",
        "wheel" => "rotate-cw",
        "back" => "arrow-left",
        "forward" => "arrow-right",
        "undo" => "undo-2",
        "redo" => "redo-2",
        "copy" => "copy",
        "paste" => "clipboard-paste",
        "cut" => "scissors",
        "save" => "save",
        "open" => "folder",
        "select_all" => "list-checks",
        "type" => "type",
        "modifier" => "command",
        "run" => "square-terminal",
        "disabled" => "ban",
        "default" => "rotate-ccw",
        "mode_toggle" => "refresh-cw",
        "precision_mode" => "crosshair",
        "display_toggle" | "show_desktop" => "monitor",
        "overview" => "layers",
        "switch_application" => "app-window",
        "notification_center" => "bell",
        "express_menu" | "radial_menu" => "circle-dot",
        "app_shortcuts" => "layout-grid",
        "keypad" => "grid-3x3",
        _ => "keyboard",
    }
}

/// SVG source of a vendored glyph; unknown names fall back to the keyboard.
pub fn svg(name: &str) -> &'static str {
    match name {
        "app-window" => include_str!("../../../assets/icons/lucide/app-window.svg"),
        "arrow-down-up" => include_str!("../../../assets/icons/lucide/arrow-down-up.svg"),
        "arrow-left" => include_str!("../../../assets/icons/lucide/arrow-left.svg"),
        "arrow-right" => include_str!("../../../assets/icons/lucide/arrow-right.svg"),
        "ban" => include_str!("../../../assets/icons/lucide/ban.svg"),
        "bell" => include_str!("../../../assets/icons/lucide/bell.svg"),
        "chevron-left" => include_str!("../../../assets/icons/lucide/chevron-left.svg"),
        "chevron-right" => include_str!("../../../assets/icons/lucide/chevron-right.svg"),
        "circle-dot" => include_str!("../../../assets/icons/lucide/circle-dot.svg"),
        "clipboard-paste" => include_str!("../../../assets/icons/lucide/clipboard-paste.svg"),
        "command" => include_str!("../../../assets/icons/lucide/command.svg"),
        "copy" => include_str!("../../../assets/icons/lucide/copy.svg"),
        "crosshair" => include_str!("../../../assets/icons/lucide/crosshair.svg"),
        "folder" => include_str!("../../../assets/icons/lucide/folder.svg"),
        "gauge" => include_str!("../../../assets/icons/lucide/gauge.svg"),
        "hand" => include_str!("../../../assets/icons/lucide/hand.svg"),
        "house" => include_str!("../../../assets/icons/lucide/house.svg"),
        "grid-3x3" => include_str!("../../../assets/icons/lucide/grid-3x3.svg"),
        "info" => include_str!("../../../assets/icons/lucide/info.svg"),
        "keyboard" => include_str!("../../../assets/icons/lucide/keyboard.svg"),
        "layers" => include_str!("../../../assets/icons/lucide/layers.svg"),
        "layout-grid" => include_str!("../../../assets/icons/lucide/layout-grid.svg"),
        "list-checks" => include_str!("../../../assets/icons/lucide/list-checks.svg"),
        "monitor" => include_str!("../../../assets/icons/lucide/monitor.svg"),
        "mouse" => include_str!("../../../assets/icons/lucide/mouse.svg"),
        "mouse-left" => include_str!("../../../assets/icons/lucide/mouse-left.svg"),
        "mouse-pointer-click" => {
            include_str!("../../../assets/icons/lucide/mouse-pointer-click.svg")
        }
        "mouse-right" => include_str!("../../../assets/icons/lucide/mouse-right.svg"),
        "move" => include_str!("../../../assets/icons/lucide/move.svg"),
        "pencil" => include_str!("../../../assets/icons/lucide/pencil.svg"),
        "plus" => include_str!("../../../assets/icons/lucide/plus.svg"),
        "redo-2" => include_str!("../../../assets/icons/lucide/redo-2.svg"),
        "refresh-cw" => include_str!("../../../assets/icons/lucide/refresh-cw.svg"),
        "rotate-ccw" => include_str!("../../../assets/icons/lucide/rotate-ccw.svg"),
        "rotate-cw" => include_str!("../../../assets/icons/lucide/rotate-cw.svg"),
        "save" => include_str!("../../../assets/icons/lucide/save.svg"),
        "scissors" => include_str!("../../../assets/icons/lucide/scissors.svg"),
        "search" => include_str!("../../../assets/icons/lucide/search.svg"),
        "settings" => include_str!("../../../assets/icons/lucide/settings.svg"),
        "square-terminal" => include_str!("../../../assets/icons/lucide/square-terminal.svg"),
        "tablet" => include_str!("../../../assets/icons/lucide/tablet.svg"),
        "type" => include_str!("../../../assets/icons/lucide/type.svg"),
        "undo-2" => include_str!("../../../assets/icons/lucide/undo-2.svg"),
        "usb" => include_str!("../../../assets/icons/lucide/usb.svg"),
        "x" => include_str!("../../../assets/icons/lucide/x.svg"),
        "zoom-in" => include_str!("../../../assets/icons/lucide/zoom-in.svg"),
        _ => include_str!("../../../assets/icons/lucide/keyboard.svg"),
    }
}

/// A glyph drawn in white, ready to tint with `egui::Image::tint`.
pub fn source(name: &str) -> egui::ImageSource<'static> {
    egui::ImageSource::Bytes {
        uri: format!("bytes://lucide-{name}.svg").into(),
        bytes: svg(name)
            .replace("currentColor", "#fff")
            .into_bytes()
            .into(),
    }
}
