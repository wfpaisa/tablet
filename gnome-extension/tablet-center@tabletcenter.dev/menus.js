// On-screen shortcuts drawn by GNOME Shell: the radial menus and grid
// panels Tablet Center otherwise opens in a window. Tablet Center writes the
// menus, already translated, to shell-menus.json; picking an entry asks
// tabletd to run its action, as the window does.

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Pango from 'gi://Pango';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';

const MENUS_FILE = GLib.build_filenamev([GLib.get_user_config_dir(), 'tablet-center', 'shell-menus.json']);
const SOCKET = GLib.build_filenamev([GLib.get_user_runtime_dir(), 'tabletd.sock']);

// Opening zooms in from 90 %, closing zooms back out.
const SHOW_MS = 140;
const HIDE_MS = 110;
const START_SCALE = 0.9;
// Keys go to the app underneath once the shell has let go of the input.
const RUN_DELAY_MS = 60;
const MARGIN = 8;
const COLUMNS = 4;
const GAP = 6;

function loadMenus() {
    const [, bytes] = GLib.file_get_contents(MENUS_FILE);
    return JSON.parse(new TextDecoder().decode(bytes));
}

// One request on tabletd's socket, the same Tablet Center sends.
function runAction(action) {
    const client = new Gio.SocketClient();
    client.connect_async(Gio.UnixSocketAddress.new(SOCKET), null, (source, result) => {
        let conn;
        try {
            conn = source.connect_finish(result);
            const line = `${JSON.stringify({cmd: 'run', action})}\n`;
            conn.get_output_stream().write_all(new TextEncoder().encode(line), null);
            // Read the answer so tabletd is not cut off mid-reply.
            conn.get_input_stream().read_bytes_async(4096, GLib.PRIORITY_DEFAULT, null, (input, res) => {
                try {
                    input.read_bytes_finish(res);
                } catch {}
                conn.close(null);
            });
        } catch (e) {
            conn?.close(null);
            console.error(`Tablet Center: tabletd did not take the action: ${e.message}`);
        }
    });
}

function iconFor(name, size) {
    return name ? new St.Icon({icon_name: name, icon_size: size, x_align: Clutter.ActorAlign.CENTER}) : null;
}

function centeredLabel(text, size) {
    const label = new St.Label({text, style: `font-size: ${size}px;`, x_align: Clutter.ActorAlign.CENTER});
    label.clutter_text.set_line_alignment(Pango.Alignment.CENTER);
    label.clutter_text.set_ellipsize(Pango.EllipsizeMode.END);
    return label;
}

const disabled = item => item.action?.type === 'disabled';

// A ring of slices, clockwise from the top, with a close button in the hub.
function radialMenu(menu, pick, close) {
    const d = menu.diameter;
    const R = d / 2 - 2;
    const r = R * 0.34;
    const n = menu.items.length;
    const angle = i => (i / n) * 2 * Math.PI - Math.PI / 2;
    let hover = -1;

    const actor = new St.Widget({width: d, height: d, reactive: true});
    const area = new St.DrawingArea({width: d, height: d, reactive: true});
    actor.add_child(area);

    area.connect('repaint', () => {
        const cr = area.get_context();
        const c = d / 2;
        menu.items.forEach((item, i) => {
            const a0 = angle(i - 0.5);
            const a1 = angle(i + 0.5);
            cr.newPath();
            cr.arc(c, c, R, a0, a1);
            cr.arcNegative(c, c, r, a1, a0);
            cr.closePath();
            if (i === hover && !disabled(item))
                cr.setSourceRGBA(0.208, 0.518, 0.894, 1);
            else
                cr.setSourceRGBA(0.14, 0.14, 0.14, 0.95);
            cr.fillPreserve();
            cr.setSourceRGBA(1, 1, 1, 0.08);
            cr.setLineWidth(1);
            cr.stroke();
        });
        cr.newPath();
        cr.arc(c, c, R, 0, 2 * Math.PI);
        cr.setSourceRGBA(1, 1, 1, 0.12);
        cr.setLineWidth(1.5);
        cr.stroke();
        cr.arc(c, c, r, 0, 2 * Math.PI);
        cr.setSourceRGBA(0.19, 0.19, 0.19, 1);
        cr.fillPreserve();
        cr.setSourceRGBA(1, 1, 1, 0.08);
        cr.setLineWidth(1);
        cr.stroke();
        cr.$dispose();
    });

    // Which slice a point falls in, or -1 in the hub or outside the ring.
    const sliceAt = (sx, sy) => {
        const [ok, x, y] = area.transform_stage_point(sx, sy);
        if (!ok)
            return -1;
        const dx = x - d / 2;
        const dy = y - d / 2;
        const dist = Math.hypot(dx, dy);
        if (dist < r || dist > R)
            return -1;
        const turn = (Math.atan2(dy, dx) + Math.PI / 2 + 2 * Math.PI) % (2 * Math.PI);
        return Math.round((turn / (2 * Math.PI)) * n) % n;
    };
    area.connect('motion-event', (_a, event) => {
        const i = sliceAt(...event.get_coords());
        if (i !== hover) {
            hover = i;
            area.queue_repaint();
        }
        return Clutter.EVENT_PROPAGATE;
    });
    area.connect('leave-event', () => {
        hover = -1;
        area.queue_repaint();
        return Clutter.EVENT_PROPAGATE;
    });
    area.connect('button-release-event', (_a, event) => {
        const i = sliceAt(...event.get_coords());
        if (i >= 0 && !disabled(menu.items[i]))
            pick(menu.items[i]);
        return Clutter.EVENT_STOP;
    });

    // Labels halfway between hub and rim; they let events through.
    const mid = (r + R) / 2;
    const width = Math.min(R - r, 2 * mid * Math.sin(Math.PI / n)) * 1.05;
    menu.items.forEach((item, i) => {
        const box = new St.BoxLayout({
            orientation: Clutter.Orientation.VERTICAL,
            width,
            style_class: `tc-slice-label${disabled(item) ? ' tc-off' : ''}`,
        });
        const icon = iconFor(item.icon, Math.round(menu.text * 1.6));
        if (icon)
            box.add_child(icon);
        box.add_child(centeredLabel(item.label, menu.text));
        // Centred on its spot once its height is known (it needs the stage).
        const cy = d / 2 + mid * Math.sin(angle(i));
        box.set_position(d / 2 + mid * Math.cos(angle(i)) - width / 2, cy);
        box.connect('notify::height', () => box.set_y(Math.round(cy - box.height / 2)));
        actor.add_child(box);
    });

    const hub = new St.Button({
        style_class: 'tc-hub',
        can_focus: false,
        width: 2 * (r - 6),
        height: 2 * (r - 6),
        accessible_name: menu.close,
        child: new St.Icon({icon_name: 'window-close-symbolic', icon_size: Math.round(r * 0.5)}),
    });
    hub.set_position(d / 2 - (r - 6), d / 2 - (r - 6));
    hub.connect('clicked', close);
    actor.add_child(hub);
    return actor;
}

// A titled panel of buttons, four to a row; the keypad's 0 spans two.
function gridPanel(menu, pick, close) {
    const panel = new St.BoxLayout({orientation: Clutter.Orientation.VERTICAL, style_class: 'tc-panel', reactive: true});
    const header = new St.BoxLayout({style_class: 'tc-panel-header'});
    header.add_child(new St.Label({text: menu.title, style_class: 'tc-panel-title', x_expand: true, y_align: Clutter.ActorAlign.CENTER}));
    const closeButton = new St.Button({
        style_class: 'tc-close',
        can_focus: false,
        accessible_name: menu.close,
        child: new St.Icon({icon_name: 'window-close-symbolic', icon_size: 14}),
    });
    closeButton.connect('clicked', close);
    header.add_child(closeButton);
    panel.add_child(header);

    const layout = new Clutter.GridLayout({column_spacing: GAP, row_spacing: GAP});
    const grid = new St.Widget({layout_manager: layout});
    const keypad = menu.kind === 'keypad';
    let column = 0;
    let row = 0;
    for (const item of menu.items) {
        const span = item.span ?? 1;
        if (column + span > COLUMNS) {
            column = 0;
            row++;
        }
        const content = new St.BoxLayout({orientation: Clutter.Orientation.VERTICAL, style_class: 'tc-cell-box', y_align: Clutter.ActorAlign.CENTER});
        const icon = keypad ? null : iconFor(item.icon, Math.round(menu.text * 1.8));
        if (icon)
            content.add_child(icon);
        content.add_child(centeredLabel(item.label, keypad ? Math.round(menu.text * 1.5) : menu.text));
        const cell = new St.Button({
            style_class: `tc-cell${disabled(item) ? ' tc-off' : ''}`,
            can_focus: false,
            reactive: !disabled(item),
            width: menu.cell * span + GAP * (span - 1),
            height: menu.cell,
            child: content,
        });
        cell.connect('clicked', () => pick(item));
        layout.attach(cell, column, row, span, 1);
        column += span;
    }
    panel.add_child(grid);
    return panel;
}

export class MenuHost {
    constructor() {
        this._open = null;
        this._runSource = 0;
    }

    // Throws when it cannot show the menu, so tabletd falls back to the
    // Tablet Center window.
    show(name, hasAt, x, y, centered) {
        if (this._open?.name === name) {
            this.close();
            return;
        }
        this.close(false);
        const menu = loadMenus()[name];
        if (!menu)
            throw new Error(`no menu ${name}`);

        const pick = item => {
            this.close();
            if (this._runSource)
                GLib.Source.remove(this._runSource);
            this._runSource = GLib.timeout_add(GLib.PRIORITY_DEFAULT, RUN_DELAY_MS, () => {
                this._runSource = 0;
                runAction(item.action);
                return GLib.SOURCE_REMOVE;
            });
        };
        const close = () => this.close();
        const actor = menu.kind === 'radial' ? radialMenu(menu, pick, close) : gridPanel(menu, pick, close);

        // A full-stage backdrop: a click outside the menu closes it.
        const backdrop = new St.Widget({reactive: true, x: 0, y: 0, width: global.stage.width, height: global.stage.height});
        backdrop.add_child(actor);
        Main.layoutManager.uiGroup.add_child(backdrop);
        // The menu stops its own clicks, so only those outside get here.
        actor.connect('button-release-event', () => Clutter.EVENT_STOP);
        backdrop.connect('button-release-event', () => {
            this.close();
            return Clutter.EVENT_STOP;
        });
        backdrop.connect('key-press-event', (_b, event) => {
            if (event.get_key_symbol() === Clutter.KEY_Escape)
                this.close();
            return Clutter.EVENT_STOP;
        });

        const grab = Main.pushModal(backdrop);
        // Another grab (a menu, the overview) may keep the pointer.
        if (grab.get_seat_state && (grab.get_seat_state() & Clutter.GrabState.POINTER) === 0) {
            Main.popModal(grab);
            backdrop.destroy();
            throw new Error('could not grab the pointer');
        }
        backdrop.grab_key_focus();
        this._place(actor, menu, hasAt, x, y, centered);

        actor.set_pivot_point(0.5, 0.5);
        actor.opacity = 0;
        actor.scale_x = actor.scale_y = START_SCALE;
        actor.ease({opacity: 255, scale_x: 1, scale_y: 1, duration: SHOW_MS, mode: Clutter.AnimationMode.EASE_OUT_QUAD});
        this._open = {name, backdrop, actor, grab};
    }

    // At the cursor when the menu says so, otherwise centred on its
    // monitor; always kept inside the work area. The shell's own pointer is
    // used, since tabletd cannot know it in mouse mode.
    _place(actor, menu, hasAt, x, y, centered) {
        const atCursor = menu.at_cursor && !centered;
        if (atCursor) {
            [x, y] = global.get_pointer();
            hasAt = true;
        }
        const monitors = Main.layoutManager.monitors;
        let index = hasAt
            ? monitors.findIndex(m => x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height)
            : -1;
        if (index < 0)
            index = Main.layoutManager.currentMonitor?.index ?? Main.layoutManager.primaryIndex;
        const area = Main.layoutManager.getWorkAreaForMonitor(index);
        const [, width] = actor.get_preferred_width(-1);
        const [, height] = actor.get_preferred_height(width);
        let [cx, cy] = atCursor
            ? [x, y]
            : [area.x + area.width / 2, area.y + area.height / 2];
        const clamp = (v, size, lo, span) => Math.min(Math.max(v, lo + size / 2 + MARGIN), lo + span - size / 2 - MARGIN);
        cx = clamp(cx, width, area.x, area.width);
        cy = clamp(cy, height, area.y, area.height);
        actor.set_position(Math.round(cx - width / 2), Math.round(cy - height / 2));
    }

    close(animate = true) {
        const open = this._open;
        if (!open)
            return;
        this._open = null;
        // Input goes back to the desktop at once; the zoom-out only shows.
        Main.popModal(open.grab);
        open.backdrop.reactive = false;
        if (!animate) {
            open.backdrop.destroy();
            return;
        }
        open.actor.ease({
            opacity: 0,
            scale_x: START_SCALE,
            scale_y: START_SCALE,
            duration: HIDE_MS,
            mode: Clutter.AnimationMode.EASE_IN_QUAD,
            onComplete: () => open.backdrop.destroy(),
        });
    }

    destroy() {
        if (this._runSource)
            GLib.Source.remove(this._runSource);
        this._runSource = 0;
        this.close(false);
    }
}
