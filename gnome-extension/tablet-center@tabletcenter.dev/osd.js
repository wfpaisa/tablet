// Mode notifications drawn above the desktop without taking input.

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {paintRadialButton} from './menus.js';
import {MODE_OSD as t, RADIAL as ring} from './theme.js';

const MODES = new Map([
    ['pen', ['Pen', 'mode-pen']],
    ['mouse', ['Mouse', 'mode-mouse']],
    ['relative', ['Relative', 'mode-relative']],
]);
const LANGUAGE_FILE = GLib.build_filenamev([GLib.get_user_config_dir(), 'tablet-center', 'language']);

function readText(path) {
    const [, bytes] = GLib.file_get_contents(path);
    return new TextDecoder().decode(bytes);
}

function loadLocale(path) {
    try {
        const language = readText(LANGUAGE_FILE).trim();
        if (/^[a-z]{2}(-[A-Z]{2})?$/.test(language))
            return JSON.parse(readText(GLib.build_filenamev([path, 'locales', `${language}.json`])));
    } catch {
        // No stored language, or an older installation.
    }
    return {};
}

function icon(path, name, size) {
    const file = Gio.File.new_for_path(GLib.build_filenamev([path, 'icons', `${name}.svg`]));
    return new St.Icon({gicon: new Gio.FileIcon({file}), icon_size: size, reactive: false});
}

function centerIcon(actor, x, y, size) {
    actor.set_size(size, size);
    actor.set_position(x - size / 2, y - size / 2);
}

export class ModeOsd {
    constructor(path) {
        this._path = path;
    }

    show(from, to) {
        if (from === to || !MODES.has(from) || !MODES.has(to))
            throw new Error('Invalid mode transition');

        this.destroy();
        const monitor = Main.layoutManager.currentMonitor ?? Main.layoutManager.primaryMonitor;
        if (!monitor)
            throw new Error('No monitor for mode notification');

        const mid = t.width / 2;
        const left = mid - t.buttonOffset;
        const right = mid + t.buttonOffset;
        const y = t.iconCenterY;
        const card = new St.Widget({
            width: t.width, height: t.height, opacity: 0, reactive: false,
        });
        this._card = card;
        card.add_child(new St.Widget({
            width: t.width, height: t.height, reactive: false,
            style: `background-color: ${t.fill}; border-radius: ${t.radius}px;
                border: ${ring.buttonBorderWidth}px solid ${t.border};`,
        }));
        card.set_position(
            monitor.x + (monitor.width - t.width) / 2,
            monitor.y + (monitor.height - t.height) / 2);
        card.set_pivot_point(0.5, 0.5);
        card.set_scale(t.startScale, t.startScale);

        const [fromText, fromGlyph] = MODES.get(from);
        const [toText, toGlyph] = MODES.get(to);
        const area = new St.DrawingArea({width: t.width, height: t.height, reactive: false});
        let pop = 1;
        area.connect('repaint', () => {
            const cr = area.get_context();
            try {
                paintRadialButton(cr, left, y, t.buttonRadius, 1, false);
                paintRadialButton(cr, right, y, t.buttonRadius * pop, pop, true);
            } finally {
                cr.$dispose();
            }
        });
        card.add_child(area);

        const previous = icon(this._path, fromGlyph, ring.activeIcon);
        previous.opacity = t.previousOpacity;
        centerIcon(previous, left, y, ring.activeIcon);
        const next = icon(this._path, toGlyph, ring.activeIcon);
        centerIcon(next, right, y, ring.activeIcon);
        next.set_pivot_point(0.5, 0.5);
        const arrow = icon(this._path, 'mode-arrow', t.arrowSize);
        centerIcon(arrow, mid - t.arrowTravel, y, t.arrowSize);
        for (const actor of [previous, next, arrow])
            card.add_child(actor);

        const locale = loadLocale(this._path);
        const translate = text => GLib.markup_escape_text(locale[text] ?? text, -1);
        const label = new St.Label({style: `font-family: Roboto, sans-serif; font-size: ${t.font}px; color: white;`});
        label.clutter_text.set_markup(`${translate(fromText)} → <b>${translate(toText)}</b>`);
        const caption = new St.Bin({
            child: label, width: t.width, height: t.labelHeight,
            y: t.labelCenterY - t.labelHeight / 2,
        });
        label.x_align = label.y_align = Clutter.ActorAlign.CENTER;
        card.add_child(caption);

        Main.layoutManager.uiGroup.add_child(card);
        const fadeAt = t.showMs + t.holdMs;
        const timeline = new Clutter.Timeline({actor: card, duration: fadeAt + t.hideMs});
        const frame = timeline.connect('new-frame', (_timeline, elapsed) => {
            const opening = Math.min(1, elapsed / t.showMs);
            const eased = 1 - (1 - opening) ** 2;
            const closing = Math.max(0, Math.min(1, (elapsed - fadeAt) / t.hideMs));
            const zoom = t.startScale + (1 - t.startScale) * eased;
            card.opacity = Math.round(ring.maxOpacity * eased * (1 - closing ** 2));
            card.set_scale(zoom, zoom);
            arrow.translation_x = t.arrowTravel * eased;
            const progress = Math.max(0, Math.min(1, (elapsed - t.showMs) / t.popMs));
            const scale = 1 + t.popScale * Math.sin(Math.PI * progress);
            if (scale !== pop) {
                pop = scale;
                next.set_scale(pop, pop);
                area.queue_repaint();
            }
        });
        const complete = timeline.connect('completed', () => card.destroy());
        card.connect('destroy', () => {
            timeline.disconnect(frame);
            timeline.disconnect(complete);
            timeline.stop();
            this._card = null;
        });
        timeline.start();
    }

    destroy() {
        this._card?.destroy();
    }
}
