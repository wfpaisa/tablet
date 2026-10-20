// WebKitGTK moves the page a whole step per wheel notch (80 px), so turning
// the wheel looks like jumps. Here the page follows the wheel through a
// critically damped spring: its speed changes smoothly from notch to
// notch instead of starting and stopping at each one.
(() => {
  const SETTLE_MS = 300, // time to cover 90% of a notch
    LINE = 40, // pixels per line when the wheel reports lines
    FRAME_MS = 1000 / 60;
  const W = 3.89 / SETTLE_MS;
  let active = null; // { el, target, pos, vel, time, frame }

  function scroller(node, dy) {
    for (let el = node; el && el !== document.documentElement; el = el.parentElement) {
      if (!(el instanceof Element)) continue;
      if (!/(auto|scroll)/.test(getComputedStyle(el).overflowY) || el.scrollHeight <= el.clientHeight) continue;
      const max = el.scrollHeight - el.clientHeight;
      if ((dy < 0 && el.scrollTop > 0) || (dy > 0 && el.scrollTop < max)) return el;
    }
    const root = document.scrollingElement;
    return root && root.scrollHeight > root.clientHeight ? root : null;
  }

  function step(now) {
    const s = active;
    // Something else moved it (scrollbar, keyboard): stop following.
    if (Math.abs(s.el.scrollTop - s.pos) > 2) return (active = null);
    // WebKit rounds frame times to whole milliseconds (16, 17); stepping
    // by whole frames keeps that jitter out of the speed.
    const frames = Math.max(1, Math.min(3, Math.round((now - s.time) / FRAME_MS)));
    const dt = frames * FRAME_MS;
    s.time = now;
    for (let i = 0; i < 4; i++) {
      const h = dt / 4;
      s.vel += (W * W * (s.target - s.pos) - 2 * W * s.vel) * h;
      s.pos += s.vel * h;
    }
    const max = s.el.scrollHeight - s.el.clientHeight;
    if (s.pos <= 0 || s.pos >= max) s.vel = 0;
    s.pos = Math.max(0, Math.min(max, s.pos));
    if (Math.abs(s.target - s.pos) < 0.5 && Math.abs(s.vel) < 0.02) s.pos = s.target;
    s.el.scrollTop = Math.round(s.pos);
    if (s.pos === s.target) active = null;
    else s.frame = requestAnimationFrame(step);
  }

  addEventListener(
    "wheel",
    (e) => {
      if (e.ctrlKey || e.defaultPrevented || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
      const dy = e.deltaY * (e.deltaMode === 1 ? LINE : e.deltaMode === 2 ? innerHeight : 1);
      if (!dy) return;
      const el = active && active.el.contains(e.target) ? active.el : scroller(e.target, dy);
      if (!el) return;
      e.preventDefault();
      if (active && active.el !== el) {
        cancelAnimationFrame(active.frame);
        active = null;
      }
      if (!active) {
        active = { el, target: el.scrollTop, pos: el.scrollTop, vel: 0, time: performance.now() };
        active.frame = requestAnimationFrame(step);
      }
      // Turning the other way drops what was still pending.
      if (Math.sign(dy) !== Math.sign(active.target - active.pos)) {
        active.target = active.pos;
        active.vel = 0;
      }
      active.target = Math.max(0, Math.min(el.scrollHeight - el.clientHeight, active.target + dy));
    },
    { passive: false },
  );
})();
