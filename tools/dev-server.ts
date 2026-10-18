// Serves ui/ for dev.sh with caching off, so the WebView never keeps an old
// copy of a page, script or stylesheet.
import { join, normalize } from "node:path";

const root = join(import.meta.dir, "..", "ui");

Bun.serve({
  hostname: "127.0.0.1",
  port: 8765,
  async fetch(req) {
    const path = normalize(decodeURIComponent(new URL(req.url).pathname));
    if (path.includes("..")) return new Response("Forbidden", { status: 403 });
    const file = Bun.file(join(root, path === "/" ? "index.html" : path));
    if (!(await file.exists())) return new Response("Not found", { status: 404 });
    return new Response(file, { headers: { "Cache-Control": "no-store" } });
  },
});
