#!/usr/bin/env node

import { createRequire } from "node:module";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = path.resolve(import.meta.dirname, "..");
const require = createRequire(path.join(root, "docs-site/package.json"));
const { chromium } = require("playwright");
const { parse: parseToml } = require("smol-toml");
const source = path.join(root, "tmp/Terminal theme configuration review/Variant Board.dc.html");
const output = path.join(root, "docs-site/static/img/themes/variants");
await mkdir(output, { recursive: true });

const browser = await chromium.launch({ headless: true, args: ["--allow-file-access-from-files"] });
const page = await browser.newPage({ viewport: { width: 1360, height: 900 }, deviceScaleFactor: 1 });
await page.goto(pathToFileURL(source).href);
await page.waitForFunction(() => document.querySelectorAll("article").length >= 60, null, { timeout: 90_000 });
await page.evaluate(() => document.fonts?.ready);

const cards = page.locator("article");
const manifest = [];
for (let index = 0; index < await cards.count(); index += 1) {
  const card = cards.nth(index);
  const id = (await card.locator("h3").innerText()).trim();
  const slug = id.replace(/([a-z0-9])([A-Z])/g, "$1-$2").replace(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
  const terminal = card.locator(":scope > div").last();
  const filename = `${slug}.png`;
  await terminal.screenshot({ path: path.join(output, filename) });
  manifest.push({ id, slug, image: `/img/themes/variants/${filename}` });
}

// The review adds accessibility and Projectious variants after the original
// 61-card board was authored. Render those from the same audited TOML so the
// documentation still has one specimen for every shipped concrete theme.
const audited = parseToml(await readFile(path.join(root, "cli/assets/aibox-theme-corrections.toml"), "utf8"));
const captured = new Set(manifest.map(({ id }) => id));
for (const [id, spec] of Object.entries(audited.themes)) {
  if (captured.has(id)) continue;
  const chrome = spec.chrome ?? {};
  const slug = id.replace(/([a-z0-9])([A-Z])/g, "$1-$2").replace(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
  const surface = chrome.surface ?? spec.bg;
  const magenta = spec.magenta ?? chrome.magenta ?? spec.accent;
  const accentText = chrome.accent_text ?? spec.accent;
  const accentFill = chrome.accent_fill ?? spec.accent;
  const tabInk = chrome.status_active_ink ?? chrome.cursor_text ?? spec.bg;
  const selBg = chrome.selection_bg ?? surface;
  const selFg = chrome.selection_fg ?? spec.fg;
  await page.setContent(`<!doctype html><style>*{box-sizing:border-box}body{margin:0;padding:24px;background:#132440}.terminal{width:840px;border:1px solid ${chrome.border_inactive ?? spec.muted};border-radius:7px;overflow:hidden;font:16px/1.75 "IBM Plex Mono",ui-monospace,monospace}.bar{display:flex;gap:14px;align-items:center;padding:7px 14px;background:${surface};color:${spec.muted}}.tab{padding:1px 10px;border-radius:3px;background:${accentFill};color:${tabInk};font-weight:700}.label{margin-left:auto}.body{padding:16px 18px;background:${spec.bg};color:${spec.fg}}.accent{color:${accentText}}.green{color:${spec.green}}.muted{color:${spec.muted};font-style:italic}.kw{color:${magenta};font-weight:700}.op{color:${spec.cyan};font-style:italic}.ty{color:${spec.yellow};font-weight:700}.err{color:${spec.red};font-weight:700}.num{color:${spec.orange}}.sel{background:${selBg};color:${selFg}}</style><div id="terminal" class="terminal"><div class="bar"><span class="tab">1 aibox</span><span>2 logs</span><span class="label">${spec.family} · ${spec.mode} · ${spec.variant}</span></div><div class="body"><div><span class="accent">~/aibox</span> <span class="green">✓</span> cargo test <span class="muted"># audited variant</span></div><div><span class="kw">async fn</span> <span class="op">resolve</span>(n: <span class="ty">Palette</span>) {</div><div><span class="err">error</span>: floor missed <span class="num">4.5</span> <span class="sel">selected range</span></div></div></div>`);
  const filename = `${slug}.png`;
  await page.locator("#terminal").screenshot({ path: path.join(output, filename) });
  manifest.push({ id, slug, image: `/img/themes/variants/${filename}` });
}

for (const item of manifest) {
  const spec = audited.themes[item.id];
  item.family = spec?.family ?? "legacy";
  item.mode = spec?.mode ?? "dark";
  item.variant = spec?.variant ?? "default";
}
const expectedVariants = Object.keys(audited.themes).length;
if (manifest.length !== expectedVariants || new Set(manifest.map(({ id }) => id)).size !== expectedVariants) {
  throw new Error(`theme specimen coverage mismatch: expected ${expectedVariants}, captured ${manifest.length}`);
}
manifest.sort((a, b) => a.family.localeCompare(b.family) || a.mode.localeCompare(b.mode) || a.id.localeCompare(b.id));
await writeFile(path.join(output, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
await browser.close();
console.log(`Captured ${manifest.length} terminal variants in ${output}`);
