// On-screen shortcuts drawn by GNOME Shell: the radial menus and grid
// panels Tablet Center otherwise opens in a window. Tablet Center writes the
// menus, already translated, to shell-menus.json; picking an entry asks
// tabletd to run its action, as the window does.

import Cairo from 'cairo';
import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Pango from 'gi://Pango';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {ACCENT, RADIAL as ring} from './theme.js';

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

// Separate circular buttons in clockwise order, with the original angular hit test.
export function paintRadialButton(cr, x, y, radius, scale, active) {
    const circle = (cx, cy, r) => cr.arc(cx, cy, r, 0, 2 * Math.PI);
    const glow = (cx, cy, r, blur, rgba) => {
        const gradient = new Cairo.RadialGradient(cx, cy, r, cx, cy, r + blur);
        gradient.addColorStopRGBA(0, ...rgba);
        gradient.addColorStopRGBA(1, ...rgba.slice(0, 3), 0);
        cr.setSource(gradient);
        circle(cx, cy, r + blur);
        cr.fill();
    };
    const blur = ring.shadowBlur * scale;
    glow(x, y + ring.shadowOffset * scale, radius - blur / 2, blur, ring.shadow);
    if (active) {
        const rim = (ring.ringGap + ring.ringWidth) * scale;
        glow(x, y, radius + rim, ring.haloBlur * scale, [...ACCENT, ring.haloOpacity]);
        const line = ring.ringWidth * scale;
        cr.setSourceRGBA(...ring.white);
        cr.setLineWidth(line);
        circle(x, y, radius + rim - line / 2);
        cr.stroke();
    }
    circle(x, y, radius);
    cr.setSourceRGBA(...(active ? [...ACCENT, 1] : ring.button));
    cr.fill();
    if (!active) {
        const line = ring.buttonBorderWidth * scale;
        cr.setSourceRGBA(...ring.buttonBorder);
        cr.setLineWidth(line);
        circle(x, y, radius - line / 2);
        cr.stroke();
    }
}

function radialMenu(menu, pick, close) {
    const d = menu.diameter;
    const scale = d / ring.diameter;
    const c = d / 2;
    const n = menu.items.length;
    const angle = i => i / n * 2 * Math.PI - Math.PI / 2;
    const direction = menu.items.map((_item, i) => [Math.cos(angle(i)), Math.sin(angle(i))]);
    const positions = direction.map(([x, y]) => [c + x * ring.orbit * scale, c + y * ring.orbit * scale]);
    // St rounds padding while allocating the text.
    const padX = Math.ceil(ring.labelPadX * scale);
    const padY = Math.ceil(ring.labelPadY * scale);
    const font = ring.labelFont * scale * menu.text / ring.labelDefaultTextSize;
    const labelHeight = Math.ceil(font * ring.labelLineHeight) + 2 * padY;
    const rim = (ring.ringGap + ring.ringWidth) * scale;
    const arrow = ring.labelArrowHeight * scale;
    const paintPad = Math.max(0, (ring.orbit + ring.activeRadius + ring.haloBlur) * scale + rim - c);
    const margin = Math.ceil(Math.max(paintPad, (ring.orbit + ring.activeRadius + ring.labelGap) * scale + rim + arrow + labelHeight - c));
    const actor = new St.Widget({width: d, height: d + 2 * margin, reactive: true});
    const area = new St.DrawingArea({x: -paintPad, width: d + 2 * paintPad, height: actor.height});
    actor.add_child(area);

    const icons = menu.items.map(item => {
        const name = item.icon || (disabled(item) ? 'action-unavailable-symbolic' : 'input-keyboard-symbolic');
        const icon = iconFor(name, ring.icon * scale);
        icon.style_class = 'tc-radial-icon';
        icon.opacity = disabled(item) ? Math.round(ring.maxOpacity * ring.disabledOpacity) : ring.maxOpacity;
        actor.add_child(icon);
        return icon;
    });
    function sizeIcon(i, size) {
        const [x, y] = positions[i];
        icons[i].icon_size = size;
        icons[i].set_position(x - size / 2, y + margin - size / 2);
    }
    icons.forEach((_icon, i) => sizeIcon(i, ring.icon * scale));
    const hubRadius = ring.hubRadius * scale;
    const hub = new St.Button({
        style_class: 'tc-radial-close',
        can_focus: false,
        width: 2 * hubRadius,
        height: 2 * hubRadius,
        accessible_name: menu.close,
        child: iconFor('window-close-symbolic', ring.hubIcon * scale),
    });
    hub.set_position(c - hubRadius, c + margin - hubRadius);
    hub.connect('clicked', close);
    actor.add_child(hub);

    const bubble = centeredLabel('', font);
    bubble.style_class = 'tc-radial-label';
    bubble.style = `font-size: ${font}px; border-radius: ${ring.labelRadius * scale}px;
        padding: ${padY}px ${padX}px;
        box-shadow: 0 ${ring.labelShadowOffset * scale}px ${ring.labelShadowBlur * scale}px rgba(0, 0, 0, ${ring.labelShadowOpacity});`;
    bubble.hide();
    actor.add_child(bubble);
    let hover = -1;
    const progress = menu.items.map(() => 0);
    const timelines = new Map();
    const radiusFor = i => (ring.buttonRadius + (ring.activeRadius - ring.buttonRadius) * progress[i]) * scale;

    // Move the pill with the expanding button and keep long labels within the menu width.
    let labelArrow;
    function positionLabel() {
        if (hover < 0)
            return;
        const [x, y] = positions[hover];
        const h = labelHeight;
        const [, naturalWidth] = bubble.clutter_text.get_preferred_width(-1);
        const width = Math.min(d, Math.ceil(naturalWidth) + 2 * padX);
        const left = Math.max(0, Math.min(x - width / 2, d - width));
        const above = hover < n / 4 || hover > 3 * n / 4;
        const sign = above ? -1 : 1;
        const tip = y + sign * (radiusFor(hover) + rim + ring.labelGap * scale);
        const edge = tip + sign * arrow;
        const arrowX = Math.max(left + padX, Math.min(x, left + width - padX));
        bubble.set_position(left, margin + (above ? edge - h : edge));
        bubble.set_size(width, h);
        labelArrow = [arrowX, tip, edge];
    }

    function animateButton(i, target) {
        if (i < 0)
            return;
        timelines.get(i)?.stop();
        const from = progress[i];
        const timeline = new Clutter.Timeline({actor, duration: ring.hoverMs});
        timeline.set_progress_mode(Clutter.AnimationMode.EASE_OUT_QUAD);
        timeline.connect('new-frame', () => {
            progress[i] = from + (target - from) * timeline.get_progress();
            positionLabel();
            area.queue_repaint();
        });
        timeline.connect('completed', () => timelines.delete(i));
        timelines.set(i, timeline);
        timeline.start();
    }

    function setHover(i) {
        if (i === hover)
            return;
        const previous = hover;
        hover = i;
        if (previous >= 0)
            sizeIcon(previous, ring.icon * scale);
        if (hover < 0) {
            bubble.hide();
        } else {
            sizeIcon(hover, ring.activeIcon * scale);
            bubble.text = menu.items[hover].label.replace(/\r?\n/g, ' ');
            bubble.show();
            positionLabel();
        }
        animateButton(previous, 0);
        animateButton(hover, 1);
        area.queue_repaint();
    }

    area.connect('repaint', () => {
        const cr = area.get_context();
        cr.translate(paintPad, margin);
        if (hover >= 0) {
            const [dx, dy] = direction[hover];
            const hub = ring.hubRadius * scale;
            const radius = radiusFor(hover);
            const inner = ring.beamHubWidth * scale / 2;
            const outer = radius * ring.beamButtonWidthRatio;
            // End on the button chord of the requested width.
            const end = ring.orbit * scale - Math.sqrt(radius * radius - outer * outer);
            const hx = c + dx * hub;
            const hy = c + dy * hub;
            const ex = c + dx * end;
            const ey = c + dy * end;
            const beam = new Cairo.LinearGradient(hx, hy, ex, ey);
            beam.addColorStopRGBA(0, ...ACCENT, ring.beamHubOpacity);
            beam.addColorStopRGBA(1, ...ACCENT, ring.beamButtonOpacity);
            cr.moveTo(c + dy * inner, c - dx * inner);
            cr.lineTo(c - dy * inner, c + dx * inner);
            cr.lineTo(hx - dy * inner, hy + dx * inner);
            cr.lineTo(ex - dy * outer, ey + dx * outer);
            cr.lineTo(ex + dy * outer, ey - dx * outer);
            cr.lineTo(hx + dy * inner, hy - dx * inner);
            cr.closePath();
            cr.setSource(beam);
            cr.fill();
        }

        const circle = (x, y, radius) => cr.arc(x, y, radius, 0, 2 * Math.PI);
        for (let i = 0; i < n; i++) {
            if (i !== hover)
                paintRadialButton(cr, ...positions[i], radiusFor(i), scale, false);
        }
        if (hover >= 0) {
            paintRadialButton(cr, ...positions[hover], radiusFor(hover), scale, true);
            const [arrowX, tip, edge] = labelArrow;
            const halfArrow = ring.labelArrowWidth * scale / 2;
            cr.setSourceRGBA(...ring.white);
            cr.moveTo(arrowX, tip);
            const join = edge + Math.sign(edge - tip) * ring.labelArrowOverlap * scale;
            cr.lineTo(arrowX - halfArrow, edge);
            cr.lineTo(arrowX - halfArrow, join);
            cr.lineTo(arrowX + halfArrow, join);
            cr.lineTo(arrowX + halfArrow, edge);
            cr.closePath();
            cr.fill();
        }
        cr.setSourceRGBA(...ring.hub);
        circle(c, c, ring.hubRadius * scale);
        cr.fill();
        cr.$dispose();
    });

    const pointerAt = (sx, sy) => {
        const [ok, x, y] = actor.transform_stage_point(sx, sy);
        const dx = x - c;
        const dy = y - margin - c;
        const distance = Math.hypot(dx, dy);
        if (!ok || Math.abs(dx) > c || Math.abs(dy) > c)
            return {outside: true};
        if (distance < ring.hubRadius * scale)
            return {hub: true};
        const outer = c - ring.hitRimInset;
        if (n === 0 || distance < outer * ring.hitInner || distance > outer)
            return {};
        const turn = (Math.atan2(dy, dx) + Math.PI / 2 + 2 * Math.PI) % (2 * Math.PI);
        const index = Math.round(turn / (2 * Math.PI) * n) % n;
        return disabled(menu.items[index]) ? {} : {index};
    };
    const track = (_actor, event) => {
        const {index} = pointerAt(...event.get_coords());
        setHover(index ?? -1);
        return Clutter.EVENT_PROPAGATE;
    };
    actor.connect('motion-event', track);
    actor.connect('enter-event', track);
    actor.connect('leave-event', () => {
        setHover(-1);
        return Clutter.EVENT_PROPAGATE;
    });
    actor.connect('button-release-event', (_actor, event) => {
        const {outside, hub, index} = pointerAt(...event.get_coords());
        if (outside || hub)
            close();
        else if (index !== undefined)
            pick(menu.items[index]);
        return Clutter.EVENT_STOP;
    });
    actor.connect('destroy', () => timelines.forEach(timeline => timeline.stop()));
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
