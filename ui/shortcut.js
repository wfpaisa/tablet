// On-screen shortcut window, kept hidden by `tablet-center --shortcut-host`
// and shown by it with showShortcut({ name, at, monitor, centered }): `at`
// is the pen inside its monitor and `monitor` that monitor's size, or both
// null; the menu is centred then, and when `centered` (ExpressKeys).
// `mapping` says the window is about to be mapped.
// Helpers (rendering, labels, translations) come from app.js.

// Closing zooms the menu out first; the window hides once it is invisible,
// so GNOME Shell's unmap animation has nothing left to move.
const HIDE_MS = 120;
let hiding = null;

function hideMenu(then) {
  const wrap = document.querySelector(".overlay-menu");
  if (!wrap?.classList.contains("shown")) return then();
  wrap.classList.remove("shown");
  clearTimeout(hiding);
  hiding = setTimeout(then, HIDE_MS);
}

const closeShortcut = () => hideMenu(() => invoke("close_shortcut").catch(() => window.close()));
window.hideShortcut = closeShortcut;
let items = [];
let ready = loadLanguage();

window.showShortcut = async ({ name, at, monitor, centered, mapping }) => {
  // A new menu replaces one that was still closing.
  clearTimeout(hiding);
  await ready;
  // Settings may have changed since the last time it was shown.
  await loadConfig();
  if (!onScreenByKey(name)) return closeShortcut();
  const menu = config.shortcuts[name];
  items = menuItems(name);
  document.querySelector(".overlay-menu")?.remove();
  const wrap = document.createElement("div");
  wrap.className = "overlay-menu";
  wrap.innerHTML = onScreenMenu(name);
  document.body.append(wrap);

  // The maximized window leaves out GNOME's top bar (and any side panel),
  // so it starts that far from the monitor's corner.
  const place = () => {
    const { width, height } = wrap.getBoundingClientRect();
    const [x, y] =
      menu.at_cursor && !centered && at && monitor
        ? [at[0] - (monitor[0] - innerWidth), at[1] - (monitor[1] - innerHeight)]
        : [innerWidth / 2, innerHeight / 2];
    // Keep the whole menu on screen when the pen is near an edge.
    const clamp = (v, size, max) => Math.min(Math.max(v, size / 2 + 8), max - size / 2 - 8);
    wrap.style.left = `${clamp(x, width, innerWidth)}px`;
    wrap.style.top = `${clamp(y, height, innerHeight)}px`;
  };
  // The window may still be growing to its maximized size: wait until it
  // holds still, then place the menu once and fade it in where it stays.
  let last = "";
  for (let i = 0; i < 20; i++) {
    await new Promise(requestAnimationFrame);
    const size = `${innerWidth}x${innerHeight}`;
    if (size === last && innerWidth > 200) break;
    last = size;
  }
  place();
  onresize = place;
  // GNOME Shell grows a newly mapped window from its bottom edge for
  // 150 ms; the menu stays invisible until that is over, then zooms in.
  if (mapping) await new Promise((done) => setTimeout(done, 160));
  wrap.classList.add("shown");
};

document.addEventListener("click", (e) => {
  if (!e.target.closest(".overlay-menu")) return closeShortcut();
  if (e.target.closest("[data-close]")) return closeShortcut();
  const item = items[Number(e.target.closest("[data-item]")?.dataset.item)];
  if (item && item.action.type !== "disabled") hideMenu(() => invoke("run_shortcut_action", { action: item.action }));
});

// Escape, or leaving for another window, closes it.
addEventListener("keydown", (e) => e.key === "Escape" && closeShortcut());
let focused = false;
addEventListener("focus", () => (focused = true));
addEventListener("blur", () => {
  if (focused) closeShortcut();
  focused = false;
});

// A request may have come in while this page was loading. ?name= lets the
// page be previewed in a browser.
(async () => {
  const preview = new URLSearchParams(location.search).get("name");
  const pending = preview ? { name: preview, at: null, monitor: null } : await invoke("pending_shortcut");
  if (pending) showShortcut(pending);
})();
