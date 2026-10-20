// Pen feedback and on-screen shortcuts drawn by GNOME Shell itself: no
// window, so nothing takes the focus or gets the map animation. tabletd
// calls it over D-Bus.

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

import {MenuHost} from './menus.js';

const BUS_NAME = 'dev.tabletcenter.Shell';
const OBJECT_PATH = '/dev/tabletcenter/Shell';
const INTERFACE = `
<node>
  <interface name="dev.tabletcenter.Shell">
    <!-- A ring at desktop point (x, y) that fills in over ms milliseconds. -->
    <method name="ShowHoldRing">
      <arg type="i" direction="in" name="x"/>
      <arg type="i" direction="in" name="y"/>
      <arg type="u" direction="in" name="ms"/>
    </method>
    <method name="HideHoldRing"/>
    <method name="ShowPrecision">
      <arg type="i" direction="in" name="x"/>
      <arg type="i" direction="in" name="y"/>
      <arg type="i" direction="in" name="w"/>
      <arg type="i" direction="in" name="h"/>
    </method>
    <method name="HidePrecision"/>
    <!-- An on-screen shortcut ("express_menu", "radial_menu",
         "app_shortcuts", "keypad"), at desktop point (x, y) when has_at.
         Fails when it cannot be shown, and tabletd opens its window. -->
    <method name="ShowMenu">
      <arg type="s" direction="in" name="name"/>
      <arg type="b" direction="in" name="has_at"/>
      <arg type="i" direction="in" name="x"/>
      <arg type="i" direction="in" name="y"/>
      <arg type="b" direction="in" name="centered"/>
    </method>
  </interface>
</node>`;

// Ring size and where it sits from the cursor's hot spot, like the
// spinner of GNOME's "busy" cursor.
const SIZE = 22;
const OFFSET = 14;
const LINE = 3;

class HoldRing {
    constructor() {
        this._progress = 0;
        this._area = new St.DrawingArea({width: SIZE, height: SIZE, reactive: false, visible: false});
        this._area.connect('repaint', area => this._paint(area));
        Main.layoutManager.uiGroup.add_child(this._area);
    }

    show(x, y, ms) {
        this._stop();
        const area = this._area;
        area.set_position(x + OFFSET, y + OFFSET);
        area.remove_all_transitions();
        area.opacity = 255;
        area.show();
        Main.layoutManager.uiGroup.set_child_above_sibling(area, null);
        this._progress = 0;
        this._timeline = new Clutter.Timeline({actor: area, duration: Math.max(ms, 1)});
        this._timeline.connect('new-frame', timeline => {
            this._progress = timeline.get_progress();
            area.queue_repaint();
        });
        // Full: a short fade says the middle button is now held.
        this._timeline.connect('completed', () => {
            this._progress = 1;
            area.queue_repaint();
            this.hide(150);
        });
        this._timeline.start();
    }

    // tabletd's hide: a full ring is already fading out on its own.
    release() {
        if (this._progress < 1)
            this.hide(100);
    }

    hide(fade = 0) {
        this._stop();
        const area = this._area;
        if (!fade) {
            area.hide();
            return;
        }
        area.ease({
            opacity: 0,
            duration: fade,
            mode: Clutter.AnimationMode.EASE_OUT_QUAD,
            onComplete: () => area.hide(),
        });
    }

    _stop() {
        this._timeline?.stop();
        this._timeline = null;
    }

    _paint(area) {
        const cr = area.get_context();
        const [w, h] = area.get_surface_size();
        const r = Math.min(w, h) / 2 - LINE;
        const start = -Math.PI / 2;
        cr.setLineWidth(LINE + 2);
        cr.setSourceRGBA(0, 0, 0, 0.45);
        cr.arc(w / 2, h / 2, r, 0, 2 * Math.PI);
        cr.stroke();
        cr.setLineWidth(LINE);
        cr.setSourceRGBA(1, 1, 1, 0.25);
        cr.arc(w / 2, h / 2, r, 0, 2 * Math.PI);
        cr.stroke();
        if (this._progress > 0) {
            cr.setSourceRGBA(1, 1, 1, 0.95);
            cr.arc(w / 2, h / 2, r, start, start + this._progress * 2 * Math.PI);
            cr.stroke();
        }
        cr.$dispose();
    }

    destroy() {
        this._stop();
        this._area.destroy();
    }
}

export default class TabletCenterExtension extends Extension {
    enable() {
        this._precision = new St.Widget({reactive: false, visible: false,
            style: 'border: 1px solid rgba(255,255,255,0.85); background-color: transparent;'});
        Main.layoutManager.uiGroup.add_child(this._precision);
        this._ring = new HoldRing();
        this._menus = new MenuHost();
        this._dbus = Gio.DBusExportedObject.wrapJSObject(INTERFACE, {
            ShowPrecision: (x, y, w, h) => {
                this._precision.set_position(x, y);
                this._precision.set_size(Math.max(w, 1), Math.max(h, 1));
                this._precision.show();
                Main.layoutManager.uiGroup.set_child_above_sibling(this._precision, null);
            },
            HidePrecision: () => this._precision.hide(),
            ShowHoldRing: (x, y, ms) => this._ring.show(x, y, ms),
            ShowMenu: (name, hasAt, x, y, centered) => this._menus.show(name, hasAt, x, y, centered),
            HideHoldRing: () => this._ring.release(),
        });
        this._dbus.export(Gio.DBus.session, OBJECT_PATH);
        this._name = Gio.bus_own_name_on_connection(
            Gio.DBus.session, BUS_NAME, Gio.BusNameOwnerFlags.NONE, null, null);
    }

    disable() {
        Gio.bus_unown_name(this._name);
        this._dbus.unexport();
        this._precision.destroy();
        this._precision = null;
        this._ring.destroy();
        this._menus.destroy();
        this._ring = this._menus = this._dbus = this._name = null;
    }
}
