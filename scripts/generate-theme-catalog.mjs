#!/usr/bin/env node

import { createRequire } from "node:module";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const require = createRequire(path.join(root, "docs-site/package.json"));
const { parse: parseToml } = require("smol-toml");
const source = path.join(root, "cli/assets/aibox-theme-corrections.toml");
const output = path.join(root, "docs-site/data/theme_catalog.json");
const audited = parseToml(await readFile(source, "utf8"));

const paletteSlots = ["bg", "fg", "accent", "green", "red", "yellow", "orange", "cyan", "muted", "magenta"];
const chromeSlots = [
  "surface", "selection_bg", "selection_fg", "cursor", "cursor_text",
  "border_active", "border_inactive", "pane_inactive_bg", "pane_inactive_fg",
  "diff_add_bg", "diff_change_bg", "diff_del_bg", "status_active_ink",
  "accent_fill", "accent_text", "code_panel", "terminal",
];

const toolUses = {
  vim: "Generated UI and syntax colors, selection, search, diff, and diagnostics",
  tmux: "Status surfaces, active and inactive windows, pane borders, and state emphasis",
  yazi: "Manager chrome, modes, selection, file types, and Git states",
  starship: "Prompt segments, repository state, duration, and language modules",
  bat: "Generated TextMate syntax colors and emphasis",
  delta: "Generated bat syntax theme with audited diff colors",
  fzf: "Prompt, match, selection, border, and disabled-state colors",
  eza: "File types, metadata, permissions, and Git states",
  less: "Headings, options, search matches, and status treatment",
  lnav: "Text, selection, status, warning, and error roles",
  lazygit: "Borders, selected rows, search, and repository state",
  claude_code: "Terminal ANSI palette; native application theme remains harness-owned",
  gemini: "Terminal palette plus the closest supported native Gemini theme",
  opencode: "Generated native application palette",
  aider: "Terminal mode plus the closest generated Pygments mapping",
};

const tools = Object.entries(audited.emphasis.tools).map(([id, spec]) => ({
  id,
  name: id.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase()),
  supports: spec.supports,
  requires: spec.requires ?? "",
  note: spec.note ?? "",
  uses: toolUses[id] ?? "Uses the shared terminal palette",
}));

function rgb(hex) {
  const value = Number.parseInt(hex.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function relativeLuminance(hex) {
  const channels = rgb(hex).map((value) => {
    const channel = value / 255;
    return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrastRatio(foreground, background) {
  const values = [relativeLuminance(foreground), relativeLuminance(background)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}

function readableInk(fill, background, foreground) {
  return [background, foreground, "#FFFFFF", "#000000"].reduce((best, candidate) =>
    contrastRatio(candidate, fill) > contrastRatio(best, fill) ? candidate : best
  );
}

function drivingColor(hex) {
  const [red, green, blue] = rgb(hex).map((value) => value / 255);
  const max = Math.max(red, green, blue);
  const min = Math.min(red, green, blue);
  const delta = max - min;
  if (delta < 0.12) return "neutral";
  let hue = 0;
  if (max === red) hue = 60 * (((green - blue) / delta) % 6);
  else if (max === green) hue = 60 * ((blue - red) / delta + 2);
  else hue = 60 * ((red - green) / delta + 4);
  if (hue < 0) hue += 360;
  if (hue < 15 || hue >= 345) return "red";
  if (hue < 45) return "orange";
  if (hue < 75) return "yellow";
  if (hue < 165) return "green";
  if (hue < 195) return "cyan";
  if (hue < 255) return "blue";
  if (hue < 290) return "purple";
  return "magenta";
}

const familyAppearance = {
  andromeeda: "cyan", "aurora-x": "blue", ayu: "blue", catppuccin: "purple",
  borland: "blue",
  contrast: "blue", "contrast-mono": "neutral", dracula: "purple", everforest: "green",
  github: "blue", gruvbox: "orange", houston: "orange", kanagawa: "blue",
  laserwave: "purple", material: "blue", min: "blue", mono: "neutral",
  monokai: "green", moonlight: "blue", "night-owl": "blue", nord: "blue",
  norton: "blue",
  "one-dark": "blue", plastic: "blue", poimandres: "green", red: "red",
  phosphor: "green",
  "rose-pine": "purple", slack: "purple", snazzy: "purple", solarized: "blue",
  "synthwave-84": "purple", "tokyo-night": "blue", vesper: "orange",
  vitesse: "green", vscode: "blue",
};

function generalAppearance(id, spec) {
  if (spec.family === "projectious") {
    return id === "ProjectiousNavy" || id === "ProjectiousDeep" ? "blue" : "red";
  }
  return familyAppearance[spec.family] ?? drivingColor(spec.bg) ?? drivingColor(spec.accent);
}

const themes = Object.entries(audited.themes).map(([id, spec]) => {
  const ratio = contrastRatio(spec.fg, spec.bg);
  const resolvedPalette = { ...spec, magenta: spec.magenta ?? spec.chrome?.magenta };
  const tabBackground = spec.chrome?.accent_fill ?? spec.accent;
  const tabForeground = spec.chrome?.status_active_ink
    ?? (spec.family === "projectious" && spec.mode === "dark"
      ? spec.chrome?.cursor_text
      : readableInk(tabBackground, spec.bg, spec.fg));
  const render = {
    background: spec.bg,
    foreground: spec.fg,
    surface: spec.chrome?.surface,
    tab_background: tabBackground,
    tab_foreground: tabForeground,
    path: spec.accent,
    success: spec.green,
    muted: spec.muted,
    keyword: resolvedPalette.magenta,
    function: spec.cyan,
    type: spec.yellow,
    error: spec.red,
    number: spec.orange,
    selection_background: spec.chrome?.selection_bg,
    selection_foreground: spec.chrome?.selection_fg,
    border: spec.chrome?.border_inactive ?? spec.muted,
  };
  return {
    id,
    slug: id.replace(/([a-z0-9])([A-Z])/g, "$1-$2").replace(/[^a-zA-Z0-9]+/g, "-").toLowerCase(),
    family: spec.family,
    mode: spec.mode,
    variant: spec.variant,
    config_variant: spec.variant === "solo" || spec.variant === "high" || spec.variant.startsWith("default") ? "" : spec.variant,
    emphasis_min: spec.emphasis_min ?? "none",
    suspect: spec.suspect ?? "",
    general_appearance: generalAppearance(id, spec),
    contrast_ratio: Number(ratio.toFixed(2)),
    contrast: spec.variant === "max" ? "maximum" : ratio >= 10 ? "high" : "soft",
    tools: tools.map((tool) => tool.id),
    render,
    palette: paletteSlots.filter((slot) => resolvedPalette[slot]).map((slot) => ({ slot, value: resolvedPalette[slot] })),
    chrome: chromeSlots.filter((slot) => spec.chrome?.[slot]).map((slot) => ({ slot, value: spec.chrome[slot] })),
    attributes: Object.entries(spec.attributes ?? {}).map(([slot, value]) => ({ slot, value })),
  };
});

themes.sort((a, b) => a.family.localeCompare(b.family) || a.mode.localeCompare(b.mode) || a.id.localeCompare(b.id));

if (themes.length !== audited.meta.variants) {
  throw new Error(`theme coverage mismatch: expected ${audited.meta.variants}, found ${themes.length}`);
}

const hexPattern = /^#[0-9a-f]{6}$/i;
for (const theme of themes) {
  for (const [role, value] of Object.entries(theme.render)) {
    if (!hexPattern.test(value ?? "")) throw new Error(`${theme.id}: missing or invalid rendered ${role}: ${value}`);
  }
  const pairs = [
    ["terminal foreground", theme.render.foreground, theme.render.background, 7],
    ["active tab", theme.render.tab_foreground, theme.render.tab_background, 4.5],
    ["selection", theme.render.selection_foreground, theme.render.selection_background, 4.5],
  ];
  for (const [role, foreground, background, floor] of pairs) {
    const measured = contrastRatio(foreground, background);
    if (measured < floor) throw new Error(`${theme.id}: ${role} ${measured.toFixed(2)}:1 misses ${floor}:1`);
  }
}

await mkdir(path.dirname(output), { recursive: true });
const filters = {
  modes: [...new Set(themes.map((theme) => theme.mode))].sort(),
  appearances: [...new Set(themes.map((theme) => theme.general_appearance))].sort(),
  contrasts: ["soft", "high", "maximum"].filter((value) => themes.some((theme) => theme.contrast === value)),
};
await writeFile(output, `${JSON.stringify({ meta: audited.meta, roles: audited.emphasis.roles, tools, filters, themes }, null, 2)}\n`);
console.log(`Generated ${themes.length} themes in ${output}`);
