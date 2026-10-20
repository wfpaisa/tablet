// Tablet Center — UI. Talks to the Rust side through Tauri commands;
// the tablet is driven by tabletd.

const invoke = window.__TAURI__?.core?.invoke ?? (async (cmd) => {
  // Lets the page render in a plain browser while designing.
  if (cmd === "list_backups") return [];
  if (cmd === "autostart_enabled") return false;
  if (cmd === "get_config") return structuredClone(DEFAULTS);
  if (cmd === "status") return { ok: true, device: { name: "Wacom Intuos S Pen", vendor: 0x056a, product: 0x0374 }, pen: { pressure: 0 }, displays: [
    { id: "DEL|DELL S2721QS|HH2HM43", name: "Dell Inc. 27\"", x: 0, y: 0, width: 2560, height: 1440, primary: true },
    { id: "DEL|DELL U2412M|YMYH12BL49CL", name: "Dell Inc. 24\"", x: 2560, y: 97, width: 1920, height: 1200 },
  ] };
  return true;
});

const DEFAULTS = {
  pen: { mode: "pen", upper_button: "pan_scroll", lower_button: "right_click", tip_sensitivity: 4, hold_time: 2 },
  pan_scroll: { speed: 3, content_follows_pen: true },
  gestures: { legacy_speed: 3, scroll_speed: 3, wheel_speed: 3, zoom_speed: 3, pan_zoom_speed: 3 },
  mapping: { display: null, force_proportions: false, orientation: "landscape" },
  express_keys: [{ type: "disabled" }, { type: "disabled" }, { type: "disabled" }, { type: "disabled" }],
  shortcuts: defaultShortcuts(),
};

// Same as ShortcutSettings::default() in tablet-config.
function defaultShortcuts() {
  const keys = (label, ...keys) => ({ label: "", action: { type: "keystroke", keys, label } });
  const item = (type) => ({ label: "", action: { type } });
  const undo = keys("Undo", "KEY_LEFTCTRL", "KEY_Z");
  const redo = keys("Redo", "KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_Z");
  const copy = keys("Copy", "KEY_LEFTCTRL", "KEY_C");
  const paste = keys("Paste", "KEY_LEFTCTRL", "KEY_V");
  const save = keys("Save", "KEY_LEFTCTRL", "KEY_S");
  const cut = keys("Cut", "KEY_LEFTCTRL", "KEY_X");
  const all = keys("Select all", "KEY_LEFTCTRL", "KEY_A");
  const look = { size: 3, text_size: 3, at_cursor: true };
  return {
    express_menu: { ...look, items: [undo, redo, copy, paste, save, item("display_toggle"), item("right_click"), item("disabled")] },
    radial_menu: { ...look, items: [undo, redo, all, cut, copy, paste, item("keypad"), item("disabled")] },
    app_shortcuts: {
      ...look,
      items: [
        undo, redo, cut, copy, paste, all, save,
        keys("Open", "KEY_LEFTCTRL", "KEY_O"), keys("New", "KEY_LEFTCTRL", "KEY_N"), keys("Find", "KEY_LEFTCTRL", "KEY_F"),
        keys("Zoom in", "KEY_LEFTCTRL", "KEY_EQUAL"), keys("Zoom out", "KEY_LEFTCTRL", "KEY_MINUS"),
      ],
    },
    keypad: { ...look, items: [] },
    in_shell: false,
  };
}

// Models with artwork extracted from the macOS driver.
const MODELS = {
  0x0374: { name: "Intuos S", model: "CTL-4100", tablet: "intuos-s", pen: "intuos-pen" },
};
const FALLBACK_MODEL = { name: "Tablet", model: "", tablet: "intuos-s", pen: "intuos-pen" };

// Pen side-button actions, in menu order. Labels are translation keys.
const ACTIONS = [
  ["pan_scroll", "Pan/Scroll"],
  ["pan_scroll_legacy", "Pan/Scroll (Legacy)"],
  ["pan_zoom", "Pan/Zoom"],
  ["scroll", "Scroll"],
  ["wheel", "Wheel"],
  ["zoom", "Zoom"],
  ["left_click", "Left click"],
  ["right_click", "Right click"],
  ["middle_click", "Middle click"],
  ["right_or_hold_middle", "Right · hold Middle"],
  ["double_click", "Double click"],
  ["back", "Back"],
  ["forward", "Forward"],
  ["display_toggle", "Display toggle"],
  ["express_menu", "Express menu"],
  ["radial_menu", "Radial menu"],
  ["app_shortcuts", "App shortcuts"],
  ["keypad", "Keypad"],
  ["default", "Default"],
  ["disabled", "Disabled"],
];

// English wording that differs from the translation keys; the keys stay
// stable so ui/locales/*.json need not change when the copy is revised.
const EXTRA_EN = {
  "Devices manager": "Device manager",
  "Right · hold Middle": "Right click / hold for middle click",
  "Force proportions": "Keep proportions",
  "Default": "Restore defaults",
  "Tablet Center autostart": "Start Tablet Center when you sign in",
  "One cursor that changes over text and borders, like the mouse. The tip clicks; drawing apps get no pressure.": "Use the pen to move the mouse pointer and tap the tip to click. Pressure sensitivity is disabled in drawing apps.",
  "How long to hold the button before it becomes a middle click. Moving the pen while holding it starts the middle click at once.": "Choose how long to hold the button for a middle click. Moving the pen while holding the button also starts a middle click.",
  "Radial menus open around the pen, so reaching an action takes a short flick.": "Radial menus arrange actions around the cursor. Click an action to use it.",
  "Grid panels hold more actions, like your app's shortcuts or a numeric keypad.": "Use a grid for app shortcuts or a numeric keypad.",
  "Pen gestures scroll, pan and zoom while you hold the button and move the pen.": "Hold a pen button and move the pen to scroll, pan or zoom.",
  "Each key types once and closes the keypad.": "Select a key to enter it and close the keypad.",
  "Click an entry to choose its action. Assign this menu to a pen button or ExpressKey to open it.": "Click a menu item to choose its action. To open this menu, assign it to a pen button or ExpressKey.",
  "Hold the button and drag the tip to pan. Hold it with the pen in the air and move up to zoom in or down to zoom out.": "Hold the button and drag the tip across the tablet to pan. To zoom, keep holding the button and lift the tip above the tablet. Move up to zoom in or down to zoom out.",
  "Hold the button, touch the tablet and drag: the content moves with the pen.": "Hold the button and drag the tip across the tablet to move the content.",
  "Hold the button and touch the tablet: the farther you move the pen, the faster it scrolls, even when the pen rests.": "Hold the button and touch the tablet with the tip. Move away from that starting point to scroll. The farther you move, the faster it scrolls. Scrolling continues even when you stop moving the pen.",
  "Works like a mouse wheel: the farther you move the pen, the faster it turns.": "Scroll as you would with a mouse wheel. Move the pen farther from the starting point to scroll faster.",
  "Display toggle cycles your cursor between multiple screens and is only available for multi-screen setups.": "Switch the pen between individual screens and all screens. This requires at least two screens.",
  "Radial menus and grid panels are drawn by the Tablet Center GNOME Shell extension: they open at once and never take the focus. Without the extension they open in a window as before.": "Requires the Tablet Center GNOME Shell extension. Menus open without switching away from your app. Without the extension, menus open in a separate window.",
  "To prevent this, select \"Force proportions\" to use only a proportional area of the pen display. This may leave a portion of your pen display inactive, but ensures proportional pen movement.": "Match the tablet area to the screen proportions so pen movements keep the same shape on screen. Part of the tablet area may be unused."
};

let dict = { ...EXTRA_EN };
const t = (en) => dict[en] ?? en;

let config = structuredClone(DEFAULTS);
let status = { device: null, pen: null };
let pollTimer = 0;

const view = document.getElementById("view");
const $ = (sel, root = document) => root.querySelector(sel);
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
const icon = (name) => `<i class="icon" style="--icon: url(assets/icons/${name}.svg)"></i>`;

function model() {
  return MODELS[status.device?.product] ?? FALLBACK_MODEL;
}

// UI languages (ui/locales), in their own names.
// English is the source text, so it needs no dictionary.
const LANGUAGES = [
  ["en", "English"],
  ["es", "Español"],
  ["pt", "Português"],
  ["fr", "Français"],
  ["de", "Deutsch"],
  ["it", "Italiano"],
  ["nl", "Nederlands"],
  ["pl", "Polski"],
  ["ru", "Русский"],
  ["tr", "Türkçe"],
  ["ja", "日本語"],
  ["ko", "한국어"],
  ["zh-CN", "简体中文"],
  ["zh-TW", "繁體中文"],
];

// Interface styles: each is its own page built on this file's helpers.
// The last field: draws its own title bar (no system frame). Keep in step
// with UI_STYLES in tablet-center's main.rs.
const UI_STYLES = [
  ["classic", "Tablet", "index.html", false],
  ["glass", "Glass", "glass.html", true],
];

// The stored choice, or else the interface of the page being shown.
function uiStyle() {
  try {
    const stored = localStorage.getItem("ui");
    if (UI_STYLES.some(([id]) => id === stored)) return stored;
  } catch {}
  const page = location.pathname.split("/").pop() || "index.html";
  return UI_STYLES.find(([, , file]) => file === page)?.[0] ?? "classic";
}

// Remembers the choice (also for the next start, see set_ui_style) and
// opens that interface, with or without the system title bar.
async function setUiStyle(id) {
  try {
    localStorage.setItem("ui", id);
  } catch {}
  const [, , page, frameless] = UI_STYLES.find(([s]) => s === id) ?? UI_STYLES[0];
  await invoke("set_ui_style", { style: id }).catch(() => {});
  await appWindow()?.setDecorations(!frameless).catch(() => {});
  location.replace(page);
}

// ---------- custom title bar (frameless interfaces) ----------

const appWindow = () => window.__TAURI__?.window?.getCurrentWindow();

// Title bar with a drag area and window buttons; pages without the system
// frame put it at the top of their layout and call mountWindowFrame().
function windowFrame(title = "Tablet Center") {
  const button = (act, label, glyph) =>
    `<button class="wf-button" data-window="${act}" aria-label="${esc(t(label))}">${glyph}</button>`;
  return `
    <header class="window-frame" data-tauri-drag-region>
      <span class="wf-title" data-tauri-drag-region><img src="app-icon.png" alt="" data-tauri-drag-region>${esc(title)}</span>
      <div class="wf-buttons">
        ${button("minimize", "Minimize", '<svg viewBox="0 0 10 10"><path d="M2 5h6"/></svg>')}
        ${button("maximize", "Maximize", '<svg viewBox="0 0 10 10"><rect x="2" y="2" width="6" height="6" rx="1"/></svg>')}
        ${button("close", "Close", '<svg viewBox="0 0 10 10"><path d="M2.5 2.5l5 5M7.5 2.5l-5 5"/></svg>')}
      </div>
    </header>
    ${["North", "South", "East", "West", "NorthEast", "NorthWest", "SouthEast", "SouthWest"]
      .map((edge) => `<div class="wf-edge wf-${edge}" data-resize="${edge}"></div>`)
      .join("")}`;
}

function mountWindowFrame() {
  const win = appWindow();
  document.querySelectorAll("[data-window]").forEach((el) =>
    el.addEventListener("click", () => {
      const act = el.dataset.window;
      if (act === "minimize") win?.minimize();
      if (act === "maximize") win?.toggleMaximize();
      if (act === "close") win?.close();
    }),
  );
  // Without a system frame, the window resizes from these edge strips.
  document.querySelectorAll("[data-resize]").forEach((el) =>
    el.addEventListener("mousedown", (e) => {
      if (e.button === 0) win?.startResizeDragging(el.dataset.resize);
    }),
  );
  // A page opened before the choice was stored may still have a frame.
  win?.setDecorations(false).catch(() => {});
}

// Settings control shared by all interfaces.
function uiStyleSelect(className = "button") {
  const options = UI_STYLES.map(
    ([id, label]) => `<option value="${id}" ${uiStyle() === id ? "selected" : ""}>${esc(t(label))}</option>`,
  ).join("");
  return `<select id="ui-style" class="${className}">${options}</select>`;
}

function mountUiStyleSelect() {
  $("#ui-style")?.addEventListener("change", (e) => setUiStyle(e.target.value));
}

// Stored per user in the webview, since it only affects this window.
function language() {
  try {
    const stored = localStorage.getItem("language");
    if (LANGUAGES.some(([code]) => code === stored)) return stored;
  } catch {}
  return "en";
}

// ---------- persistence ----------

let saveTimer = 0;
function save() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(async () => {
    try {
      saveShellMenus();
      const applied = await invoke("save_config", { config });
      if (!applied) toast(t("Settings saved. Start tabletd to apply them."));
    } catch (e) {
      toast(String(e));
    }
  }, 150);
}

function toast(text) {
  const el = document.getElementById("toast");
  el.textContent = text;
  el.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => (el.hidden = true), 3500);
}

// ---------- shared pieces ----------

function slider(id, value, min, max, left, right) {
  const fill = ((value - min) / (max - min)) * 100;
  const ticks = Array.from({ length: max - min + 1 }, () => "<i></i>").join("");
  return `
    <input type="range" id="${id}" min="${min}" max="${max}" step="1" value="${value}" style="--fill:${fill}%">
    <div class="ticks">${ticks}</div>
    <div class="scale"><span>${esc(t(left))}</span><span>${esc(t(right))}</span></div>`;
}

function bindSlider(id, onChange) {
  const el = document.getElementById(id);
  el.addEventListener("input", () => {
    const v = Number(el.value);
    el.style.setProperty("--fill", `${((v - el.min) / (el.max - el.min)) * 100}%`);
    onChange(v);
  });
}

function deviceSidebar(active) {
  const m = model();
  const item = (route, ic, label) =>
    `<a class="side-item ${active === route ? "active" : ""}" href="#/devices/${route}">${icon(ic)}${esc(t(label))}</a>`;
  return `
    <aside class="sidebar">
      <a class="side-device" href="#/devices" style="color:inherit;text-decoration:none">
        <img src="assets/devices/${m.tablet}.png" alt="">${esc(m.name)}
      </a>
      ${item("pen", "sidebar-pen", "Pen")}
      ${item("expresskeys", "sidebar-expresskeys", "ExpressKeys")}
      ${item("mapping", "sidebar-mapping", "Mapping")}
      ${item("display-toggle", "sidebar-displaytoggle", "Display toggle")}
    </aside>`;
}

function later(text = "Coming to Linux in a later version.") {
  return `<div class="note">${icon("info")}${esc(t(text))}</div>`;
}

// ---------- pages ----------

function welcome() {
  return `
    <section class="welcome">
      <div class="band">
        <p style="font-size:26px">${esc(t("Welcome to"))}</p>
        <p class="brand">tablet <b>Center</b></p>
        <p style="font-size:17px">${esc(t("Connect a device to see its settings here."))}</p>
        ${status.error ? `<p style="color:#d8d8d8">${esc(t("The tablet driver is not responding."))} (tabletd)</p>` : ""}
      </div>
    </section>`;
}

const ORIENTATIONS = [
  ["landscape", "Landscape"],
  ["portrait", "Portrait"],
  ["landscape_flipped", "Landscape flipped"],
  ["portrait_flipped", "Portrait flipped"],
];

function orientationTurns() {
  return Math.max(0, ORIENTATIONS.findIndex(([value]) => value === config.mapping.orientation));
}

function orientationSelector() {
  return `<div class="orientation-selector" role="group" aria-label="${esc(t("Tablet orientation"))}">
    ${ORIENTATIONS.map(([value, label], turns) => `
      <button type="button" data-orientation="${value}" aria-label="${esc(t(label))}" title="${esc(t(label))}"
        aria-pressed="${turns === orientationTurns()}">
        <svg viewBox="0 0 48 48" aria-hidden="true"><g transform="rotate(${turns * 90} 24 24)">
          <rect x="7" y="12" width="34" height="24" rx="3" fill="none" stroke="currentColor" stroke-width="2"/>
          <path d="M11 13H37" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
        </g></svg>
      </button>`).join("")}
  </div>`;
}

function mountOrientation() {
  document.querySelectorAll("[data-orientation]").forEach((button) => {
    button.onclick = async () => {
      const value = button.dataset.orientation;
      if (value === (config.mapping.orientation ?? "landscape")) return;
      const label = ORIENTATIONS.find(([key]) => key === value)[1];
      if (!await settingsDialog(`Would you like to apply the orientation "${label}"?`, "OK")) return;
      config.mapping.orientation = value;
      save();
      render();
    };
  });
}

function devicesOverview() {
  const m = model();
  return `
    ${deviceSidebar("")}
    <section class="content">
      <div class="watermark">${esc(m.name)}</div>
      <h1 class="page-title">${esc(m.name)}</h1>
      <div class="page-sub">${esc(m.model)}</div>
      <div class="overview">
        <div class="tablet glow">
          <div class="tablet-art"><img style="transform:rotate(${orientationTurns() * 90}deg) scale(${orientationTurns() % 2 ? 0.79 : 1})" src="assets/devices/${m.tablet}.png" alt="${esc(m.name)}"></div>
          <div class="chips">
            <a class="chip" href="#/devices/expresskeys">${icon("sidebar-expresskeys")}${esc(t("ExpressKeys"))}</a>
            <a class="chip" href="#/devices/mapping">${icon("sidebar-mapping")}${esc(t("Mapping"))}</a>
            <a class="chip" href="#/devices/display-toggle">${icon("sidebar-displaytoggle")}${esc(t("Display toggle"))}</a>
          </div>
        </div>
        <div class="pen-figure">
          <img src="assets/devices/${m.pen}.png" alt="">
          <a class="chip" href="#/devices/pen">${icon("sidebar-pen")}${esc(t("Pen"))}</a>
        </div>
      </div>
      ${orientationSelector()}
    </section>`;
}

// Button positions on the Intuos pen artwork, as fractions of its height.
const PEN_SPOTS = { upper: 0.645, lower: 0.735, tip: 0.975 };

function penPage() {
  const m = model();
  const label = (key) => esc(t(ACTIONS.find(([k]) => k === config.pen[key])?.[1] ?? "Default"));
  const chosen = selected?.page === "pen" ? selected.key : null;
  const callout = (spot, key, text) => `
    <div class="callout ${key && key === chosen ? "selected" : ""}" data-spot="${spot}" style="top:${PEN_SPOTS[spot] * 100}%">
      ${key ? `<button data-button="${key}">${label(key)}</button>` : `<button disabled>${esc(t(text))}</button>`}
      <span class="lead">${key ? `<button class="ring" data-button="${key}" aria-label="${label(key)}"></button>` : `<span class="ring"></span>`}</span>
    </div>`;
  return `
    ${deviceSidebar("pen")}
    ${chosen ? actionsPanel(PEN_GROUPS, config.pen[chosen], config.pen[chosen] === "right_or_hold_middle" ? holdTimePanel() : "") : ""}
    <section class="content">
      <div class="pen-page">
        <div class="pen-callouts" title="${esc(t("Customize functions of the buttons on the device."))}">
          <img src="assets/devices/${m.pen}.png" alt="">
          ${callout("upper", "upper_button")}
          ${callout("lower", "lower_button")}
          ${callout("tip", null, "Click")}
        </div>
        <div class="pen-side">
          <p class="label">${esc(t("Test area"))}</p>
          <div class="test-area"><canvas id="test"></canvas></div>
          <div class="pressure"><span id="pressure-fill" style="width:0"></span></div>
          <div class="scale"><span>${esc(t("Click"))}</span><span>${esc(t("Maximum"))}</span></div>
          <div class="slider">
            <p class="label">${esc(t("Tip sensitivity"))}</p>
            ${slider("tip", config.pen.tip_sensitivity, 1, 7, "Soft", "Firm")}
          </div>
          <label class="toggle-row">
            <span class="switch"><input type="checkbox" id="mouse-mode" ${config.pen.mode === "mouse" ? "checked" : ""}><span></span></span>
            ${esc(t("Use the pen as a mouse"))}
          </label>
          <p class="gesture-hint">${esc(t(MOUSE_MODE_HINT))}</p>
        </div>
      </div>
      <button class="button corner" id="pen-default">${esc(t("Default"))}</button>
    </section>`;
}

const MOUSE_MODE_HINT = "One cursor that changes over text and borders, like the mouse. The tip clicks; drawing apps get no pressure.";

function mountPen() {
  layoutCallouts();
  window.addEventListener("resize", layoutCallouts);
  document.querySelectorAll("[data-button]").forEach((btn) =>
    btn.addEventListener("click", () => select({ page: "pen", key: btn.dataset.button })),
  );
  mountActions((value) => {
    config.pen[selected.key] = value;
    save();
    render();
  });
  bindSlider("tip", (v) => {
    config.pen.tip_sensitivity = v;
    save();
  });
  if ($("#hold-time"))
    bindSlider("hold-time", (v) => {
      config.pen.hold_time = v;
      save();
    });
  $("#mouse-mode").addEventListener("change", (e) => {
    config.pen.mode = e.target.checked ? "mouse" : "pen";
    save();
  });
  $("#pen-default").addEventListener("click", () => {
    config.pen = structuredClone(DEFAULTS.pen);
    save();
    render();
  });
  mountTestArea();
  startPolling(40, () => {
    const p = status.pen?.pressure ?? 0;
    const fill = $("#pressure-fill");
    if (fill) fill.style.width = `${Math.round(p * 100)}%`;
    // Light up the button being pressed, so users learn which is which.
    for (const key of ["upper_button", "lower_button"]) {
      document.querySelectorAll(`.callout [data-button="${key}"]`).forEach((el) =>
        el.closest(".callout").classList.toggle("held", Boolean(status.pen?.[key])),
      );
    }
  });
}

// Lead lines end on the pen's buttons whatever size the artwork renders at.
function layoutCallouts() {
  const box = $(".pen-callouts");
  const img = box && $("img", box);
  if (!img?.complete) return img?.addEventListener("load", layoutCallouts, { once: true });
  const art = img.getBoundingClientRect();
  const frame = box.getBoundingClientRect();
  const room = frame.width;
  for (const el of box.querySelectorAll(".callout")) {
    const onBody = el.dataset.spot === "tip" ? 0.49 : 0.4;
    const right = frame.right - (art.left + art.width * onBody);
    el.style.right = `${right}px`;
    // Shorter leads in a narrow window, so the label still fits.
    const lead = Math.min(el.dataset.spot === "tip" ? 150 : 140, room - right - 170);
    $(".lead", el).style.width = `${Math.max(lead, 30)}px`;
  }
}

// ---------- Actions panel ----------

// The control whose action is being chosen, e.g. { page: "pen", key:
// "upper_button" } or { page: "keys", index: 0 }. Cleared on navigation.
let selected = null;

// Action groups as in Tablet Center's Actions panel. Values are config
// actions; "preset:N" and "record" are ExpressKey keystrokes.
const GESTURE_VALUES = ["pan_scroll", "pan_scroll_legacy", "pan_zoom", "scroll", "wheel", "zoom"];
const CLICK_VALUES = ["left_click", "right_click", "middle_click", "double_click", "back", "forward"];
const PEN_GROUPS = [
  ["Recommended", ["default", "right_click", "middle_click", "right_or_hold_middle", "pan_scroll", "display_toggle", "disabled"]],
  ["Clicks", [...CLICK_VALUES, "right_or_hold_middle"]],
  ["Tablet", ["display_toggle"]],
  ["Radial menus", ["express_menu", "radial_menu"]],
  ["Grid panels", ["app_shortcuts", "keypad"]],
  ["Pen gestures", GESTURE_VALUES],
];
// A function because KEY_PRESETS is defined further down.
const keyGroups = () => [
  ["Recommended", ["disabled", "preset:0", "preset:1", "pan_scroll", "display_toggle", "record"]],
  ["Clicks", CLICK_VALUES],
  ["Keyboard", [...KEY_PRESETS.map((_, i) => `preset:${i}`), "record"]],
  ["Tablet", ["display_toggle"]],
  ["Radial menus", ["express_menu", "radial_menu"]],
  ["Grid panels", ["app_shortcuts", "keypad"]],
  ["Pen gestures", GESTURE_VALUES],
];
// On-screen shortcut entries run once, so they cannot hold a pen gesture.
const menuGroups = () => [
  ["Recommended", ["disabled", "preset:0", "preset:1", "display_toggle", "record"]],
  ["Clicks", CLICK_VALUES],
  ["Keyboard", [...KEY_PRESETS.map((_, i) => `preset:${i}`), "record"]],
  ["Tablet", ["display_toggle"]],
  ["Radial menus", ["express_menu", "radial_menu"]],
  ["Grid panels", ["app_shortcuts", "keypad"]],
];

function actionLabel(value) {
  if (value.startsWith("preset:")) return t(KEY_PRESETS[Number(value.slice(7))][0]);
  if (value === "record") return `${t("Keyboard shortcut")}…`;
  return t([...ACTIONS, ...KEY_ACTIONS].find(([v]) => v === value)?.[1] ?? value);
}

// Icon of each action, as Tablet Center shows it (assets/icons/action-*.svg).
const PRESET_ICONS = ["undo", "redo", "copy", "paste", "save"];
// Shortcut actions use the On-screen shortcuts sidebar icons.
const SHORTCUT_ICONS = {
  express_menu: "radial",
  radial_menu: "radial",
  app_shortcuts: "grid",
  keypad: "keypad",
  right_or_hold_middle: "action-middle_click",
};
function actionIcon(value) {
  if (value.startsWith("preset:")) return icon(`action-${PRESET_ICONS[Number(value.slice(7))] ?? "keyboard"}`);
  return icon(SHORTCUT_ICONS[value] ?? `action-${value}`);
}

// Above the actions of a "Right · hold Middle" pen button: how long the
// button waits (and the ring fills) before it becomes a middle click.
function holdTimePanel() {
  return `
    <div class="hold-time">
      <p class="label">${esc(t("Hold time"))}</p>
      ${slider("hold-time", config.pen.hold_time ?? 2, 1, 3, "Short", "Long")}
      <p class="hint">${esc(t(HOLD_TIME_HINT))}</p>
    </div>`;
}
const HOLD_TIME_HINT = "How long to hold the button before it becomes a middle click. Moving the pen while holding it starts the middle click at once.";

function actionsPanel(groups, current, extra = "") {
  const group = ([title, values], i) => {
    // Recommended starts open, and so does the group holding the current action.
    const open = i === 0 || (values.includes(current) && !groups[0][1].includes(current));
    const items = values
      .map((v) => `<button class="action ${v === current ? "active" : ""}" data-action="${v}">${actionIcon(v)}${esc(actionLabel(v))}</button>`)
      .join("");
    return `<details ${open ? "open" : ""}><summary>${esc(t(title))}</summary>${items}</details>`;
  };
  return `<aside class="actions"><h3>${esc(t("Actions"))}</h3>${extra}${groups.map(group).join("")}</aside>`;
}

function select(control) {
  const same = selected && JSON.stringify(selected) === JSON.stringify(control);
  selected = same ? null : control;
  render();
}

addEventListener("keydown", (e) => {
  if (e.key === "Escape" && selected) select(selected);
});

function mountActions(apply) {
  document.querySelectorAll("[data-action]").forEach((btn) =>
    btn.addEventListener("click", () => apply(btn.dataset.action)),
  );
}

// Draw with the pen to try the sensitivity curve; width follows pressure.
function mountTestArea() {
  const canvas = $("#test");
  const ctx = canvas.getContext("2d");
  const fit = () => {
    const r = canvas.getBoundingClientRect();
    canvas.width = r.width * devicePixelRatio;
    canvas.height = r.height * devicePixelRatio;
    ctx.scale(devicePixelRatio, devicePixelRatio);
    ctx.lineCap = "round";
    ctx.strokeStyle = "#e8e8e8";
  };
  fit();
  let last = null;
  canvas.addEventListener("pointerdown", (e) => {
    canvas.setPointerCapture(e.pointerId);
    last = [e.offsetX, e.offsetY];
  });
  canvas.addEventListener("pointermove", (e) => {
    if (!last) return;
    const pressure = e.pointerType === "pen" && e.pressure ? e.pressure : status.pen?.pressure || 0.5;
    ctx.lineWidth = 1 + pressure * 9;
    ctx.beginPath();
    ctx.moveTo(...last);
    ctx.lineTo(e.offsetX, e.offsetY);
    ctx.stroke();
    last = [e.offsetX, e.offsetY];
  });
  canvas.addEventListener("pointerup", () => (last = null));
  canvas.addEventListener("dblclick", fit);
}

// ExpressKey centres on the Intuos S artwork, as fractions of the image.
const KEY_SPOTS = [0.189, 0.395, 0.602, 0.808];
const KEY_ROW_Y = 0.15;
// Room above the tablet for the labels; matches .key-leads in style.css.
const LEAD_ROOM = 220;

// Ready-made shortcuts offered in the ExpressKey menu.
const KEY_PRESETS = [
  ["Undo", ["KEY_LEFTCTRL", "KEY_Z"]],
  ["Redo", ["KEY_LEFTCTRL", "KEY_LEFTSHIFT", "KEY_Z"]],
  ["Copy", ["KEY_LEFTCTRL", "KEY_C"]],
  ["Paste", ["KEY_LEFTCTRL", "KEY_V"]],
  ["Save", ["KEY_LEFTCTRL", "KEY_S"]],
];

const KEY_ACTIONS = [
  ["disabled", "Disabled"],
  ["pan_scroll", "Pan/Scroll"],
  ["pan_scroll_legacy", "Pan/Scroll (Legacy)"],
  ["pan_zoom", "Pan/Zoom"],
  ["scroll", "Scroll"],
  ["wheel", "Wheel"],
  ["zoom", "Zoom"],
  ["left_click", "Left click"],
  ["right_click", "Right click"],
  ["middle_click", "Middle click"],
  ["double_click", "Double click"],
  ["back", "Back"],
  ["forward", "Forward"],
  ["display_toggle", "Display toggle"],
  ["express_menu", "Express menu"],
  ["radial_menu", "Radial menu"],
  ["app_shortcuts", "App shortcuts"],
  ["keypad", "Keypad"],
];

function keyLabel(action) {
  if (action?.type === "keystroke") return action.label || t("Keystroke");
  return t(KEY_ACTIONS.find(([v]) => v === action?.type)?.[1] ?? "Disabled");
}

function expressKeysPage() {
  const m = model();
  const keys = config.express_keys ?? [];
  const chosen = selected?.page === "keys" ? selected.index : null;
  // The key itself (a ring over the artwork, as on the Pen page) and its
  // label both open the Actions panel.
  const callouts = KEY_SPOTS.map(
    (_, i) => `<button class="key-label ${i === chosen ? "selected" : ""}" data-key="${i}">${esc(keyLabel(keys[i]))}</button>
      <button class="key-ring ${i === chosen ? "selected" : ""}" data-key="${i}" aria-label="${esc(keyLabel(keys[i]))}"></button>`,
  ).join("");
  return `
    ${deviceSidebar("expresskeys")}
    ${chosen !== null ? actionsPanel(keyGroups(), keyValue(keys[chosen])) : ""}
    <section class="content">
      <div class="keys-figure">
        <div class="keys-tablet">
          <img src="assets/devices/${m.tablet}.png" alt="">
          <svg class="key-leads"></svg>
          ${callouts}
        </div>
      </div>
      <button class="button corner" id="keys-default">${esc(t("Default"))}</button>
    </section>`;
}

// Leads from each label down to its key. Inner labels sit higher and their
// leads step outwards above the tablet, so the four labels never collide.
function layoutKeyLeads() {
  const box = $(".keys-tablet");
  const img = box && $("img", box);
  if (!img?.complete) return img?.addEventListener("load", layoutKeyLeads, { once: true });
  const w = img.clientWidth;
  const h = img.clientHeight;
  const rowY = KEY_ROW_Y * h;
  const bendY = -0.12 * w;
  const paths = KEY_SPOTS.map((spot, i) => {
    const kx = spot * w;
    const inner = i === 1 || i === 2;
    const lx = inner ? kx + (i === 1 ? -0.045 : 0.045) * w : kx;
    const ly = (inner ? -75 : -45) - 0.19 * w;
    const label = box.querySelector(`.key-label[data-key="${i}"]`);
    label.style.left = `${lx}px`;
    label.style.top = `${ly - 34}px`;
    const ring = box.querySelector(`.key-ring[data-key="${i}"]`);
    ring.style.left = `${kx}px`;
    ring.style.top = `${rowY}px`;
    const points = inner ? [[kx, rowY], [kx, bendY], [lx, bendY - 9], [lx, ly]] : [[kx, rowY], [lx, ly]];
    return `<polyline points="${points.map((p) => p.join(",")).join(" ")}"/><circle cx="${lx}" cy="${ly}" r="4"/>`;
  });
  const leads = $(".key-leads", box);
  leads.setAttribute("viewBox", `0 -${LEAD_ROOM} ${w} ${h + LEAD_ROOM}`);
  leads.innerHTML = paths.join("");
}

function mountExpressKeys() {
  layoutKeyLeads();
  window.addEventListener("resize", layoutKeyLeads);
  // Light up the key being pressed; also notice the tablet going away.
  startPolling(50, () => {
    if (!status.device) return render();
    document.querySelectorAll(".key-ring").forEach((el) =>
      el.classList.toggle("held", Boolean(status.express_keys?.[Number(el.dataset.key)])),
    );
  });
  document.querySelectorAll("[data-key]").forEach((btn) =>
    btn.addEventListener("click", () => select({ page: "keys", index: Number(btn.dataset.key) })),
  );
  mountActions((value) => {
    const index = selected.index;
    if (value === "record") return recordKeystroke((action) => setKey(index, action));
    if (value.startsWith("preset:")) {
      const [label, keys] = KEY_PRESETS[Number(value.slice(7))];
      return setKey(index, { type: "keystroke", keys, label: t(label) });
    }
    setKey(index, { type: value });
  });
  $("#keys-default").addEventListener("click", () => {
    config.express_keys = structuredClone(DEFAULTS.express_keys);
    save();
    render();
  });
}

function setKey(index, action) {
  config.express_keys = Array.from({ length: 4 }, (_, i) => config.express_keys?.[i] ?? { type: "disabled" });
  config.express_keys[index] = action;
  save();
  render();
}

// The Actions panel value for an ExpressKey's current action.
function keyValue(action = { type: "disabled" }) {
  if (action.type !== "keystroke") return action.type;
  const preset = KEY_PRESETS.findIndex(([l]) => t(l) === action.label);
  return preset >= 0 ? `preset:${preset}` : "record";
}

// KeyboardEvent.code -> evdev key name. Both describe physical positions,
// so the shortcut matches whatever keyboard layout GNOME applies.
function evdevKey(code) {
  const named = {
    ControlLeft: "LEFTCTRL", ControlRight: "RIGHTCTRL", ShiftLeft: "LEFTSHIFT", ShiftRight: "RIGHTSHIFT",
    AltLeft: "LEFTALT", AltRight: "RIGHTALT", MetaLeft: "LEFTMETA", MetaRight: "RIGHTMETA",
    Escape: "ESC", Enter: "ENTER", Space: "SPACE", Tab: "TAB", Backspace: "BACKSPACE", Delete: "DELETE",
    Insert: "INSERT", Home: "HOME", End: "END", PageUp: "PAGEUP", PageDown: "PAGEDOWN",
    ArrowUp: "UP", ArrowDown: "DOWN", ArrowLeft: "LEFT", ArrowRight: "RIGHT",
    Minus: "MINUS", Equal: "EQUAL", BracketLeft: "LEFTBRACE", BracketRight: "RIGHTBRACE",
    Semicolon: "SEMICOLON", Quote: "APOSTROPHE", Backquote: "GRAVE", Backslash: "BACKSLASH",
    Comma: "COMMA", Period: "DOT", Slash: "SLASH", IntlBackslash: "102ND",
    NumpadAdd: "KPPLUS", NumpadSubtract: "KPMINUS", NumpadMultiply: "KPASTERISK", NumpadDivide: "KPSLASH",
    NumpadEnter: "KPENTER", NumpadDecimal: "KPDOT",
  };
  if (named[code]) return `KEY_${named[code]}`;
  let m;
  if ((m = code.match(/^Key([A-Z])$/))) return `KEY_${m[1]}`;
  if ((m = code.match(/^Digit(\d)$/))) return `KEY_${m[1]}`;
  if ((m = code.match(/^Numpad(\d)$/))) return `KEY_KP${m[1]}`;
  if ((m = code.match(/^F(\d{1,2})$/))) return `KEY_F${m[1]}`;
  return null;
}

const MODIFIER_NAMES = { Control: "Ctrl", Shift: "Shift", Alt: "Alt", Meta: "Super" };

function recordKeystroke(apply) {
  const dialog = document.createElement("div");
  dialog.className = "dialog-backdrop";
  dialog.innerHTML = `
    <div class="dialog" role="dialog" aria-modal="true">
      <h3>${esc(t("Keystroke"))}</h3>
      <p class="dialog-hint">${esc(t("Press the key combination on your keyboard."))}</p>
      <div class="capture" tabindex="0">—</div>
      <div class="dialog-buttons">
        <button class="button" data-act="cancel">${esc(t("Cancel"))}</button>
        <button class="button primary" data-act="ok" disabled>${esc(t("OK"))}</button>
      </div>
    </div>`;
  document.body.append(dialog);
  const capture = $(".capture", dialog);
  const ok = $('[data-act="ok"]', dialog);
  let keys = [];
  let label = "";
  capture.focus();
  capture.addEventListener("keydown", (e) => {
    e.preventDefault();
    const key = evdevKey(e.code);
    if (!key) return;
    const isModifier = /Control|Shift|Alt|Meta/.test(e.code);
    const mods = [];
    if (e.ctrlKey) mods.push(["Control", "KEY_LEFTCTRL"]);
    if (e.shiftKey) mods.push(["Shift", "KEY_LEFTSHIFT"]);
    if (e.altKey) mods.push(["Alt", "KEY_LEFTALT"]);
    if (e.metaKey) mods.push(["Meta", "KEY_LEFTMETA"]);
    keys = [...mods.map(([, k]) => k), ...(isModifier ? [] : [key])];
    const main = isModifier ? [] : [e.key.length === 1 ? e.key.toUpperCase() : e.code.replace(/^(Key|Digit|Arrow)/, "")];
    label = [...mods.map(([n]) => MODIFIER_NAMES[n]), ...main].join("+");
    capture.textContent = label || "—";
    ok.disabled = isModifier && mods.length === 0;
  });
  dialog.addEventListener("click", (e) => {
    const act = e.target.closest("[data-act]")?.dataset.act;
    if (act === "ok" && keys.length) apply({ type: "keystroke", keys, label });
    if (act) dialog.remove();
  });
}

// Pen-sensitive part of the Intuos S artwork, as fractions of the image.
const ACTIVE_AREA = { x0: 0.114, x1: 0.904, y0: 0.304, y1: 0.926 };

function mappingPage() {
  const displays = status.displays ?? [];
  const m = config.mapping;
  const option = (value, label) => `
    <label class="radio">
      <input type="radio" name="display" value="${esc(value)}" ${(m.display ?? "") === value ? "checked" : ""}>
      <span></span>${esc(label)}
    </label>`;
  const displayOptions = displays
    .map((d, i) => option(d.id, `${t("Display %s").replace("%s", i + 1)} · ${d.name} (${Math.round(d.width)}×${Math.round(d.height)})`))
    .join("");
  return `
    ${deviceSidebar("mapping")}
    <section class="content">
      <div class="mapping">
        <div class="mapping-figure">${mappingDiagram()}</div>
        <div class="mapping-options">
          <p class="label">${esc(t("Display mapping"))}</p>
          ${option("", t(displays.length > 1 ? "Map to all displays" : "Map to full display"))}
          ${displayOptions}
          <p class="label" style="margin-top:34px">${esc(t("Force proportions"))}
            <span class="help" title="${esc(t(FORCE_HELP))}">?</span></p>
          <label class="toggle-row" style="width:auto;margin:0">
            <span class="switch"><input type="checkbox" id="force" ${m.force_proportions ? "checked" : ""}><span></span></span>
            ${esc(t(m.force_proportions ? "On" : "Off"))}
          </label>
          <p class="label" style="margin-top:34px">${esc(t("Tablet area"))}</p>
          <label class="radio"><input type="radio" checked><span></span>${esc(t("Use full area"))}</label>
        </div>
      </div>
      <button class="button corner" id="mapping-default">${esc(t("Default"))}</button>
    </section>`;
}

const FORCE_HELP =
  'To prevent this, select "Force proportions" to use only a proportional area of the pen display. This may leave a portion of your pen display inactive, but ensures proportional pen movement.';

// Screens on top as they are arranged on the desktop, the tablet below,
// and a projection from the tablet's active area to the screens it drives.
function mappingDiagram() {
  const real = status.displays?.length > 0;
  const displays = real ? status.displays : [{ id: "", x: 0, y: 0, width: 16, height: 9 }];
  const W = 600;
  const x0 = Math.min(...displays.map((d) => d.x));
  const y0 = Math.min(...displays.map((d) => d.y));
  const dw = Math.max(...displays.map((d) => d.x + d.width)) - x0;
  const dh = Math.max(...displays.map((d) => d.y + d.height)) - y0;
  const scale = Math.min(540 / dw, 240 / dh);
  const ox = (W - dw * scale) / 2;
  const oy = 16;
  const box = (d) => ({ x: ox + (d.x - x0) * scale, y: oy + (d.y - y0) * scale, w: d.width * scale, h: d.height * scale });

  const selected = displays.find((d) => d.id === config.mapping.display);
  const target = selected ? box(selected) : { x: ox, y: oy, w: dw * scale, h: dh * scale };

  // Tablet image and the part of its active area in use.
  const img = { x: 160, y: 384, w: 277, h: 221 };
  let area = {
    x: img.x + ACTIVE_AREA.x0 * img.w,
    y: img.y + ACTIVE_AREA.y0 * img.h,
    w: (ACTIVE_AREA.x1 - ACTIVE_AREA.x0) * img.w,
    h: (ACTIVE_AREA.y1 - ACTIVE_AREA.y0) * img.h,
  };
  const turns = orientationTurns();
  const rotate = (x, y) => {
    const dx = x - (img.x + img.w / 2), dy = y - (img.y + img.h / 2);
    const [u, v] = [[dx, dy], [-dy, dx], [-dx, -dy], [dy, -dx]][turns];
    return [img.x + img.w / 2 + u, img.y + img.h / 2 + v];
  };
  const corners = [rotate(area.x, area.y), rotate(area.x + area.w, area.y + area.h)];
  area = {
    x: Math.min(...corners.map(([x]) => x)),
    y: Math.min(...corners.map(([, y]) => y)),
    w: Math.abs(corners[1][0] - corners[0][0]),
    h: Math.abs(corners[1][1] - corners[0][1]),
  };
  const full = { ...area };
  if (config.mapping.force_proportions) {
    const physicalAspect = status.device?.width_mm && status.device?.height_mm
      ? status.device.width_mm / status.device.height_mm : (ACTIVE_AREA.x1 - ACTIVE_AREA.x0) * img.w / ((ACTIVE_AREA.y1 - ACTIVE_AREA.y0) * img.h);
    const tablet = turns % 2 ? 1 / physicalAspect : physicalAspect;
    const screen = target.w / target.h;
    if (tablet > screen) area.w *= screen / tablet;
    else area.h *= tablet / screen;
  }

  // A small gap between neighbouring screens keeps them apart visually.
  const GAP = 3;
  const monitors = displays
    .map((d, i) => {
      const b = box(d);
      const on = !selected || d === selected;
      const cx = b.x + b.w / 2, cy = b.y + b.h / 2;
      const name = real && b.w > 90 && b.h > 60 ? `<text x="${cx}" y="${cy + 22}" class="screen-size">${Math.round(d.width)}×${Math.round(d.height)}</text>` : "";
      return `
        <g class="screen${on ? " on" : ""}${real ? " pick" : ""}" data-display="${esc(d.id)}">
          <title>${esc(real ? `${t("Display %s").replace("%s", i + 1)} · ${d.name}` : "")}</title>
          <rect x="${b.x + GAP}" y="${b.y + GAP}" width="${Math.max(b.w - 2 * GAP, 1)}" height="${Math.max(b.h - 2 * GAP, 1)}" rx="6"/>
          ${real ? `<text x="${cx}" y="${cy + (name ? 0 : 8)}" class="screen-number">${i + 1}</text>${name}` : ""}
        </g>`;
    })
    .join("");
  const all = !selected && displays.length > 1
    ? `<rect x="${target.x}" y="${target.y}" width="${target.w}" height="${target.h}" rx="8" class="span"/>`
    : "";

  // Projection from the active area's top edge to the target's bottom edge.
  const beam = [
    [target.x + GAP, target.y + target.h - GAP],
    [target.x + target.w - GAP, target.y + target.h - GAP],
    [area.x + area.w, area.y],
    [area.x, area.y],
  ];
  return `
    <svg viewBox="0 0 ${W} 670" class="mapping-svg">
      <polygon points="${beam.map((p) => p.join(",")).join(" ")}" class="beam"/>
      <line x1="${beam[0][0]}" y1="${beam[0][1]}" x2="${beam[3][0]}" y2="${beam[3][1]}" class="beam-edge"/>
      <line x1="${beam[1][0]}" y1="${beam[1][1]}" x2="${beam[2][0]}" y2="${beam[2][1]}" class="beam-edge"/>
      ${monitors}
      ${all}
      <image href="assets/devices/${model().tablet}.png" x="${img.x}" y="${img.y}" width="${img.w}" height="${img.h}" transform="rotate(${turns * 90} ${img.x + img.w / 2} ${img.y + img.h / 2})"/>
      ${area.w !== full.w || area.h !== full.h ? `<rect x="${full.x}" y="${full.y}" width="${full.w}" height="${full.h}" rx="3" class="area-unused"/>` : ""}
      <rect x="${area.x}" y="${area.y}" width="${area.w}" height="${area.h}" rx="3" class="area"/>
    </svg>`;
}

function mountMapping() {
  document.querySelectorAll(".mapping-svg .pick").forEach((el) =>
    el.addEventListener("click", () => {
      config.mapping.display = config.mapping.display === el.dataset.display ? null : el.dataset.display;
      save();
      render();
    }),
  );
  document.querySelectorAll('input[name="display"]').forEach((el) =>
    el.addEventListener("change", () => {
      config.mapping.display = el.value || null;
      save();
      render();
    }),
  );
  $("#force").addEventListener("change", (e) => {
    config.mapping.force_proportions = e.target.checked;
    save();
    render();
  });
  $("#mapping-default").addEventListener("click", () => {
    // Orientation belongs to the device page, not to this one.
    config.mapping = { ...structuredClone(DEFAULTS.mapping), orientation: config.mapping.orientation };
    save();
    render();
  });
}

function displayTogglePage() {
  const displays = status.displays ?? [];
  const toggled = status.toggled_display;
  const current = toggled !== undefined && toggled !== null ? toggled : config.mapping.display;
  const index = displays.findIndex((d) => d.id === current);
  const active = index >= 0 ? `${t("Display %s").replace("%s", index + 1)} · ${displays[index].name}` : t("Map to all displays");
  return `
    ${deviceSidebar("display-toggle")}
    <section class="content">
      <div class="toggle-page">
        ${toggleAnimation()}
        <p class="toggle-text">${esc(t("Display toggle cycles your cursor between multiple screens and is only available for multi-screen setups."))}</p>
        ${
          displays.length > 1
            ? `<p class="toggle-status">${esc(t("Now"))}: <b>${esc(active)}</b></p>`
            : ""
        }
      </div>
    </section>`;
}

// Display toggle at work: each press of the key moves the pen's area from
// all displays to the left one, then the right one. Drawn in CSS so it
// follows the theme; style.css runs the three phases.
function toggleAnimation() {
  const area = { x: 205, y: 222, w: 110, h: 50 };
  const beam = (x0, x1, phase) => `
    <g class="phase p${phase}">
      <polygon class="beam" points="${x0},140 ${x1},140 ${area.x + area.w},${area.y} ${area.x},${area.y}"/>
      <rect class="target" x="${x0}" y="20" width="${x1 - x0}" height="120" rx="8"/>
      <circle class="cursor" cx="${(x0 + x1) / 2}" cy="80" r="5"/>
    </g>`;
  return `
    <svg class="toggle-anim" viewBox="0 0 520 300" aria-hidden="true">
      <rect class="tg-screen" x="40" y="20" width="200" height="120" rx="8"/>
      <rect class="tg-screen" x="280" y="20" width="200" height="120" rx="8"/>
      <path class="stand" d="M132 140v14M108 154h48M372 140v14M348 154h48"/>
      ${beam(36, 484, 0)}${beam(40, 240, 1)}${beam(280, 480, 2)}
      <rect class="tg-tablet" x="180" y="190" width="160" height="96" rx="10"/>
      <rect class="active" x="${area.x}" y="${area.y}" width="${area.w}" height="${area.h}" rx="3"/>
      <rect class="key" x="196" y="198" width="30" height="14" rx="7"/>
      <rect class="key press" x="196" y="198" width="30" height="14" rx="7"/>
      <rect class="key" x="230" y="198" width="30" height="14" rx="7"/>
      <rect class="key" x="264" y="198" width="30" height="14" rx="7"/>
      <rect class="key" x="298" y="198" width="30" height="14" rx="7"/>
    </svg>`;
}

const GESTURES = [
  ["pan-scroll", "Pan/Scroll"],
  ["pan-scroll-legacy", "Pan/Scroll (Legacy)"],
  ["pan-zoom", "Pan/Zoom"],
  ["scroll", "Scroll"],
  ["wheel", "Wheel"],
  ["zoom", "Zoom"],
];

function shortcutsSidebar(active) {
  const item = (route, ic, label) =>
    `<a class="side-item sub ${active === route ? "active" : ""}" href="#/shortcuts/${route}">${icon(ic)}${esc(t(label))}</a>`;
  return `
    <aside class="sidebar plain">
      <a class="side-item ${active === "" ? "active" : ""}" href="#/shortcuts">${icon("home")}${esc(t("Start"))}</a>
      <div class="side-heading">${esc(t("Radial menus"))}</div>
      ${item("express-menu", "radial", "Express menu")}
      ${item("radial-menu", "radial", "Radial menu")}
      <div class="side-heading">${esc(t("Grid panels"))}</div>
      ${item("app-shortcuts", "grid", "App shortcuts")}
      ${item("keypad", "keypad", "Keypad")}
      <div class="side-heading">${esc(t("Pen gestures"))}</div>
      ${GESTURES.map(([route, label]) => item(route, "gesture-pan", label)).join("")}
    </aside>`;
}

// Pen gesture pages: config action, speed setting, how it is used, and
// whether the illustration shows only the vertical arrows.
const GESTURE_PAGES = {
  "pan-scroll": {
    speed: ["pan_scroll", "speed"],
    hint: "Hold the button, touch the tablet and drag: the content moves with the pen.",
  },
  "pan-scroll-legacy": {
    speed: ["gestures", "legacy_speed"],
    hint: "Hold the button and touch the tablet: the farther you move the pen, the faster it scrolls, even when the pen rests.",
  },
  "pan-zoom": {
    speed: ["gestures", "pan_zoom_speed"],
    hint: "Hold the button and drag the tip to pan. Hold it with the pen in the air and move up to zoom in or down to zoom out.",
  },
  scroll: {
    speed: ["gestures", "scroll_speed"],
    hint: "Hold the button, touch the tablet and drag up or down to scroll.",
    vertical: true,
  },
  wheel: {
    speed: ["gestures", "wheel_speed"],
    hint: "Works like a mouse wheel: the farther you move the pen, the faster it turns.",
  },
  zoom: {
    speed: ["gestures", "zoom_speed"],
    hint: "Hold the button, touch the tablet and move the pen up to zoom in or down to zoom out.",
    vertical: true,
  },
};

function shortcutsPage(sub) {
  if (ON_SCREEN[sub]) return onScreenPage(sub);
  const title = GESTURES.find(([r]) => r === sub)?.[1];
  const page = GESTURE_PAGES[sub];
  if (!page) return shortcutsStart();
  const [group, key] = page.speed;
  return `
    ${shortcutsSidebar(sub)}
    <section class="content">
      <div class="gesture">
        <h2>${esc(t(title))}</h2>
        ${panIllustration(page.vertical)}
        <div class="slider">
          <p class="label">${esc(t("Speed"))}</p>
          ${slider("gesture-speed", config[group][key], 1, 5, "Slow", "Fast")}
        </div>
        ${
          sub === "pan-scroll"
            ? `<label class="toggle-row">
                 <span class="switch"><input type="checkbox" id="follows" ${config.pan_scroll.content_follows_pen ? "checked" : ""}><span></span></span>
                 ${esc(t("Content follows the pen"))}
               </label>`
            : ""
        }
        <p class="gesture-hint">${esc(t(page.hint))}</p>
      </div>
    </section>`;
}

function mountShortcuts(sub) {
  if (ON_SCREEN[sub]) return mountOnScreen(sub);
  const page = GESTURE_PAGES[sub];
  if (!page) return;
  const [group, key] = page.speed;
  bindSlider("gesture-speed", (v) => {
    config[group][key] = v;
    save();
  });
  $("#follows")?.addEventListener("change", (e) => {
    config.pan_scroll.content_follows_pen = e.target.checked;
    save();
  });
}

function shortcutsStart() {
  const card = (href, ic, title, text) => `
    <a class="start-card" href="#/shortcuts/${href}">
      ${icon(ic)}
      <b>${esc(t(title))}</b>
      <span>${esc(t(text))}</span>
    </a>`;
  return `
    ${shortcutsSidebar("")}
    <section class="content">
      <div class="watermark">${esc(t("On-screen shortcuts"))}</div>
      <h1 class="page-title">${esc(t("On-screen shortcuts"))}</h1>
      <div class="page-sub">${esc(t("Assign an on-screen shortcut to a pen button or ExpressKey."))}</div>
      <div class="start-cards">
        ${card("express-menu", "radial", "Radial menus", "Radial menus open around the pen, so reaching an action takes a short flick.")}
        ${card("app-shortcuts", "grid", "Grid panels", "Grid panels hold more actions, like your app's shortcuts or a numeric keypad.")}
        ${card("pan-scroll", "gesture-pan", "Pen gestures", "Pen gestures scroll, pan and zoom while you hold the button and move the pen.")}
      </div>
    </section>`;
}

// ---------- On-screen shortcuts: radial menus and grid panels ----------

// Pages under On-screen shortcuts: route -> config key and layout.
const ON_SCREEN = {
  "express-menu": { key: "express_menu", kind: "radial", title: "Express menu" },
  "radial-menu": { key: "radial_menu", kind: "radial", title: "Radial menu" },
  "app-shortcuts": { key: "app_shortcuts", kind: "grid", title: "App shortcuts" },
  keypad: { key: "keypad", kind: "keypad", title: "Keypad" },
};
const onScreenByKey = (key) => Object.values(ON_SCREEN).find((p) => p.key === key);

// Slider steps 1..5 in pixels.
const RADIAL_SIZES = [240, 290, 340, 390, 440];
const CELL_SIZES = [60, 70, 80, 92, 104];
const TEXT_SIZES = [11, 12, 13, 15, 17];
const RADIAL_SLICES = 8;
const GRID_CELLS = 12;

// Number keys from the main row, so they type digits whatever Num Lock says.
const KEYPAD = [
  ["Esc", "KEY_ESC"], ["Tab", "KEY_TAB"], ["⌫", "KEY_BACKSPACE"], ["/", "KEY_KPSLASH"],
  ["7", "KEY_7"], ["8", "KEY_8"], ["9", "KEY_9"], ["*", "KEY_KPASTERISK"],
  ["4", "KEY_4"], ["5", "KEY_5"], ["6", "KEY_6"], ["−", "KEY_KPMINUS"],
  ["1", "KEY_1"], ["2", "KEY_2"], ["3", "KEY_3"], ["+", "KEY_KPPLUS"],
  ["0", "KEY_0", 2], [".", "KEY_DOT"], ["Enter", "KEY_ENTER"],
].map(([label, key, span]) => ({ label, span, action: { type: "keystroke", keys: [key], label } }));

function menuItems(key) {
  const menu = config.shortcuts?.[key] ?? DEFAULTS.shortcuts[key];
  if (key === "keypad") return KEYPAD;
  const count = onScreenByKey(key).kind === "radial" ? RADIAL_SLICES : GRID_CELLS;
  return Array.from({ length: count }, (_, i) => menu.items[i] ?? { label: "", action: { type: "disabled" } });
}

function itemLabel(item) {
  if (item.label) return item.label;
  const a = item.action ?? { type: "disabled" };
  return a.type === "keystroke" ? t(a.label) || a.keys.join("+") : actionLabel(a.type);
}

function itemIcon(item) {
  const a = item.action ?? { type: "disabled" };
  if (a.type !== "keystroke") return actionIcon(a.type);
  const preset = KEY_PRESETS.findIndex(([, keys]) => keys.join() === a.keys.join());
  return icon(`action-${PRESET_ICONS[preset] ?? "keyboard"}`);
}

// A ring of slices, clockwise from the top, with a close button in the hub.
function radialMenu(key, selectedIndex = null) {
  const menu = config.shortcuts[key];
  const items = menuItems(key);
  const n = items.length;
  const R = 100;
  const r = 34;
  const at = (radius, a) => `${(radius * Math.cos(a)).toFixed(2)} ${(radius * Math.sin(a)).toFixed(2)}`;
  const angle = (i) => (i / n) * 2 * Math.PI - Math.PI / 2;
  const slices = items.map((item, i) => {
    const a0 = angle(i - 0.5);
    const a1 = angle(i + 0.5);
    const off = item.action?.type === "disabled" ? "off" : "";
    return `<path data-item="${i}" class="slice ${off} ${i === selectedIndex ? "selected" : ""}"
      d="M${at(r, a0)} L${at(R, a0)} A${R} ${R} 0 0 1 ${at(R, a1)} L${at(r, a1)} A${r} ${r} 0 0 0 ${at(r, a0)} Z"/>`;
  });
  // Labels sit halfway between hub and rim; the SVG spans 204 units.
  const labels = items.map((item, i) => {
    const mid = ((r + R) / 2 / 204) * 100;
    const x = 50 + mid * Math.cos(angle(i));
    const y = 50 + mid * Math.sin(angle(i));
    const off = item.action?.type === "disabled" ? "off" : "";
    return `<span class="slice-label ${off}" style="left:${x}%;top:${y}%">${itemIcon(item)}<span>${esc(itemLabel(item))}</span></span>`;
  });
  return `
    <div class="radial" style="--d:${RADIAL_SIZES[menu.size - 1] ?? 340}px;--text:${TEXT_SIZES[menu.text_size - 1] ?? 13}px">
      <svg viewBox="-102 -102 204 204">${slices.join("")}<circle class="hub" r="${r - 4}"/></svg>
      ${labels.join("")}
      <button class="hub-close" data-close aria-label="${esc(t("Close"))}">${icon("action-disabled")}</button>
    </div>`;
}

// A grid panel: buttons under a title bar.
function gridPanel(key, selectedIndex = null) {
  const menu = config.shortcuts[key];
  const keypad = key === "keypad";
  const cells = menuItems(key).map((item, i) => {
    const off = item.action?.type === "disabled" ? "off" : "";
    const content = keypad ? esc(item.label) : `${itemIcon(item)}<span>${esc(itemLabel(item))}</span>`;
    return `<button class="cell ${off} ${keypad ? "key" : ""} ${i === selectedIndex ? "selected" : ""}"
      data-item="${i}" ${item.span ? `style="grid-column:span ${item.span}"` : ""}>${content}</button>`;
  });
  return `
    <div class="grid-panel" style="--cell:${CELL_SIZES[menu.size - 1] ?? 80}px;--text:${TEXT_SIZES[menu.text_size - 1] ?? 13}px">
      <header><span>${esc(t(onScreenByKey(key).title))}</span>
        <button class="panel-close" data-close aria-label="${esc(t("Close"))}">${icon("action-disabled")}</button></header>
      <div class="cells">${cells.join("")}</div>
    </div>`;
}

function onScreenMenu(key, selectedIndex) {
  return onScreenByKey(key).kind === "radial" ? radialMenu(key, selectedIndex) : gridPanel(key, selectedIndex);
}

function onScreenPage(sub) {
  const page = ON_SCREEN[sub];
  const menu = config.shortcuts[page.key];
  const chosen = selected?.page === "menu" && page.kind !== "keypad" ? selected.index : null;
  const item = chosen !== null ? menuItems(page.key)[chosen] : null;
  const current = item && (item.action.type === "keystroke" ? keyValue(item.action) : item.action.type);
  return `
    ${shortcutsSidebar(sub)}
    ${item ? actionsPanel(menuGroups(), current) : ""}
    <section class="content">
      <div class="on-screen">
        <div class="on-screen-preview">
          <h2>${esc(t(page.title))}</h2>
          ${onScreenMenu(page.key, chosen)}
          <p class="gesture-hint">${esc(t(page.kind === "keypad"
            ? "Each key types once and closes the keypad."
            : "Click an entry to choose its action. Assign this menu to a pen button or ExpressKey to open it."))}</p>
        </div>
        <div class="on-screen-options">
          ${item ? `<p class="label">${esc(t("Label"))}</p>
            <input class="text-input" id="item-label" value="${esc(item.label)}" placeholder="${esc(itemLabel({ ...item, label: "" }))}">` : ""}
          <div class="slider">
            <p class="label">${esc(t("Size"))}</p>
            ${slider("menu-size", menu.size, 1, 5, "Small", "Large")}
          </div>
          <div class="slider">
            <p class="label">${esc(t("Text size"))}</p>
            ${slider("menu-text", menu.text_size, 1, 5, "Small", "Large")}
          </div>
          <label class="toggle-row">
            <span class="switch"><input type="checkbox" id="at-cursor" ${menu.at_cursor ? "checked" : ""}><span></span></span>
            ${esc(t("Display at cursor"))}
          </label>
        </div>
      </div>
      <button class="button corner" id="menu-default">${esc(t("Default"))}</button>
    </section>`;
}

function mountOnScreen(sub) {
  const { key, kind } = ON_SCREEN[sub];
  const menu = config.shortcuts[key];
  const setItem = (index, patch) => {
    menu.items = menuItems(key).map((it, i) => (i === index ? { ...it, ...patch } : it));
    save();
    render();
  };
  const bindItems = () => {
    if (kind === "keypad") return;
    document.querySelectorAll(".on-screen [data-item]").forEach((el) =>
      el.addEventListener("click", () => select({ page: "menu", index: Number(el.dataset.item) })),
    );
  };
  // Sliders redraw only the preview, so dragging them stays smooth.
  const redraw = () => {
    $(".on-screen-preview .radial, .on-screen-preview .grid-panel").outerHTML = onScreenMenu(key, selected?.index ?? null);
    bindItems();
  };
  bindItems();
  mountActions((value) => {
    const index = selected.index;
    if (value === "record") return recordKeystroke((action) => setItem(index, { action }));
    if (value.startsWith("preset:")) {
      const [label, keys] = KEY_PRESETS[Number(value.slice(7))];
      return setItem(index, { action: { type: "keystroke", keys, label: t(label) } });
    }
    setItem(index, { action: { type: value } });
  });
  $("#item-label")?.addEventListener("change", (e) => setItem(selected.index, { label: e.target.value.trim() }));
  bindSlider("menu-size", (v) => {
    menu.size = v;
    save();
    redraw();
  });
  bindSlider("menu-text", (v) => {
    menu.text_size = v;
    save();
    redraw();
  });
  $("#at-cursor").addEventListener("change", (e) => {
    menu.at_cursor = e.target.checked;
    save();
  });
  $("#menu-default").addEventListener("click", () => {
    config.shortcuts[key] = structuredClone(DEFAULTS.shortcuts[key]);
    selected = null;
    save();
    render();
  });
}

// Four dashed arrows with a pen, as in Tablet Center's gesture page.
function panIllustration(verticalOnly = false) {
  const arm = (x2, y2, a) =>
    `<line x1="210" y1="210" x2="${x2}" y2="${y2}" stroke-dasharray="14 12"/>` +
    `<polyline points="${a}" fill="none"/>`;
  return `
    <svg viewBox="0 0 420 420" fill="none" stroke="currentColor" stroke-width="5" stroke-linecap="round" aria-hidden="true">
      ${arm(210, 20, "188,42 210,20 232,42")}
      ${arm(210, 400, "188,378 210,400 232,378")}
      ${verticalOnly ? "" : arm(20, 210, "42,188 20,210 42,232")}
      ${verticalOnly ? "" : arm(400, 210, "378,188 400,210 378,232")}
      <g stroke-width="4" stroke-linejoin="round">
        <path d="M214 206 L228 168 L364 32 a10 10 0 0 1 16 16 L244 184 Z"/>
        <path d="M244 184 L228 168"/>
        <path d="M300 106 l-34 34 M308 114 l-34 34"/>
      </g>
    </svg>`;
}

function settingsPage(tab = "general") {
  if (!["devices", "backups"].includes(tab)) tab = "general";
  const tabs = [
    ["general", "General settings"],
    ["devices", "Devices manager"],
    ["backups", "Backup & restore"],
  ]
    .map(([id, label]) => `<a href="#/settings/${id}" class="${tab === id ? "active" : ""}">${esc(t(label))}</a>`)
    .join("");
  let body;
  if (tab === "general") {
    const running = !status.error;
    body = `
      <h3>${esc(t("Driver service"))}</h3>
      <p><span class="status-dot ${running ? "on" : ""}"></span>${esc(t(running ? "Running" : "Not running"))} — tabletd</p>
      <hr>
      <button class="button" id="restart">${esc(t("Restart Driver"))}</button>
      <hr>
      <h3>${esc(t("Language"))}</h3>
      <select id="language" class="button">
        ${LANGUAGES.map(([v, l]) => `<option value="${v}" ${language() === v ? "selected" : ""}>${esc(l)}</option>`).join("")}
      </select>
      <hr>
      <h3>${esc(t("Interface"))}</h3>
      ${uiStyleSelect()}
      <hr>
      <label class="autostart-row" for="autostart">${esc(t("Tablet Center autostart"))}
        <span class="switch"><input type="checkbox" id="autostart" disabled><span></span></span>
      </label>
      <hr>
      ${shellMenusSwitch()}`;
  } else if (tab === "backups") {
    body = `
      <div class="backup-buttons">
        <button class="button" id="new-backup">${esc(t("New backup"))}</button>
        <button class="button" id="open-backups">${esc(t("Open backup folder"))}</button>
        <button class="button" id="reset-config">${esc(t("Reset all settings"))}</button>
      </div>
      <table class="backups"><thead><tr>
        <th>${esc(t("Title"))}</th><th>${esc(t("Date created"))}</th><th>${esc(t("Actions"))}</th>
      </tr></thead><tbody id="backup-list"></tbody></table>`;
  } else {
    const m = model();
    body = status.device
      ? `<h3>${esc(t("Your connected devices"))}</h3>
         <p style="display:flex;align-items:center;gap:16px">
           <img src="assets/devices/${m.tablet}.png" alt="" style="width:96px">
           ${esc(m.name)} &nbsp; <span style="color:var(--muted)">${esc(m.model)}</span>
         </p>`
      : `<p>${esc(t("Connect a device to see its settings here."))}</p>`;
  }
  return `<section class="content settings"><nav class="tabs">${tabs}</nav><div class="card">${body}</div></section>`;
}

function settingsDialog(title, action, withTitle = false) {
  return new Promise((resolve) => {
    const backdrop = document.createElement("div");
    backdrop.className = "dialog-backdrop";
    backdrop.innerHTML = `<form class="dialog" role="dialog" aria-modal="true" aria-labelledby="settings-dialog-title">
      <h3 id="settings-dialog-title">${esc(t(title))}</h3>
      ${withTitle ? `<label for="backup-title">${esc(t("Backup title"))}</label><input id="backup-title" maxlength="120" required>` : ""}
      <div class="dialog-buttons">
        <button type="button" class="button" data-cancel>${esc(t("Cancel"))}</button>
        <button type="submit" class="button primary">${esc(t(action))}</button>
      </div></form>`;
    const previous = document.activeElement;
    const close = (value) => {
      backdrop.remove();
      previous?.focus();
      resolve(value);
    };
    backdrop.addEventListener("keydown", (e) => {
      if (e.key === "Escape") { e.preventDefault(); close(null); }
      if (e.key === "Tab") {
        const items = [...backdrop.querySelectorAll("input, button")];
        const next = (items.indexOf(document.activeElement) + (e.shiftKey ? -1 : 1) + items.length) % items.length;
        e.preventDefault();
        items[next].focus();
      }
    });
    $("[data-cancel]", backdrop).onclick = () => close(null);
    $("form", backdrop).onsubmit = (e) => {
      e.preventDefault();
      const value = withTitle ? $("input", backdrop).value.trim() : true;
      if (value) close(value);
    };
    document.body.append(backdrop);
    $(withTitle ? "input" : "[data-cancel]", backdrop).focus();
  });
}

async function refreshBackups() {
  const list = $("#backup-list");
  if (!list) return;
  const backups = await invoke("list_backups");
  list.innerHTML = backups.length ? backups.map((backup) => `<tr>
    <td>${esc(backup.title)}</td><td>${esc(new Date(backup.created * 1000).toLocaleString(language()))}</td>
    <td>${["Restore", "Delete"].map((action) => `<button class="button" data-backup="${esc(backup.filename)}" data-action="${action}">${esc(t(action))}</button>`).join(" ")}</td>
  </tr>`).join("") : `<tr><td colspan="3">${esc(t("No backups yet."))}</td></tr>`;
}

async function applyBackupCommand(command, args = {}) {
  clearTimeout(saveTimer);
  const applied = await invoke(command, args);
  config = { ...structuredClone(DEFAULTS), ...(await invoke("get_config")) };
  render();
  if (!applied) toast(t("Settings saved. Start tabletd to apply them."));
}

function mountSettings(tab) {
  const handle = (callback) => async () => {
    try { await callback(); } catch (e) { toast(String(e)); }
  };
  if (tab === "backups") {
    refreshBackups().catch((e) => toast(String(e)));
    $("#new-backup").onclick = handle(async () => {
      const title = await settingsDialog("New backup", "Create", true);
      if (!title) return;
      clearTimeout(saveTimer);
      await invoke("save_config", { config });
      await invoke("create_backup", { title });
      await refreshBackups();
    });
    $("#open-backups").onclick = handle(() => invoke("open_backups_folder"));
    $("#reset-config").onclick = handle(async () => {
      if (await settingsDialog("Reset all settings to their defaults?", "Reset all settings")) await applyBackupCommand("reset_config");
    });
    $("#backup-list").onclick = async (e) => {
      const button = e.target.closest("[data-backup]");
      if (!button) return;
      await handle(async () => {
        const restore = button.dataset.action === "Restore";
        if (!await settingsDialog(restore ? "Restore this backup? Current settings will be replaced." : "Delete this backup? This cannot be undone.", restore ? "Restore" : "Delete")) return;
        const args = { filename: button.dataset.backup };
        if (restore) await applyBackupCommand("restore_backup", args);
        else { await invoke("delete_backup", args); await refreshBackups(); }
      })();
    };
    return;
  }
  if (tab === "devices") return;
  const autostart = $("#autostart");
  if (autostart) {
    invoke("autostart_enabled").then((enabled) => {
      autostart.checked = enabled;
      autostart.disabled = false;
    }).catch((e) => toast(String(e)));
    autostart.onchange = async () => {
      autostart.disabled = true;
      try { await invoke("set_autostart", { enabled: autostart.checked }); }
      catch (e) { autostart.checked = !autostart.checked; toast(String(e)); }
      finally { autostart.disabled = false; }
    };
  }
  mountUiStyleSelect();
  $("#language")?.addEventListener("change", (e) => {
    try {
      localStorage.setItem("language", e.target.value);
    } catch {}
    location.reload();
  });
  $("#restart")?.addEventListener("click", async () => {
    try {
      await invoke("restart_daemon");
      toast("tabletd ✓");
    } catch (e) {
      toast(String(e));
    }
  });
}

// ---------- router ----------

// Asks again only after the previous answer arrived, and not at all while
// the window is hidden, so a slow daemon never piles requests up.
function startPolling(ms, onTick) {
  stopPolling();
  const run = pollTimer;
  const tick = async () => {
    if (document.visibilityState === "visible") {
      status = await invoke("status");
      if (run !== pollTimer) return;
      onTick?.();
    }
    if (run === pollTimer) setTimeout(tick, ms);
  };
  setTimeout(tick, ms);
}

function stopPolling() {
  pollTimer++;
}

let lastRoute = "";

function render() {
  if (location.hash !== lastRoute) selected = null;
  lastRoute = location.hash;
  stopPolling();
  window.removeEventListener("resize", layoutCallouts);
  window.removeEventListener("resize", layoutKeyLeads);
  const [section = "devices", sub = ""] = location.hash.replace(/^#\//, "").split("/");
  document.querySelectorAll(".tool").forEach((el) => el.classList.toggle("active", el.dataset.route === section));

  if (section === "devices") {
    if (!status.device) {
      view.innerHTML = welcome();
    } else if (sub === "pen") {
      view.innerHTML = penPage();
      mountPen();
      return;
    } else {
      view.innerHTML = { expresskeys: expressKeysPage, mapping: mappingPage, "display-toggle": displayTogglePage }[sub]?.() ?? devicesOverview();
      mountOrientation();
      if (sub === "mapping") mountMapping();
      if (sub === "expresskeys") return mountExpressKeys();
    }
  } else if (section === "shortcuts") {
    view.innerHTML = shortcutsPage(sub);
    mountShortcuts(sub);
  } else if (section === "settings") {
    view.innerHTML = settingsPage(sub || "general");
    mountSettings(sub);
  } else {
    location.replace("#/devices");
    return;
  }
  // Keep the device list fresh so plugging the tablet in updates the page.
  const hadDevice = Boolean(status.device);
  startPolling(1000, () => {
    if (Boolean(status.device) !== hadDevice) render();
  });
}

async function loadLanguage() {
  const lang = language();
  document.documentElement.lang = lang;
  dict = lang === "en"
    ? { ...EXTRA_EN }
    : await fetch(`locales/${lang}.json`).then((r) => r.json()).catch(() => ({}));
}

// The classic interface carries no Wacom branding: every "Wacom" in its
// text reads "Tablet".
function debrand(root) {
  const swap = (text) =>
    text.replace(/\bwacom\b/gi, (w) => (w === "WACOM" ? "TABLET" : w[0] === "W" ? "Tablet" : "tablet"));
  if (root.nodeType === Node.TEXT_NODE) root = root.parentNode;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  for (let node; (node = walker.nextNode()); ) {
    const text = swap(node.data);
    if (text !== node.data) node.data = text;
  }
  root.querySelectorAll?.("[title],[alt],[aria-label]").forEach((el) => {
    for (const name of ["title", "alt", "aria-label"]) {
      const value = el.getAttribute(name);
      if (value && swap(value) !== value) el.setAttribute(name, swap(value));
    }
  });
}

// ---------- On-screen shortcuts in the GNOME Shell extension ----------

// GNOME symbolic icons for the extension, which has no access to ours.
const SHELL_ICONS = {
  left_click: "input-mouse-symbolic",
  right_click: "input-mouse-symbolic",
  middle_click: "input-mouse-symbolic",
  double_click: "input-mouse-symbolic",
  back: "go-previous-symbolic",
  forward: "go-next-symbolic",
  display_toggle: "video-display-symbolic",
  pan_scroll: "input-tablet-symbolic",
  pan_scroll_legacy: "input-tablet-symbolic",
  pan_zoom: "zoom-fit-best-symbolic",
  scroll: "input-tablet-symbolic",
  wheel: "input-tablet-symbolic",
  zoom: "zoom-in-symbolic",
  express_menu: "view-more-symbolic",
  radial_menu: "view-more-symbolic",
  app_shortcuts: "view-app-grid-symbolic",
  keypad: "input-dialpad-symbolic",
};
const SHELL_PRESET_ICONS = ["edit-undo-symbolic", "edit-redo-symbolic", "edit-copy-symbolic", "edit-paste-symbolic", "document-save-symbolic"];

function shellIcon(action) {
  if (action.type !== "keystroke") return SHELL_ICONS[action.type] ?? "";
  const preset = KEY_PRESETS.findIndex(([, keys]) => keys.join() === action.keys.join());
  return SHELL_PRESET_ICONS[preset] ?? "input-keyboard-symbolic";
}

// Every menu as the extension draws it: sizes in pixels, labels in the
// current language, and the action tabletd runs for each entry.
function shellMenus() {
  return Object.fromEntries(
    Object.values(ON_SCREEN).map(({ key, kind, title }) => {
      const menu = config.shortcuts?.[key] ?? DEFAULTS.shortcuts[key];
      const items = menuItems(key).map((item) => ({
        label: kind === "keypad" ? item.label : itemLabel(item),
        icon: kind === "keypad" ? "" : shellIcon(item.action),
        span: item.span ?? 1,
        action: item.action,
      }));
      return [key, {
        kind,
        title: t(title),
        close: t("Close"),
        diameter: RADIAL_SIZES[menu.size - 1] ?? 340,
        cell: CELL_SIZES[menu.size - 1] ?? 80,
        text: TEXT_SIZES[menu.text_size - 1] ?? 13,
        at_cursor: menu.at_cursor,
        items,
      }];
    }),
  );
}

function saveShellMenus() {
  invoke("save_shell_menus", { menus: shellMenus() }).catch(() => {});
}

const SHELL_MENUS_HINT =
  "Radial menus and grid panels are drawn by the Tablet Center GNOME Shell extension: they open at once and never take the focus. Without the extension they open in a window as before.";

// Shared by the interfaces; the change handler below saves it.
function shellMenusSwitch() {
  return `
    <label class="autostart-row" for="in-shell">${esc(t("On-screen shortcuts in GNOME Shell"))}
      <span class="switch"><input type="checkbox" id="in-shell" ${config.shortcuts?.in_shell ? "checked" : ""}><span></span></span>
    </label>
    <p class="hint">${esc(t(SHELL_MENUS_HINT))}</p>`;
}

document.addEventListener("change", (e) => {
  if (e.target.id !== "in-shell") return;
  config.shortcuts.in_shell = e.target.checked;
  save();
});

async function init() {
  document.title = "Tablet Center";
  window.__TAURI__?.window?.getCurrentWindow().setTitle("Tablet Center").catch(() => {});
  debrand(document.body);
  new MutationObserver((records) => records.forEach((r) => debrand(r.target))).observe(document.body, {
    childList: true,
    subtree: true,
    characterData: true,
  });
  await loadLanguage();
  document.querySelectorAll("[data-t]").forEach((el) => (el.textContent = t(el.dataset.t)));
  await loadConfig();
  saveShellMenus();
  status = await invoke("status");
  window.addEventListener("hashchange", render);
  render();
}

async function loadConfig() {
  config = { ...structuredClone(DEFAULTS), ...(await invoke("get_config")) };
}

// Custom option list for every <select>: GTK's native popup can't be
// styled. The <select> stays the source of truth and receives a normal
// "change" event, so pages keep their listeners.
const dropdown = { select: null, menu: null, active: -1 };

function closeDropdown() {
  dropdown.menu?.remove();
  dropdown.select?.classList.remove("dd-open");
  dropdown.select = dropdown.menu = null;
}

function openDropdown(select) {
  closeDropdown();
  const options = [...select.options];
  const menu = document.createElement("div");
  menu.className = "dd-menu";
  menu.setAttribute("role", "listbox");
  let group = null;
  options.forEach((option, i) => {
    const parent = option.parentElement;
    if (parent.tagName === "OPTGROUP" && parent !== group) {
      group = parent;
      const label = document.createElement("div");
      label.className = "dd-group";
      label.textContent = parent.label;
      menu.append(label);
    }
    const item = document.createElement("div");
    item.className = "dd-option" + (option.disabled ? " disabled" : "");
    item.setAttribute("role", "option");
    item.setAttribute("aria-selected", String(option.selected));
    item.dataset.index = i;
    item.append(Object.assign(document.createElement("span"), { textContent: option.textContent }));
    menu.append(item);
  });
  // Measured and placed before it is ever painted.
  menu.style.visibility = "hidden";
  document.body.append(menu);

  const rect = select.getBoundingClientRect();
  menu.style.minWidth = `${rect.width}px`;
  const below = innerHeight - rect.bottom - 8;
  const above = rect.top - 8;
  const height = Math.min(menu.offsetHeight, Math.max(below, above));
  menu.style.maxHeight = `${height}px`;
  menu.style.left = `${Math.max(8, Math.min(rect.left, innerWidth - menu.offsetWidth - 8))}px`;
  menu.style.top = `${below >= height || below >= above ? rect.bottom + 4 : rect.top - height - 4}px`;

  Object.assign(dropdown, { select, menu });
  select.classList.add("dd-open");
  highlight(select.selectedIndex, false);
  // Bring the current option into view by scrolling the list only; with
  // scrollIntoView the page could scroll too.
  const current = menu.querySelector(".dd-option.active");
  if (current) menu.scrollTop = current.offsetTop - (menu.clientHeight - current.offsetHeight) / 2;
  menu.style.visibility = "";
  menu.addEventListener("mousedown", (e) => e.preventDefault());
  menu.addEventListener("mousemove", (e) => {
    const item = e.target.closest(".dd-option");
    if (item) highlight(Number(item.dataset.index), false);
  });
  menu.addEventListener("click", (e) => {
    const item = e.target.closest(".dd-option");
    if (item) choose(Number(item.dataset.index));
  });
}

function highlight(index, scroll = true) {
  const items = dropdown.menu.querySelectorAll(".dd-option");
  items.forEach((el) => el.classList.toggle("active", Number(el.dataset.index) === index));
  dropdown.active = index;
  const item = scroll && dropdown.menu.querySelector(".dd-option.active");
  if (!item) return;
  // Keep the option in view by scrolling the list, never the page.
  const menu = dropdown.menu;
  if (item.offsetTop < menu.scrollTop) menu.scrollTop = item.offsetTop;
  else if (item.offsetTop + item.offsetHeight > menu.scrollTop + menu.clientHeight)
    menu.scrollTop = item.offsetTop + item.offsetHeight - menu.clientHeight;
}

function choose(index) {
  const select = dropdown.select;
  const option = select.options[index];
  closeDropdown();
  select.focus({ preventScroll: true });
  if (!option || option.disabled || index === select.selectedIndex) return;
  select.selectedIndex = index;
  select.dispatchEvent(new Event("input", { bubbles: true }));
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

function stepDropdown(select, delta) {
  const options = select.options;
  let i = dropdown.select ? dropdown.active : select.selectedIndex;
  do i += delta;
  while (options[i]?.disabled);
  return options[i] ? i : null;
}

document.addEventListener(
  "mousedown",
  (e) => {
    const select = e.target.closest?.("select");
    if (select && !select.multiple && select.size <= 1 && !select.disabled && e.button === 0) {
      e.preventDefault();
      select.focus({ preventScroll: true });
      if (dropdown.select === select) closeDropdown();
      else openDropdown(select);
    } else if (!e.target.closest?.(".dd-menu")) closeDropdown();
  },
  true,
);

document.addEventListener(
  "keydown",
  (e) => {
    const select = dropdown.select ?? (e.target.tagName === "SELECT" ? e.target : null);
    if (!select || select.multiple) return;
    const open = dropdown.select === select;
    let handled = true;
    if (e.key === "Escape" && open) closeDropdown();
    else if ((e.key === "Enter" || e.key === " ") && open) choose(dropdown.active);
    else if (e.key === "Enter" || e.key === " " || (e.altKey && e.key === "ArrowDown")) openDropdown(select);
    else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      const next = stepDropdown(select, e.key === "ArrowDown" ? 1 : -1);
      if (next !== null && open) highlight(next);
      else if (next !== null) {
        select.selectedIndex = next;
        select.dispatchEvent(new Event("change", { bubbles: true }));
      }
    } else if (e.key === "Tab" && open) {
      closeDropdown();
      handled = false;
    } else handled = false;
    if (handled) {
      e.preventDefault();
      e.stopPropagation();
    }
  },
  true,
);

addEventListener("blur", closeDropdown);
addEventListener("resize", closeDropdown);
document.addEventListener("scroll", (e) => e.target !== dropdown.menu && closeDropdown(), true);

// The other interfaces and shortcut.html reuse this file for its helpers
// and run their own start-up.
if (view && uiStyle() !== "classic") setUiStyle(uiStyle());
else if (view) init();
