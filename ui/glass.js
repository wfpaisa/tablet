// Glass uses the shared configuration, action catalog and persistence helpers.
const glassNav = [
  ["pen", "Pen", "sidebar-pen"],
  ["keys", "ExpressKeys", "sidebar-expresskeys"],
  ["mapping", "Mapping", "sidebar-mapping"],
  ["gestures", "Gestures", "gesture-pan"],
  ["shortcuts/express-menu", "On-screen shortcuts", "radial"],
  ["info", "Device info", "info"],
  ["settings", "Settings", "nav-settings"],
];
let glassRoute = "",
  glassUndo,
  glassPopover,
  glassPoll,
  glassTick = false;
const gt = (s) => esc(t(s));
const gb = (label, attrs = "") => `<button ${attrs}>${gt(label)}</button>`;
const glassSection = (title, body, hint = "") =>
  `<section class="card"><h2>${gt(title)}</h2>${hint ? `<p class="hint">${gt(hint)}</p>` : ""}${body}</section>`;
const glassProfile = path => /^(pen|express_keys)\./.test(path) ? toolProfile() : config;
const glassGet = (path) => path.split(".").reduce((o, k) => o?.[k], glassProfile(path));
function glassSet(path, value) {
  const keys = path.split("."),
    last = keys.pop();
  keys.reduce((o, k) => o[k], glassProfile(path))[last] = value;
}
function glassChanged(change, redraw = false) {
  glassUndo = structuredClone(config);
  change();
  save();
  glassNotice("Settings saved", () => {
    config = glassUndo;
    glassUndo = null;
    save();
    glassRender();
    glassNotice("Changes undone");
  });
  if (redraw) glassRender();
}
function glassNotice(message, undo) {
  const el = $("#toast");
  clearTimeout(toast.timer);
  el.innerHTML = `<span>${gt(message)}</span>${undo ? gb("Undo", "data-undo") : ""}`;
  el.hidden = false;
  $("[data-undo]", el)?.addEventListener("click", undo, { once: true });
  toast.timer = setTimeout(() => {
    el.hidden = true;
  }, 5500);
}
function glassRange(label, path, max = 5, ends) {
  const [low, high] = ends ?? (max === 7 ? ["Soft", "Firm"] : /size$/.test(path) ? ["Small", "Large"] : ["Slow", "Fast"]);
  return `<label class="range-row"><span>${gt(label)} <output>${esc(glassGet(path))}</output></span><input type="range" min="1" max="${max}" value="${glassGet(path)}" data-range="${path}" aria-label="${gt(label)}"><span class="range-ends"><small>${gt(low)}</small><small>${gt(high)}</small></span></label>`;
}
function glassSwitch(label, path, checked = glassGet(path)) {
  return `<label class="switch-row"><span>${gt(label)}</span><input role="switch" type="checkbox" data-switch="${path}" ${checked ? "checked" : ""}><span class="switch-track"></span></label>`;
}
function glassTablet() {
  return `<img src="${esc(tabletSource())}" alt="${esc(model().name)}" style="width:100%">`;
}

function glassPen() {
  return (
    glassSection(
      "Pen buttons",
      `<div class="pen-stage"><svg viewBox="0 0 570 130" aria-hidden="true"><path d="M48 52Q25 65 48 78L458 78 514 65 458 52Z" class="device-fill"/><path d="M450 52v26m36-7 28-6-28-6" class="device-line"/><rect x="295" y="53" width="88" height="24" rx="10" class="device-area"/></svg>${["upper", "lower"].map((key, i) => `<button class="pen-hotspot hotspot" style="left:${i ? 62 : 50}%" data-pen="${key}" data-action-path="pen.${key}_button" aria-label="${gt(i ? "Lower button" : "Upper button")}"></button>`).join("")}</div><div class="two-col">${["upper", "lower"].map((key, i) => `<button class="action-row" data-pen="${key}" data-action-path="pen.${key}_button"><span>${gt(i ? "Lower button" : "Upper button")}</span><strong>${actionIcon(toolProfile().pen[`${key}_button`])}${esc(actionLabel(toolProfile().pen[`${key}_button`]))}</strong></button>`).join("")}</div>${[toolProfile().pen.upper_button, toolProfile().pen.lower_button].includes("right_or_hold_middle") ? glassRange("Hold time", "pen.hold_time", 3, ["Short", "Long"]) + `<p class="hint">${gt(HOLD_TIME_HINT)}</p>` : ""}`,
      "Press a pen button to identify it, then choose its action.",
    ) +
    glassSection(
      "Tip feel",
      `${glassRange("Tip sensitivity", "pen.tip_sensitivity", 7)}${glassRange("Tip feel", "pen.click_threshold", 7, ["Light", "Firm"])}<label class="range-row"><span>${gt("Tip double-click")} · ${gt("Double-click distance")}</span><input type="range" min="0" max="5" value="${DOUBLE_CLICK_MM.indexOf(toolProfile().pen.double_click_mm)}" id="glass-double-click" aria-label="${gt("Double-click distance")}"><span class="range-ends"><small>${gt("Off")}</small><small>${gt("Large")}</small></span></label><div class="test-head"><span>${gt("Try your pen")}</span><span id="pressure">0%</span>${gb("Clear", 'id="clear-test"')}</div><canvas id="pen-test" aria-label="${gt("Drawing test area")}"></canvas><div class="pressure-track"><span id="pressure-fill"></span></div>`,
    ) +
    glassSection(
      "Pen mode",
      `${glassSwitch("Use the pen as a mouse", "pen.mode", toolProfile().pen.mode === "mouse")}<p class="hint">${gt(MOUSE_MODE_HINT)}</p><footer>${gb("Default", 'id="pen-default"')}</footer>`,
    )
  );
}
function glassKeys() {
  return glassSection(
    "ExpressKeys",
    `<div class="tablet-stage">${glassTablet()}<div class="tablet-keys" style="inset:0;display:block">${Array.from({ length: model().keys }, (_, i) => `<button class="hotspot" style="position:absolute;left:${keySpots()[i][0] * 100}%;top:${keySpots()[i][1] * 100}%;transform:translate(-50%,-50%)" data-key="${i}" data-action-path="express_keys.${i}" aria-label="${gt("ExpressKey")} ${i + 1}">${i + 1}</button>`).join("")}</div></div><div class="two-col">${Array.from({ length: model().keys }, (_, i) => `<button class="action-row" data-key="${i}" data-action-path="express_keys.${i}"><span>${gt("ExpressKey")} ${i + 1}</span><strong>${itemIcon({ action: toolProfile().express_keys[i] })}${esc(keyLabel(toolProfile().express_keys[i]))}</strong></button>`).join("")}</div><footer>${gb("Default", 'id="keys-default"')}</footer>`,
    "Press an ExpressKey to identify it, then choose its action.",
  );
}
function glassMapping() {
  const displays = status.displays ?? [],
    minX = Math.min(0, ...displays.map((d) => d.x)),
    minY = Math.min(0, ...displays.map((d) => d.y));
  const width = Math.max(1, ...displays.map((d) => d.x + d.width)) - minX,
    height = Math.max(1, ...displays.map((d) => d.y + d.height)) - minY;
  return (
    glassSection(
      "Mapping",
      `<div class="monitor-layout" style="aspect-ratio:${width}/${height}">${displays.map((d, i) => `<button class="monitor ${config.mapping.display === d.id ? "selected" : ""}" data-display="${i}" style="left:${((d.x - minX) / width) * 100}%;top:${((d.y - minY) / height) * 100}%;width:${(d.width / width) * 100}%;height:${(d.height / height) * 100}%"><svg viewBox="0 0 60 42" aria-hidden="true"><rect x="4" y="2" width="52" height="32" rx="3"/><path d="M30 34v6m-12 0h24"/></svg><strong>${esc(d.name || d.connector)}</strong><small>${d.width} × ${d.height}</small></button>`).join("")}</div>${gb("All displays", `id="all-displays" aria-pressed="${config.mapping.display === null}"`)}${glassSwitch("Force proportions", "mapping.force_proportions")}<p class="hint">${gt(FORCE_HELP)}</p>`,
    ) +
    glassSection(
      "Display toggle",
      `<p>${gt("Assign Display toggle to a pen button or ExpressKey to cycle between your displays and all displays.")}</p><p>${gt("Current target")} <strong id="toggle-target"></strong></p>`,
    )
  );
}
function glassGestures() {
  return GESTURES.map(([id, title]) => {
    const page = GESTURE_PAGES[id];
    return glassSection(
      title,
      `${glassRange("Speed", page.speed.join("."))}${id === "pan-scroll" ? glassSwitch("Content follows the pen", "pan_scroll.content_follows_pen") : ""}`,
      page.hint,
    );
  }).join("");
}
function glassShortcuts() {
  const sub = glassRoute.split("/")[1],
    page = ON_SCREEN[sub] ?? ON_SCREEN["express-menu"],
    menu = config.shortcuts[page.key];
  return (
    `<nav class="tabs">${Object.entries(ON_SCREEN)
      .map(
        ([route, p]) =>
          `<a href="#/shortcuts/${route}" class="${p.key === page.key ? "active" : ""}">${gt(p.title)}</a>`,
      )
      .join("")}</nav>` +
    glassSection(
      page.title,
      `<div class="preview">${onScreenMenu(page.key)}</div>${
        page.kind !== "keypad"
          ? `<div class="menu-entries">${menuItems(page.key)
              .map(
                (item, i) =>
                  `<div class="entry"><span class="entry-number">${i + 1}</span><input aria-label="${gt("Label")} ${i + 1}" data-label="${i}" data-menu="${page.key}" value="${esc(item.label)}" placeholder="${esc(itemLabel(item))}"><button data-action-path="shortcuts.${page.key}.items.${i}.action">${itemIcon(item)}${esc(itemLabel(item))}</button></div>`,
              )
              .join("")}</div>`
          : ""
      }${glassRange("Size", `shortcuts.${page.key}.size`)}${glassRange("Text size", `shortcuts.${page.key}.text_size`)}${glassSwitch("Display at cursor", `shortcuts.${page.key}.at_cursor`)}<footer>${gb("Default", `data-menu-default="${page.key}"`)}</footer>`,
    )
  );
}
function glassSettings() {
  return (
    glassSection(
      "General settings",
      `<div class="setting-row"><div><h3>${gt("Driver service")}</h3><p id="driver-state"></p></div>${gb("Restart Driver", "data-restart")}</div><div class="setting-row"><label for="language">${gt("Language")}</label><select id="language">${LANGUAGES.map(([code, label]) => `<option value="${code}" ${language() === code ? "selected" : ""}>${esc(label)}</option>`).join("")}</select></div><div class="setting-row"><label for="ui-style">${gt("Interface")}</label>${uiStyleSelect()}</div>${glassSwitch("Tablet Center autostart", "autostart", false)}<label class="switch-row"><span>${gt("On-screen shortcuts in GNOME Shell")}</span><input role="switch" type="checkbox" id="in-shell" ${config.shortcuts?.in_shell ? "checked" : ""}><span class="switch-track"></span></label><p class="hint">${gt(SHELL_MENUS_HINT)}</p>`,
    ) +
    glassSection(
      "Backup & restore",
      `<div class="backup-actions">${gb("New backup", 'id="new-backup"')}${gb("Open backup folder", 'id="open-backups"')}${gb("Reset all settings", 'id="reset-config"')}</div><div id="glass-backups"></div>`,
    )
  );
}
function glassRender() {
  glassClosePopover();
  glassRoute = location.hash.replace(/^#\//, "") || "pen";
  const section = glassRoute.split("/")[0];
  document
    .querySelectorAll(".side-nav a")
    .forEach((el) => el.classList.toggle("active", el.dataset.route.split("/")[0] === section));
  $("#page-title").textContent = t(glassNav.find(([r]) => r.split("/")[0] === section)?.[1] ?? "Pen");
  const content = $("#glass-content");
  content.innerHTML =
    section === "settings"
      ? glassSettings()
      : !status.device
        ? glassSection(
            "Connect your tablet",
            `<div class="empty-device">${glassTablet()}</div><p>${gt("Connect your tablet to configure your pen, ExpressKeys and shortcuts.")}</p>`,
          )
        : (
            {
              info: () => deviceInfoPage(false),
              pen: glassPen,
              keys: glassKeys,
              mapping: glassMapping,
              gestures: glassGestures,
              shortcuts: glassShortcuts,
            }[section] ?? glassPen
          )();
  glassBind();
  glassUpdateStatus();
  glassSchedule(0);
}
function glassClosePopover() {
  if (!glassPopover) return;
  const { element, anchor } = glassPopover;
  element.remove();
  anchor?.setAttribute("aria-expanded", "false");
  glassPopover = null;
}
function glassActions(anchor) {
  glassClosePopover();
  const path = anchor.dataset.actionPath,
    pen = path.startsWith("pen."),
    groups = pen
      ? [...PEN_GROUPS, ["Actions", ACTIONS.map(([v]) => v)]]
      : path.startsWith("express_keys")
        ? keyGroups()
        : menuGroups();
  const current = pen ? glassGet(path) : keyValue(glassGet(path));
  const pop = document.createElement("div");
  pop.className = "action-popover";
  pop.setAttribute("role", "dialog");
  pop.setAttribute("aria-label", t("Choose an action"));
  pop.innerHTML = `<input class="action-filter" placeholder="${gt("Search actions")}" aria-label="${gt("Search actions")}"><div class="action-list">${groups.map(([title, values]) => `<section><h3>${gt(title)}</h3>${values.map((v) => `<button data-value="${v}" aria-pressed="${v === current}">${actionIcon(v)}${esc(actionLabel(v))}${v === current ? '<span class="check">✓</span>' : ""}</button>`).join("")}</section>`).join("")}</div>`;
  document.body.append(pop);
  const r = anchor.getBoundingClientRect();
  pop.style.left = `${Math.max(12, Math.min(r.left, innerWidth - 332))}px`;
  pop.style.top = `${Math.max(52, Math.min(r.bottom + 8, innerHeight - Math.min(460, innerHeight - 76) - 12))}px`;
  anchor.setAttribute("aria-expanded", "true");
  glassPopover = { element: pop, anchor };
  $("input", pop).focus();
  $("input", pop).addEventListener("input", (e) => {
    pop.querySelectorAll("[data-value]").forEach((b) => {
      b.hidden = !b.textContent.toLowerCase().includes(e.target.value.toLowerCase());
    });
    pop.querySelectorAll("section").forEach((s) => {
      s.hidden = !s.querySelector("button:not([hidden])");
    });
  });
  pop.addEventListener("keydown", (e) => {
    const buttons = [...pop.querySelectorAll("button:not([hidden])")];
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      buttons[
        (buttons.indexOf(document.activeElement) + (e.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length
      ]?.focus();
    }
    if (e.key === "Enter" && e.target.tagName === "INPUT") {
      e.preventDefault();
      buttons[0]?.click();
    }
    if (e.key === "Tab") {
      const nodes = [$("input", pop), ...buttons];
      e.preventDefault();
      nodes[(nodes.indexOf(document.activeElement) + (e.shiftKey ? -1 : 1) + nodes.length) % nodes.length]?.focus();
    }
  });
  pop.addEventListener("click", (e) => {
    const button = e.target.closest("[data-value]");
    if (!button) return;
    const value = button.dataset.value;
    const apply = (action) =>
      glassChanged(() => {
        if (path.includes(".items.")) {
          const key = path.split(".")[1];
          config.shortcuts[key].items = menuItems(key);
        }
        glassSet(path, action);
      }, true);
    glassClosePopover();
    if (pen) apply(value);
    else if (value === "record") glassRecord(apply, anchor);
    else if (value.startsWith("preset:")) {
      const [label, keys] = KEY_PRESETS[Number(value.slice(7))];
      apply({ type: "keystroke", keys: [...keys], label: t(label) });
    } else apply({ type: value });
  });
}
function glassRecord(apply, anchor) {
  recordKeystroke(apply);
  const backdrop = document.querySelector(".dialog-backdrop");
  backdrop.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Escape") {
        e.preventDefault();
        e.stopPropagation();
        backdrop.remove();
        anchor?.focus();
      }
      if (e.key === "Tab") {
        e.stopPropagation();
        const nodes = [...backdrop.querySelectorAll(".capture, button:not(:disabled)")];
        e.preventDefault();
        nodes[(nodes.indexOf(document.activeElement) + (e.shiftKey ? -1 : 1) + nodes.length) % nodes.length]?.focus();
      }
    },
    true,
  );
}
function glassBind() {
  $("#glass-double-click")?.addEventListener("change", e =>
    glassChanged(() => { toolProfile().pen.double_click_mm = DOUBLE_CLICK_MM[Number(e.target.value)]; }));
  $("#pen-default")?.addEventListener("click", () =>
    glassChanged(() => {
      toolProfile().pen = structuredClone(DEFAULTS.pen);
    }, true),
  );
  document.querySelectorAll("[data-action-path]").forEach((el) => el.addEventListener("click", () => glassActions(el)));
  document.querySelectorAll("[data-range]").forEach((el) => {
    el.addEventListener("input", () => {
      el.previousElementSibling.querySelector("output").value = el.value;
    });
    el.addEventListener("change", () => {
      glassChanged(() => glassSet(el.dataset.range, Number(el.value)));
      glassRefreshPreview();
    });
  });
  document.querySelectorAll("[data-switch]").forEach((el) =>
    el.addEventListener("change", async () => {
      if (el.dataset.switch === "autostart") {
        const previous = !el.checked;
        try {
          await invoke("set_autostart", { enabled: el.checked });
          glassNotice("Settings saved", async () => {
            await invoke("set_autostart", { enabled: previous });
            el.checked = previous;
            glassNotice("Changes undone");
          });
        } catch (error) {
          el.checked = previous;
          toast(String(error));
        }
      } else
        glassChanged(() =>
          glassSet(el.dataset.switch, el.dataset.switch === "pen.mode" ? (el.checked ? "mouse" : "pen") : el.checked),
        );
    }),
  );
  $("#keys-default")?.addEventListener("click", () =>
    glassChanged(() => {
      toolProfile().express_keys = Array.from({ length: model().keys }, () => ({ type: "disabled" }));
    }, true),
  );
  document.querySelectorAll("[data-display]").forEach((el) =>
    el.addEventListener("click", () =>
      glassChanged(() => {
        config.mapping.display = status.displays[Number(el.dataset.display)].id;
      }, true),
    ),
  );
  $("#all-displays")?.addEventListener("click", () =>
    glassChanged(() => {
      config.mapping.display = null;
    }, true),
  );
  document.querySelectorAll("[data-label]").forEach((el) =>
    el.addEventListener("change", () =>
      glassChanged(() => {
        const menu = config.shortcuts[el.dataset.menu];
        menu.items = menuItems(el.dataset.menu);
        menu.items[Number(el.dataset.label)].label = el.value.trim();
      }, true),
    ),
  );
  $("[data-menu-default]")?.addEventListener("click", (e) =>
    glassChanged(() => {
      const key = e.currentTarget.dataset.menuDefault;
      config.shortcuts[key] = structuredClone(DEFAULTS.shortcuts[key]);
    }, true),
  );
  glassBindPreview();
  glassBindCanvas();
  if (glassRoute === "settings") glassBindSettings();
}
function glassBindPreview() {
  document.querySelectorAll(".preview [data-item]").forEach((el) => {
    if (glassRoute.endsWith("keypad")) return;
    const index = el.dataset.item;
    el.setAttribute("tabindex", "0");
    el.setAttribute("role", "button");
    el.setAttribute("aria-label", `${t("Edit shortcut")} ${Number(index) + 1}`);
    const open = () => {
      const button = $(`.entry:nth-child(${Number(index) + 1}) button`);
      button?.scrollIntoView({ block: "nearest" });
      if (button) glassActions(button);
    };
    el.addEventListener("click", open);
    el.addEventListener("keydown", (e) => {
      if (["Enter", " "].includes(e.key)) {
        e.preventDefault();
        open();
      }
    });
  });
}
function glassRefreshPreview() {
  if (!$(".preview")) return;
  const page = ON_SCREEN[glassRoute.split("/")[1]] ?? ON_SCREEN["express-menu"];
  $(".preview").innerHTML = onScreenMenu(page.key);
  glassBindPreview();
}
function glassBindCanvas() {
  const canvas = $("#pen-test");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  let last;
  const resize = () => {
    const r = canvas.getBoundingClientRect();
    canvas.width = r.width * devicePixelRatio;
    canvas.height = r.height * devicePixelRatio;
    ctx.scale(devicePixelRatio, devicePixelRatio);
  };
  resize();
  canvas.addEventListener("pointerdown", (e) => {
    canvas.setPointerCapture(e.pointerId);
    last = [e.offsetX, e.offsetY];
  });
  canvas.addEventListener("pointermove", (e) => {
    if (!last || !e.buttons) return;
    ctx.beginPath();
    ctx.moveTo(...last);
    ctx.lineTo(e.offsetX, e.offsetY);
    ctx.lineWidth = 1 + (e.pointerType === "pen" ? e.pressure : status.pen?.pressure || 0.25) * 13;
    ctx.strokeStyle = "#3584e4";
    ctx.lineCap = "round";
    ctx.stroke();
    last = [e.offsetX, e.offsetY];
  });
  canvas.addEventListener("pointerup", () => {
    last = null;
  });
  canvas.addEventListener("pointercancel", () => {
    last = null;
  });
  $("#clear-test").addEventListener("click", () => ctx.clearRect(0, 0, canvas.width, canvas.height));
}
async function glassDialog(title, action, input = false) {
  return settingsDialog(title, action, input);
}
async function glassBackups() {
  const target = $("#glass-backups");
  if (!target) return;
  const backups = await invoke("list_backups");
  if (!target.isConnected) return;
  target.innerHTML = backups.length
    ? backups
        .map(
          (b) =>
            `<div class="backup-row"><div><strong>${esc(b.title)}</strong><small>${esc(new Date(b.created * 1000).toLocaleString(language()))}</small></div>${gb("Restore", `data-restore="${esc(b.filename)}"`)}${gb("Delete", `data-delete="${esc(b.filename)}"`)}</div>`,
        )
        .join("")
    : `<p class="hint">${gt("No backups yet.")}</p>`;
  target.querySelectorAll("[data-restore], [data-delete]").forEach((el) =>
    el.addEventListener("click", () =>
      glassSafe(async () => {
        const restore = el.hasAttribute("data-restore");
        if (
          !(await glassDialog(
            restore
              ? "Restore this backup? Current settings will be replaced."
              : "Delete this backup? This cannot be undone.",
            restore ? "Restore" : "Delete",
          ))
        )
          return;
        if (restore)
          await glassReplaceConfig("restore_backup", {
            filename: el.dataset.restore,
          });
        else {
          await invoke("delete_backup", { filename: el.dataset.delete });
          await glassBackups();
          glassNotice("Backup deleted");
        }
      }),
    ),
  );
}
async function glassSafe(fn) {
  try {
    await fn();
  } catch (error) {
    toast(String(error));
  }
}
async function glassReplaceConfig(command, args = {}) {
  const previous = structuredClone(config);
  clearTimeout(saveTimer);
  await invoke(command, args);
  await loadConfig();
  glassRender();
  glassUndo = previous;
  glassNotice("Settings saved", () => {
    config = previous;
    save();
    glassRender();
    glassNotice("Changes undone");
  });
}
function glassBindSettings() {
  mountUiStyleSelect();
  $("#language").addEventListener("change", (e) => {
    localStorage.setItem("language", e.target.value);
    location.reload();
  });
  const auto = $('[data-switch="autostart"]');
  auto.disabled = true;
  glassSafe(async () => {
    auto.checked = await invoke("autostart_enabled");
    auto.disabled = false;
  });
  $("#new-backup").addEventListener("click", () =>
    glassSafe(async () => {
      const title = await glassDialog("New backup", "Create", true);
      if (!title) return;
      clearTimeout(saveTimer);
      await invoke("save_config", { config });
      await invoke("create_backup", { title });
      await glassBackups();
      glassNotice("Backup created");
    }),
  );
  $("#open-backups").addEventListener("click", () => glassSafe(() => invoke("open_backups_folder")));
  $("#reset-config").addEventListener("click", () =>
    glassSafe(async () => {
      if (await glassDialog("Reset all settings to their defaults?", "Reset all settings"))
        await glassReplaceConfig("reset_config");
    }),
  );
  glassSafe(glassBackups);
}
function glassUpdateStatus() {
  const put = (selector, value) => {
    const el = $(selector);
    if (el && el.textContent !== value) el.textContent = value;
  };
  put("#device-name", status.device ? `${model().name} ${model().model}` : t("No tablet connected"));
  const battery = $("#battery-status");
  if (battery) battery.innerHTML = batteryIndicator();
  updateDeviceInfo();
  const banner = $("#daemon-banner");
  banner.hidden = !status.error;
  put("#daemon-message", t("The driver is not running. Restart it to reconnect your tablet."));
  put("#driver-state", t(status.error ? "Not running" : "Running"));
  put(
    "#toggle-target",
    String(
      status.toggled_display == null
        ? t("All displays")
        : ((status.displays ?? []).find((d) => d.id === status.toggled_display)?.name ?? status.toggled_display),
    ),
  );
  document
    .querySelectorAll("[data-pen]")
    .forEach((el) => el.classList.toggle("held", !!status.pen?.[`${el.dataset.pen}_button`]));
  document
    .querySelectorAll("[data-key]")
    .forEach((el) => el.classList.toggle("held", !!status.express_keys?.[Number(el.dataset.key)]));
  const pressure = Math.round((status.pen?.pressure ?? 0) * 100);
  put("#pressure", `${pressure}%`);
  const fill = $("#pressure-fill");
  if (fill && fill.style.width !== `${pressure}%`) fill.style.width = `${pressure}%`;
}
function glassLiveView() {
  const main = $("main")?.getBoundingClientRect();
  if (!main) return false;
  return [...document.querySelectorAll("[data-pen], [data-key], #pen-test, #info-pressure")].some((el) => {
    const r = el.getBoundingClientRect();
    return r.bottom > main.top && r.top < main.bottom && r.right > main.left && r.left < main.right;
  });
}
function glassSchedule(delay) {
  clearTimeout(glassPoll);
  if (document.visibilityState !== "visible") return;
  glassPoll = setTimeout(glassPollStatus, delay ?? (glassLiveView() ? 50 : 1000));
}
async function glassPollStatus() {
  if (glassTick || document.visibilityState !== "visible") return;
  glassTick = true;
  try {
    const before = !!status.device,
      displays = JSON.stringify(status.displays);
    status = await invoke("status");
    if (before !== !!status.device) glassRender();
    else {
      if (glassRoute === "mapping" && displays !== JSON.stringify(status.displays) && $(".monitor-layout")) {
        const template = document.createElement("template");
        template.innerHTML = glassMapping();
        $(".monitor-layout").replaceWith(template.content.querySelector(".monitor-layout"));
        document.querySelectorAll("[data-display]").forEach((el) =>
          el.addEventListener("click", () =>
            glassChanged(() => {
              config.mapping.display = status.displays[Number(el.dataset.display)].id;
            }, true),
          ),
        );
      }
      glassUpdateStatus();
    }
  } catch (error) {
    status.error = String(error);
    glassUpdateStatus();
  } finally {
    glassTick = false;
    glassSchedule();
  }
}
function glassSearch() {
  const input = $("#glass-search"),
    results = $("#search-results");
  const entries = [
    ...glassNav.map(([route, label]) => [route, label]),
    ...GESTURES.map(([, label]) => ["gestures", label]),
    ...Object.entries(ON_SCREEN).map(([route, p]) => [`shortcuts/${route}`, p.title]),
    ...["Tip sensitivity", "Use the pen as a mouse", "Tip feel"].map((s) => ["pen", s]),
    ...["Force proportions", "Display toggle", "All displays"].map((s) => ["mapping", s]),
    ...["Language", "Interface", "Backup & restore", "Tablet Center autostart", "Restart Driver"].map((s) => [
      "settings",
      s,
    ]),
    ...ACTIONS.map(([, label]) => ["pen", label]),
  ];
  input.addEventListener("input", () => {
    const query = input.value.trim().toLowerCase();
    results.hidden = !query;
    const found = entries.filter(([, label]) => t(label).toLowerCase().includes(query));
    results.innerHTML = found.length
      ? found
          .slice(0, 12)
          .map(([route, label]) => `<a href="#/${route}">${gt(label)}</a>`)
          .join("")
      : `<p>${gt("No results")}</p>`;
  });
  const close = () => {
    results.hidden = true;
    input.value = "";
  };
  results.addEventListener("click", close);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      $("a", results)?.click();
      close();
      input.blur();
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      $("a", results)?.focus();
    }
    if (e.key === "Escape") close();
  });
}
async function glassInit() {
  await loadLanguage();
  await loadConfig();
  saveShellMenus();
  try {
    status = await invoke("status");
  } catch (error) {
    status = { device: null, error: String(error) };
  }
  $("#glass").innerHTML =
    `<div class="wf-window">${windowFrame()}<div class="search-wrap"><input id="glass-search" type="search" placeholder="${gt("Search settings")}" aria-label="${gt("Search settings")}"><div id="search-results" hidden></div></div><div class="app-layout"><aside><div class="device-card"><div class="mini-tablet">${glassTablet()}</div><strong id="device-name"></strong><div id="battery-status"></div><a href="#/info">${gt("Device info")}</a><small>${gt("Tablet Center")} · Glass</small></div><nav class="side-nav" aria-label="${gt("Settings")}">${glassNav.map(([route, label, ic]) => `<a data-route="${route}" href="#/${route}">${icon(ic)}${gt(label)}</a>`).join("")}</nav></aside><main><div id="daemon-banner" hidden><span id="daemon-message"></span>${gb("Restart Driver", "data-restart")}</div><header class="page-heading"><small>${gt("Tablet Center")}</small><h1 id="page-title"></h1></header><div id="glass-content"></div></main></div></div>`;
  mountWindowFrame();
  glassSearch();
  document.addEventListener("click", (e) => {
    if (e.target.closest("[data-restart]"))
      glassSafe(async () => {
        await invoke("restart_daemon");
        glassNotice("Driver restarted");
        glassSchedule(0);
      });
    if (glassPopover && !glassPopover.element.contains(e.target) && !e.target.closest("[data-action-path]"))
      glassClosePopover();
    if (!e.target.closest(".search-wrap")) $("#search-results").hidden = true;
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") $("#search-results").hidden = true;
    if (e.key === "Escape" && glassPopover) {
      const anchor = glassPopover.anchor;
      glassClosePopover();
      anchor?.focus();
    }
  });
  window.addEventListener("hashchange", glassRender);
  document.addEventListener("visibilitychange", () => glassSchedule(0));
  glassRender();
}
glassInit().catch((error) => toast(String(error)));
