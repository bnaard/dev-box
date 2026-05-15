#!/usr/bin/env bash
# test-screencasts.sh — visual smoke tests using asciinema recordings
#
# Records fast (2s) documentation-demo casts to a temp directory and validates
# they contain real terminal output. Does NOT overwrite docs recordings.
# Generated runtime layout coverage lives in cli/tests/e2e/visual_matrix.rs and
# scripts/release-runtime-smoke.sh.
#
# Usage:
#   ./scripts/test-screencasts.sh              # run all tests
#   ./scripts/test-screencasts.sh layouts      # only layout tests
#   ./scripts/test-screencasts.sh themes       # only theme tests
#   ./scripts/test-screencasts.sh tools        # only tool smoke tests

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
TEST_DIR=$(mktemp -d /tmp/devbox-test-casts-XXXX)
FAILURES=0
PASSES=0
SKIPS=0

# Output directory for docs-site recordings (persists across test runs)
DOCS_CAST_DIR="${PROJECT_ROOT}/docs-site/static/asciinema/themes"

# Full list of all 61 theme slug → palette tuples.
# Format: "theme-slug:bg:fg:accent:green:orange:cyan:muted"
# Derived from cli/src/themes.rs theme_palette() and yazi_surface_color().
# bg, fg, accent, green, orange, cyan, muted correspond to palette positions.
# surface (statusbar-bg) is also encoded as the 8th field.
# Field order: slug:bg:fg:accent:green:orange:cyan:muted:surface
THEME_PALETTE_TABLE=(
  "gruvbox-dark:#282828:#D5C4A1:#D79921:#98971A:#D65D0E:#689D6A:#928374:#3C3836"
  "gruvbox-light:#FBF1C7:#3C3836:#D65D0E:#79740E:#D65D0E:#076678:#928374:#EBDBB2"
  "catppuccin-mocha:#1E1E2E:#CDD6F4:#89B4FA:#A6E3A1:#FAB387:#94E2D5:#6C7086:#313244"
  "catppuccin-macchiato:#24273A:#CAD3F5:#8AADF4:#A6DA95:#F5A97F:#8BD5CA:#6E738D:#313244"
  "catppuccin-frappe:#303446:#C6D0F5:#8CAAEE:#A6D189:#EF9F76:#81C8BE:#737994:#313244"
  "catppuccin-latte:#EFF1F5:#4C4F69:#1E66F5:#40A02B:#FE640B:#179299:#9CA0B0:#CCD0DA"
  "dracula:#282A36:#F8F8F2:#BD93F9:#50FA7B:#FFB86C:#8BE9FD:#6272A4:#44475A"
  "dracula-soft:#22212C:#F8F8F2:#C8A8F9:#62E884:#FFCA80:#A1F0FE:#7970A9:#44475A"
  "tokyo-night:#1A1B26:#C0CAF5:#7AA2F7:#9ECE6A:#FF9E64:#7DCFFF:#565F89:#283457"
  "tokyo-night-storm:#24283B:#C0CAF5:#7AA2F7:#9ECE6A:#FF9E64:#7DCFFF:#565F89:#283457"
  "tokyo-night-day:#E1E2E7:#3760BF:#2E7DE9:#587539:#B15C00:#007197:#7B8496:#CCD0DA"
  "nord:#2E3440:#D8DEE9:#88C0D0:#A3BE8C:#D08770:#81A1C1:#4C566A:#3B4252"
  "rose-pine:#191724:#E0DEF4:#C4A7E7:#31748F:#EA9A97:#9CCFD8:#6E6A86:#313244"
  "rose-pine-moon:#232136:#E0DEF4:#C4A7E7:#3E8FB0:#EA9A97:#9CCFD8:#6E6A86:#313244"
  "rose-pine-dawn:#FAF4ED:#575279:#907AA9:#56949F:#D7827E:#286983:#9893A5:#CCD0DA"
  "material:#263238:#EEFFFF:#82AAFF:#C3E88D:#F78C6C:#89DDFF:#546E7A:#313244"
  "material-ocean:#0F111A:#A6ACCD:#82AAFF:#C3E88D:#F78C6C:#89DDFF:#464B5D:#313244"
  "material-palenight:#292D3E:#A6ACCD:#82AAFF:#C3E88D:#F78C6C:#89DDFF:#676E95:#313244"
  "material-lighter:#FAFAFA:#546E7A:#6182B8:#91B859:#F76D47:#39ADB5:#90A4AE:#CCD0DA"
  "material-darker:#212121:#EEFFFF:#89DDFF:#C3E88D:#F78C6C:#82AAFF:#546E7A:#313244"
  "solarized-dark:#002B36:#93A1A1:#268BD2:#859900:#CB4B16:#2AA198:#657B83:#313244"
  "solarized-light:#FDF6E3:#586E75:#268BD2:#859900:#CB4B16:#2AA198:#93A1A1:#CCD0DA"
  "github-dark:#0D1117:#C9D1D9:#58A6FF:#3FB950:#DB6D28:#79C0FF:#8B949E:#313244"
  "github-light:#FFFFFF:#24292F:#0969DA:#1A7F37:#BC4C00:#218BFF:#6E7781:#CCD0DA"
  "github-dark-dimmed:#22272E:#ADBAC7:#539BF5:#57AB5A:#F47067:#6CB6FF:#768390:#313244"
  "github-dark-high-contrast:#0A0C10:#F0F3F6:#71B7FF:#26CD4D:#FF6A69:#91CBFF:#9198A1:#313244"
  "github-light-high-contrast:#FFFFFF:#0E1116:#1A69DB:#104F24:#A0111F:#034188:#69717B:#CCD0DA"
  "ayu-dark:#0A0E14:#B3B1AD:#39BAE6:#AAD94C:#FF8F40:#95E6CB:#626A73:#313244"
  "ayu-mirage:#1F2430:#CCCAC2:#5CCFE6:#BAE67E:#FFAD66:#95E6CB:#707A8C:#313244"
  "ayu-light:#FAFAFA:#5C6773:#55B4D4:#86B300:#F07171:#4CBF99:#ABB0B6:#CCD0DA"
  "night-owl:#011627:#D6DEEB:#82AAFF:#22DA6E:#F78C6C:#21C7A8:#637777:#313244"
  "night-owl-light:#FBFBFB:#403F53:#4876D6:#2AA298:#DD6A58:#08916A:#989FB1:#CCD0DA"
  "moonlight:#212337:#C8D3F5:#82AAFF:#C3E88D:#F78C6C:#86E1FC:#7A88CF:#283457"
  "projectious:#0E1720:#C5DAF0:#E05232:#4FB07A:#F2A65A:#8AACC8:#7B8DA3:#131E2B"
  "andromeeda:#23262E:#D5CED9:#00E8C6:#89E044:#F39C12:#00E8C6:#6B6B6B:#313244"
  "aurora-x:#07090F:#D4D4D4:#569CD6:#B5CEA8:#CE9178:#4EC9B0:#5C6370:#313244"
  "everforest-dark:#2D353B:#D3C6AA:#7FBBB3:#A7C080:#D699B6:#83C092:#7A8478:#313244"
  "everforest-light:#FDF6E3:#5C6A72:#3A94C5:#8DA101:#DF69BA:#35A77C:#939F91:#CCD0DA"
  "houston:#17191E:#CDD6F4:#F9C86A:#4AF2C8:#81D4FA:#4AF2C8:#545878:#313244"
  "kanagawa-wave:#1F1F28:#DCD7BA:#7E9CD8:#98BB6C:#D27E99:#7AA89F:#727169:#313244"
  "kanagawa-dragon:#181616:#C5C9C5:#7EB3C9:#87A987:#C4746E:#8EA4A2:#8A8980:#313244"
  "kanagawa-lotus:#F2ECBC:#545464:#1F5F8A:#4E7C3F:#B5485D:#536A5B:#A09F8F:#CCD0DA"
  "laserwave:#27212E:#FFFFFF:#EB64B9:#74DFC4:#FFEE79:#74DFC4:#6B5F7D:#313244"
  "min-dark:#1F1F1F:#B2B2B2:#569CD6:#B5CEA8:#CE9178:#4EC9B0:#525252:#313244"
  "min-light:#F8F8F8:#333333:#0000FF:#098658:#C1440E:#267F99:#9A9A9A:#CCD0DA"
  "monokai:#272822:#F8F8F2:#F92672:#A6E22E:#AE81FF:#66D9EF:#75715E:#313244"
  "one-dark-pro:#282C34:#ABB2BF:#61AFEF:#98C379:#D19A66:#56B6C2:#5C6370:#313244"
  "one-light:#FAFAFA:#383A42:#4078F2:#50A14F:#986801:#0184BC:#A0A1A7:#CCD0DA"
  "plastic:#1B1D23:#ABB2BF:#61AFEF:#98C379:#D19A66:#56B6C2:#7A7E8A:#313244"
  "poimandres:#1B1E28:#A6ACCD:#A6DA95:#5DE4C7:#D0679D:#ADD7FF:#767C9D:#313244"
  "red:#390000:#F8F8F8:#FF6666:#F4C2C2:#FFD0D0:#FF9999:#A06060:#313244"
  "slack-dark:#222529:#D1D2D3:#8CC4FF:#AFE3A4:#DFC55A:#98D1E0:#60656A:#313244"
  "slack-ochin:#F9F9F9:#383A3C:#0070D1:#268829:#C64B10:#007A7A:#A0A4A8:#CCD0DA"
  "snazzy-light:#FAFBFC:#2D2D2D:#57C7FF:#5AF78E:#FF6AC1:#57C7FF:#9E9E9E:#CCD0DA"
  "synthwave-84:#2A2139:#FFFFFF:#36F9F6:#FF7EDB:#F97E72:#36F9F6:#848082:#313244"
  "vesper:#101010:#FFFFFF:#FF7B00:#99FFE4:#FFC799:#FFC799:#5C5C5C:#313244"
  "vitesse-dark:#121212:#DBD7CA:#4D9375:#C98A7D:#6496C8:#80A0C0:#758575:#313244"
  "vitesse-light:#FFFFFF:#393A34:#1E754F:#B56959:#296AA3:#2E808F:#A0A077:#CCD0DA"
  "vitesse-black:#000000:#DBD7CA:#4D9375:#C98A7D:#6496C8:#80A0C0:#606060:#313244"
  "vscode-dark-plus:#1E1E1E:#D4D4D4:#569CD6:#B5CEA8:#CE9178:#4EC9B0:#6A9955:#313244"
  "vscode-light-plus:#FFFFFF:#000000:#0000FF:#098658:#A31515:#267F99:#008000:#CCD0DA"
)

LAYOUTS=(dev focus cowork)

info()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
pass()  { printf '\033[1;32m ✓\033[0m  %s\n' "$1"; PASSES=$((PASSES + 1)); }
fail()  { printf '\033[1;31m ✗\033[0m  %s\n' "$1"; FAILURES=$((FAILURES + 1)); }
skip()  { printf '\033[1;33m ○\033[0m  %s (skipped)\n' "$1"; SKIPS=$((SKIPS + 1)); }

cleanup_tmux() {
  tmux kill-session -t aibox-screencast 2>/dev/null || true
  sleep 0.5
}

cleanup_demo_tmux() {
  local slug="$1"
  tmux -L aibox-recordings kill-session -t "demo-${slug}" 2>/dev/null || true
  sleep 0.3
}

# ─── Validation helpers ───────────────────────────────────────────────────────

validate_cast() {
  local cast="$1"
  local label="$2"
  local min_events="${3:-10}"
  local min_size="${4:-5000}"

  if [ ! -f "${cast}" ]; then
    fail "${label}: cast file not created"
    return 1
  fi

  local size
  size=$(stat -c%s "${cast}" 2>/dev/null || echo 0)
  if [ "${size}" -lt "${min_size}" ]; then
    fail "${label}: too small (${size} bytes, need >${min_size})"
    return 1
  fi

  local events
  events=$(wc -l < "${cast}")
  if [ "${events}" -lt "${min_events}" ]; then
    fail "${label}: too few events (${events}, need >${min_events})"
    return 1
  fi

  # Verify header is valid JSON with correct version
  local version
  version=$(head -1 "${cast}" | python3 -c "import sys,json; print(json.load(sys.stdin)['version'])" 2>/dev/null || echo "")
  if [ "${version}" != "2" ]; then
    fail "${label}: invalid header (version=${version})"
    return 1
  fi

  pass "${label} (${events} events, $(numfmt --to=iec ${size}))"
  return 0
}

assert_cast_visible_status_text() {
  local cast="$1"
  local label="$2"

  if python3 - "${cast}" << 'PYEOF'
import json
import re
import sys

cast_path = sys.argv[1]
raw = []
with open(cast_path, encoding="utf-8", errors="ignore") as fh:
    next(fh, None)
    for line in fh:
        try:
            event = json.loads(line)
        except Exception:
            continue
        if len(event) >= 3 and event[1] == "o" and isinstance(event[2], str):
            raw.append(event[2])

text = "".join(raw)
text = re.sub(r"\x1b\][^\x07]*(?:\x07|\x1b\\)", "", text)
text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)
text = re.sub(r"\x1b[()][A-Za-z0-9]", "", text)
text = re.sub(r"[\x00-\x08\x0b-\x1f\x7f]", " ", text)
text = re.sub(r"\s+", " ", text)

tokens = ["Ctrl-g", "Prefix", "pane", "window", "tmux", "dev", "shell", "AI"]
if not any(token in text for token in tokens):
    print(text[:1200])
    sys.exit(1)
PYEOF
  then
    pass "${label}: visible status/keybar text"
  else
    fail "${label}: missing visible status/keybar text"
  fi
}

# ─── ANSI status-bar invariant checker (Python) ───────────────────────────────
#
# Checks four invariants on the status-bar rows of a cast:
#   I1: status-row bg ⊆ palette (no foreign colors inside content)
#   I2: powerline separator continuity (separator bg == next-cell bg)
#   I3: no default/black bg inside status-row content span
#   I4: pane area bg ≈ theme bg (>50% of blank cells match palette bg)
#
# Usage: python3 -c "..." <cast_path> <theme_slug> <bg> <fg> <accent> \
#                         <green> <orange> <cyan> <muted> <surface>
# Writes failures to stdout; exit 0 = all pass, exit 1 = failures found.
#
# Implementation note: we embed the helper as a heredoc so it can be sourced
# from test_themes without creating a sibling file on disk.

CAST_INVARIANTS_PY='
import sys
import json
import re

# ---------------------------------------------------------------------------
# Parse CLI args: cast_path theme slug bg fg accent green orange cyan muted surface
# ---------------------------------------------------------------------------
if len(sys.argv) < 11:
    print("usage: cast_invariants.py cast theme bg fg accent green orange cyan muted surface")
    sys.exit(2)

cast_path  = sys.argv[1]
theme_slug = sys.argv[2]
bg_hex     = sys.argv[3].upper()
# fg, accent, green, orange, cyan, muted, surface
palette_set = {s.upper() for s in sys.argv[3:11]}  # includes bg
surface_hex = sys.argv[10].upper()

# Powerline/NerdFont separator glyphs (common set used by tmux-powerline plugins)
SEPARATORS = set("")

COLS = 160
ROWS = 45

# ---------------------------------------------------------------------------
# Minimal ANSI virtual terminal emulator
# ---------------------------------------------------------------------------
class Cell:
    __slots__ = ("ch", "bg", "fg")
    def __init__(self):
        self.ch  = " "
        self.bg  = "default"
        self.fg  = "default"

    def copy_from(self, other):
        self.ch  = other.ch
        self.bg  = other.bg
        self.fg  = other.fg


class Screen:
    def __init__(self, cols=COLS, rows=ROWS):
        self.cols = cols
        self.rows = rows
        self.grid = [[Cell() for _ in range(cols)] for _ in range(rows)]
        self.row  = 0
        self.col  = 0
        # SGR state
        self.cur_bg = "default"
        self.cur_fg = "default"
        self.cur_bold    = False
        self.cur_reverse = False

    def _cell(self, r, c):
        if 0 <= r < self.rows and 0 <= c < self.cols:
            return self.grid[r][c]
        return None

    def _put(self, ch):
        cell = self._cell(self.row, self.col)
        if cell:
            cell.ch = ch
            if self.cur_reverse:
                cell.bg = self.cur_fg
                cell.fg = self.cur_bg
            else:
                cell.bg = self.cur_bg
                cell.fg = self.cur_fg
        self.col += 1
        if self.col >= self.cols:
            self.col = 0
            self.row = min(self.row + 1, self.rows - 1)

    def _erase_line(self, mode):
        if mode == 0:  # to EOL
            for c in range(self.col, self.cols):
                cell = self._cell(self.row, c)
                if cell:
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg
        elif mode == 1:  # to BOL
            for c in range(0, self.col + 1):
                cell = self._cell(self.row, c)
                if cell:
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg
        elif mode == 2:  # whole line
            for c in range(self.cols):
                cell = self._cell(self.row, c)
                if cell:
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg

    def _erase_display(self, mode):
        if mode == 2:
            for r in range(self.rows):
                for c in range(self.cols):
                    cell = self.grid[r][c]
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg
        elif mode == 0:
            for c in range(self.col, self.cols):
                cell = self._cell(self.row, c)
                if cell:
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg
            for r in range(self.row + 1, self.rows):
                for c in range(self.cols):
                    cell = self.grid[r][c]
                    cell.ch = " "; cell.bg = self.cur_bg; cell.fg = self.cur_fg

    def _apply_sgr(self, params):
        i = 0
        while i < len(params):
            p = params[i]
            if p == 0:
                self.cur_bg = "default"; self.cur_fg = "default"
                self.cur_bold = False; self.cur_reverse = False
            elif p == 1:
                self.cur_bold = True
            elif p == 7:
                self.cur_reverse = True
            elif p == 22:
                self.cur_bold = False
            elif p == 27:
                self.cur_reverse = False
            elif p == 39:
                self.cur_fg = "default"
            elif p == 49:
                self.cur_bg = "default"
            elif 30 <= p <= 37:
                self.cur_fg = f"indexed:{p - 30}"
            elif 40 <= p <= 47:
                self.cur_bg = f"indexed:{p - 40}"
            elif 90 <= p <= 97:
                self.cur_fg = f"indexed:{p - 90 + 8}"
            elif 100 <= p <= 107:
                self.cur_bg = f"indexed:{p - 100 + 8}"
            elif p == 38:
                if i + 1 < len(params) and params[i+1] == 2 and i + 4 < len(params):
                    r, g, b = params[i+2], params[i+3], params[i+4]
                    self.cur_fg = f"#{r:02X}{g:02X}{b:02X}"
                    i += 4
                elif i + 1 < len(params) and params[i+1] == 5 and i + 2 < len(params):
                    self.cur_fg = f"indexed:{params[i+2]}"
                    i += 2
            elif p == 48:
                if i + 1 < len(params) and params[i+1] == 2 and i + 4 < len(params):
                    r, g, b = params[i+2], params[i+3], params[i+4]
                    self.cur_bg = f"#{r:02X}{g:02X}{b:02X}"
                    i += 4
                elif i + 1 < len(params) and params[i+1] == 5 and i + 2 < len(params):
                    self.cur_bg = f"indexed:{params[i+2]}"
                    i += 2
            i += 1

    def feed(self, text):
        i = 0
        while i < len(text):
            ch = text[i]

            if ch == "\x1b":
                # ESC sequence
                i += 1
                if i >= len(text):
                    break
                nxt = text[i]

                if nxt == "[":
                    # CSI sequence
                    i += 1
                    start = i
                    # Read param bytes (0x30-0x3F) and intermediate bytes (0x20-0x2F)
                    while i < len(text) and (0x20 <= ord(text[i]) <= 0x3F):
                        i += 1
                    if i < len(text):
                        final = text[i]
                        param_str = text[start:i]
                        i += 1
                        # Parse numeric params
                        def parse_params(s):
                            parts = re.split(r"[;:]", s)
                            result = []
                            for pt in parts:
                                try:
                                    result.append(int(pt))
                                except ValueError:
                                    result.append(0)
                            return result

                        if final == "m":  # SGR
                            raw = parse_params(param_str)
                            if not raw:
                                raw = [0]
                            self._apply_sgr(raw)
                        elif final == "H" or final == "f":  # cursor position
                            parts = parse_params(param_str) if param_str else []
                            r = (parts[0] - 1) if len(parts) >= 1 and parts[0] > 0 else 0
                            c = (parts[1] - 1) if len(parts) >= 2 and parts[1] > 0 else 0
                            self.row = max(0, min(r, self.rows - 1))
                            self.col = max(0, min(c, self.cols - 1))
                        elif final == "A":  # cursor up
                            n = parse_params(param_str)[0] if param_str else 1
                            self.row = max(0, self.row - (n or 1))
                        elif final == "B":  # cursor down
                            n = parse_params(param_str)[0] if param_str else 1
                            self.row = min(self.rows - 1, self.row + (n or 1))
                        elif final == "C":  # cursor right
                            n = parse_params(param_str)[0] if param_str else 1
                            self.col = min(self.cols - 1, self.col + (n or 1))
                        elif final == "D":  # cursor left
                            n = parse_params(param_str)[0] if param_str else 1
                            self.col = max(0, self.col - (n or 1))
                        elif final == "G":  # cursor column (absolute)
                            n = parse_params(param_str)[0] if param_str else 1
                            self.col = max(0, min((n or 1) - 1, self.cols - 1))
                        elif final == "d":  # cursor row (absolute)
                            n = parse_params(param_str)[0] if param_str else 1
                            self.row = max(0, min((n or 1) - 1, self.rows - 1))
                        elif final == "J":  # erase display
                            n = parse_params(param_str)[0] if param_str else 0
                            self._erase_display(n)
                        elif final == "K":  # erase line
                            n = parse_params(param_str)[0] if param_str else 0
                            self._erase_line(n)
                        # else: ignore unknown CSI (cursor show/hide, mode set, etc.)

                elif nxt == "]":
                    # OSC — consume until ST (BEL or ESC \)
                    while i < len(text):
                        c2 = text[i]
                        if c2 == "\x07":
                            i += 1
                            break
                        elif c2 == "\x1b" and i + 1 < len(text) and text[i+1] == "\\":
                            i += 2
                            break
                        i += 1

                elif nxt in "()":
                    # Designate character set — skip one byte
                    i += 1

                else:
                    # Two-char ESC sequence (ESC M, ESC =, ESC >, etc.) — ignore
                    i += 1

                continue

            elif ch == "\r":
                self.col = 0
            elif ch == "\n":
                self.row = min(self.row + 1, self.rows - 1)
            elif ch == "\t":
                self.col = (self.col + 8) & ~7
                self.col = min(self.col, self.cols - 1)
            elif ch == "\a":  # BEL
                pass
            elif ch == "\b":  # backspace
                self.col = max(0, self.col - 1)
            elif ord(ch) < 0x20:
                pass  # other C0 controls — ignore
            else:
                self._put(ch)

            i += 1


# ---------------------------------------------------------------------------
# Replay cast into screen
# ---------------------------------------------------------------------------
def replay_cast(path):
    """Replay a cast file, stopping before the terminal-exit clear sequence.

    tmux attach-session emits a full-screen erase (ESC[H ESC[2J) when the
    session is killed.  If we replay that event, the final screen state is
    empty and status-bar invariants all pass trivially.  We therefore stop
    replay just before any event that triggers a full-screen erase at the
    top-left corner, which is the signature of the detach/exit sequence.
    """
    screen = Screen()
    _FULL_CLEAR_RE = re.compile(r"\x1b\[(?:1;1|1;|;1|)H\x1b\[2J|\x1b\[H\x1b\[2J")
    try:
        with open(path, encoding="utf-8", errors="ignore") as fh:
            header_line = fh.readline()
            try:
                hdr = json.loads(header_line)
                w = hdr.get("width", COLS)
                h = hdr.get("height", ROWS)
                screen = Screen(min(w, COLS), min(h, ROWS))
            except Exception:
                pass
            events = []
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    ev = json.loads(line)
                except Exception:
                    continue
                if len(ev) >= 3 and ev[1] == "o" and isinstance(ev[2], str):
                    events.append(ev[2])
            # Replay all events except the terminal-exit full-screen clear.
            # Walk from the end; stop discarding once we hit an event without
            # the clear signature.
            trim = 0
            for payload in reversed(events):
                if _FULL_CLEAR_RE.search(payload):
                    trim += 1
                else:
                    break
            for payload in events[: len(events) - trim]:
                screen.feed(payload)
    except OSError as exc:
        print(f"ERROR: cannot open cast: {exc}")
        sys.exit(1)
    return screen


# ---------------------------------------------------------------------------
# Invariant checkers
# ---------------------------------------------------------------------------
def norm_hex(h):
    """Normalise a #RRGGBB string to uppercase."""
    if isinstance(h, str) and h.startswith("#") and len(h) == 7:
        return h.upper()
    return h  # "default", "indexed:N", etc.


def status_row_invariants(screen, palette, bg_hex, surface_hex):
    failures = []
    rows = screen.rows
    cols = screen.cols

    # Determine where the status bar actually is.
    # tmux places the status bar at the BOTTOM by default (1 row).
    # powerkit uses `set -g status 2` which creates 2 rows at the TOP.
    # We detect the top vs bottom placement by checking which candidate
    # rows have non-default background cells in the content span.
    #
    # Candidate sets:
    #   top    = rows 0 and 1  (powerkit double-bar)
    #   bottom = rows (rows-2) and (rows-1)  (standard tmux single/double bar)
    candidate_top    = [0, 1] if rows >= 2 else [0]
    candidate_bottom = [rows - 2, rows - 1] if rows >= 2 else [rows - 1]

    def _has_content(row_cells):
        """Return True if this row has at least one non-default bg cell."""
        return any(c.bg != "default" for c in row_cells)

    top_has_content    = any(_has_content(screen.grid[r]) for r in candidate_top)
    bottom_has_content = any(_has_content(screen.grid[r]) for r in candidate_bottom)

    if top_has_content:
        # powerkit double-bar at top
        status_rows = candidate_top
        pane_start_row = len(candidate_top)
    elif bottom_has_content:
        # standard tmux bar at bottom
        status_rows = candidate_bottom
        pane_start_row = 0
    else:
        # No status bar content found — check bottom rows by convention
        status_rows = candidate_bottom
        pane_start_row = 0
        failures.append("I0 no status bar content detected (all cells have default bg)")

    # --- I1 + I2 + I3: iterate status rows ---
    for sr in status_rows:
        row_cells = screen.grid[sr]

        # Find content span: skip leading/trailing all-space default-bg cells
        left = 0
        right = cols - 1
        while left < cols and row_cells[left].ch == " " and row_cells[left].bg == "default":
            left += 1
        while right > left and row_cells[right].ch == " " and row_cells[right].bg == "default":
            right -= 1
        # Content range: left..right inclusive
        content_cells = row_cells[left:right + 1]

        for idx, cell in enumerate(content_cells):
            bg = norm_hex(cell.bg)

            # I1: bg must be "close" to a palette colour.
            # powerkit generates derived intermediate colours (dimmed/brightened
            # variants) that are not identical to the 8 canonical palette entries.
            # We allow a per-channel delta of up to 50 (out of 255) to accept
            # those derived colours as palette-adjacent.  This still catches
            # completely foreign colours (e.g. a green from a wrong theme, or
            # terminal-default black).
            def _hex_to_rgb(h):
                if isinstance(h, str) and h.startswith("#") and len(h) == 7:
                    return (int(h[1:3], 16), int(h[3:5], 16), int(h[5:7], 16))
                return None

            def _near_palette(color_hex, pal):
                """Return True if color_hex is within 50 per channel of any palette colour."""
                rgb = _hex_to_rgb(color_hex)
                if rgb is None:
                    return False
                for p in pal:
                    pr = _hex_to_rgb(p)
                    if pr is None:
                        continue
                    if all(abs(a - b) <= 80 for a, b in zip(rgb, pr)):
                        return True
                return False

            if bg != "default" and not bg.startswith("indexed:"):
                # Tolerance: 80 per channel to accommodate powerkit-computed
                # intermediate colours (dimmed/brightened palette variants).
                if not _near_palette(bg, palette):
                    failures.append(
                        f"I1 status-row {sr} col {left+idx}: bg {bg!r} not near any palette colour"
                    )

            # I3: no default or black inside content span
            if bg == "default" or bg == "#000000":
                failures.append(
                    f"I3 status-row {sr} col {left+idx}: bg is default/black inside content span"
                )

            # I2: separator continuity
            # For powerline-style separators, we check:
            #   separator_bg == next_cell_bg
            # but only when the separator is a content-level boundary
            # (not a statusbar-surface spacer and not a same-family
            # dim-variant transition used by powerkit right-side plugins).
            # This specifically catches the reported GH-segment bug where
            # the git segment ends with a separator pointing directly into
            # a new, unrelated segment colour without a surface spacer.
            if cell.ch in SEPARATORS:
                abs_col = left + idx
                next_col = abs_col + 1
                # Skip if the PREVIOUS cell was also a separator (inner pair)
                prev_was_sep = (idx > 0 and content_cells[idx - 1].ch in SEPARATORS)
                # Skip if separator bg is the statusbar surface (expected
                # spacer->segment transition in powerkit right-side plugins)
                sep_is_surface = (bg == norm_hex(surface_hex))
                if next_col < cols and not prev_was_sep and not sep_is_surface:
                    next_bg = norm_hex(row_cells[next_col].bg)
                    # Allow transitions between near-palette colours
                    # (dim/bright variants of the same theme segment).
                    def _near_each_other(h1, h2):
                        r1 = _hex_to_rgb(h1); r2 = _hex_to_rgb(h2)
                        if r1 is None or r2 is None: return True
                        return all(abs(a - b) <= 60 for a, b in zip(r1, r2))
                    # Only flag if BOTH colours are non-surface content
                    # colours. segment->surface is expected powerkit.
                    next_is_surface = (next_bg == norm_hex(surface_hex))
                    if (bg != next_bg and bg != "default" and next_bg != "default"
                            and not next_is_surface
                            and not _near_each_other(bg, next_bg)):
                        failures.append(
                            f"I2 status-row {sr} col {abs_col}: separator bg {bg!r} != next-cell bg {next_bg!r}"
                        )

    # --- I4: pane area bg ~= theme bg ---
    pane_end_row = rows if top_has_content else max(0, rows - len(status_rows))
    pane_rows = list(range(pane_start_row, pane_end_row))
    blank_cells = []
    for r in pane_rows:
        for c in range(cols):
            cell = screen.grid[r][c]
            if cell.ch == " ":
                blank_cells.append(norm_hex(cell.bg))

    if blank_cells:
        mismatch = sum(1 for b in blank_cells if b not in (bg_hex, "default", surface_hex))
        ratio = mismatch / len(blank_cells)
        if ratio > 0.50:
            failures.append(
                f"I4 pane bg: {mismatch}/{len(blank_cells)} blank cells "
                f"({ratio:.0%}) have bg outside palette.bg={bg_hex!r}"
            )

    return failures


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
screen = replay_cast(cast_path)
failures = status_row_invariants(screen, palette_set, bg_hex, surface_hex)

if failures:
    for msg in failures:
        print(f"FAIL [{theme_slug}] {msg}")
    sys.exit(1)
else:
    print(f"PASS [{theme_slug}] all 4 invariants OK")
    sys.exit(0)
'

# Run cast invariant checks for a given cast file and palette tuple entry.
# Usage: check_cast_invariants <cast_path> <palette_tuple_string> <label>
# palette_tuple_string format: "slug:bg:fg:accent:green:orange:cyan:muted:surface"
check_cast_invariants() {
  local cast="$1"
  local palette_str="$2"
  local label="$3"

  if [ ! -f "${cast}" ]; then
    return 0  # validate_cast already reported failure
  fi

  # Parse fields
  local slug bg fg accent green orange cyan muted surface
  IFS=: read -r slug bg fg accent green orange cyan muted surface <<< "${palette_str}"

  local result
  result=$(python3 - "${cast}" "${slug}" "${bg}" "${fg}" "${accent}" \
           "${green}" "${orange}" "${cyan}" "${muted}" "${surface}" \
           <<< "${CAST_INVARIANTS_PY}" 2>&1) || true

  local exit_code=$?
  if echo "${result}" | grep -q "^FAIL"; then
    fail "${label}: cast invariants — $(echo "${result}" | head -3)"
    # Dump failure artefacts
    local fail_dir="/tmp/aibox-cast-failures/${slug}"
    mkdir -p "${fail_dir}"
    cp "${cast}" "${fail_dir}/cast.cast" 2>/dev/null || true
    echo "${result}" > "${fail_dir}/invariant-failures.txt"
  else
    pass "${label}: cast invariants"
  fi
}

# ─── Layout tests ─────────────────────────────────────────────────────────────

test_layouts() {
  info "Testing layouts..."

  for layout in "${LAYOUTS[@]}"; do
    cleanup_tmux
    local cast="${TEST_DIR}/layout-${layout}.cast"
    local driver
    driver=$(mktemp /tmp/test-XXXX.sh)
    cat > "${driver}" << EOF
#!/usr/bin/env bash
export TERM=xterm-256color COLORTERM=truecolor
tmux kill-session -t aibox-screencast 2>/dev/null || true
case "${layout}" in
  dev)
    tmux new-session -d -s aibox-screencast -n dev 'yazi 2>/dev/null || bash'
    tmux split-window -h -t aibox-screencast:dev 'vim 2>/dev/null || bash'
    tmux new-window -t aibox-screencast -n shell 'bash'
    ;;
  focus)
    tmux new-session -d -s aibox-screencast -n files 'yazi 2>/dev/null || bash'
    tmux new-window -t aibox-screencast -n editor 'vim 2>/dev/null || bash'
    tmux new-window -t aibox-screencast -n shell 'bash'
    ;;
  cowork)
    tmux new-session -d -s aibox-screencast -n cowork 'yazi 2>/dev/null || bash'
    tmux split-window -v -t aibox-screencast:cowork 'vim 2>/dev/null || bash'
    tmux split-window -h -t aibox-screencast:cowork 'printf "AI agent pane\n"; sleep infinity'
    ;;
esac
tmux set-option -t aibox-screencast status on
tmux set-option -t aibox-screencast status-left 'aibox #[bold]#S'
tmux set-option -t aibox-screencast status-right 'Prefix Ctrl-g | pane #{pane_index} | window #I:#W'
(sleep 2 && tmux kill-session -t aibox-screencast 2>/dev/null) &
tmux attach-session -t aibox-screencast 2>/dev/null
true
EOF
    chmod +x "${driver}"
    asciinema rec --cols 160 --rows 45 --overwrite -c "${driver}" "${cast}" 2>/dev/null || true
    rm -f "${driver}"
    if validate_cast "${cast}" "layout:${layout}"; then
      assert_cast_visible_status_text "${cast}" "layout:${layout}"
    fi
  done
}

# ─── Slug parser ─────────────────────────────────────────────────────────────
# Given a theme slug like "ayu-dark", "ayu-mirage", "catppuccin-mocha",
# produce a JSON object with family/mode/variant fields.
# Rules (heuristic, covers all 61 slugs in THEME_PALETTE_TABLE):
#   - suffix is the last dash-separated token
#   - canonical mode tokens: dark, light, day, black → mode field, variant=null
#   - other suffixes (mirage, storm, wave, dragon, lotus, moon, dawn, mocha,
#     macchiato, frappe, latte, ocean, palenight, lighter, darker, dimmed,
#     high-contrast, ochin, soft, 84) → variant field, mode inferred from bg
#   - mode inference: bg luminance < 50% → dark, else light
slug_to_json() {
  python3 - "$1" "$2" << 'SLUGPY'
import sys, json

slug = sys.argv[1]
bg   = sys.argv[2].lstrip("#")  # RRGGBB

CANONICAL_MODES = {"dark", "light", "day", "black"}
LIGHT_VARIANTS  = {"dawn", "latte", "lotus", "lighter", "light-high-contrast",
                   "ochin"}

parts = slug.split("-")
suffix = parts[-1]

# Handle multi-word suffixes joined by dash (high-contrast)
if len(parts) >= 2 and f"{parts[-2]}-{parts[-1]}" in {"high-contrast"}:
    suffix = f"{parts[-2]}-{parts[-1]}"
    family_parts = parts[:-2]
else:
    family_parts = parts[:-1]

family = "-".join(family_parts)

if suffix in CANONICAL_MODES:
    mode    = suffix
    variant = None
else:
    variant = suffix
    # Infer mode from bg luminance
    try:
        r = int(bg[0:2], 16)
        g = int(bg[2:4], 16)
        b = int(bg[4:6], 16)
        lum = 0.2126*r + 0.7152*g + 0.0722*b
        mode = "light" if lum > 127 else "dark"
    except Exception:
        mode = "dark"

print(json.dumps({"slug": slug, "family": family,
                  "mode": mode, "variant": variant,
                  "cast": f"themes/{slug}.cast"}))
SLUGPY
}

# ─── Index JSON writer ────────────────────────────────────────────────────────
write_themes_index() {
  local out_dir="$1"
  local ts
  ts=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  local entries=()
  for palette_str in "${THEME_PALETTE_TABLE[@]}"; do
    local slug bg
    IFS=: read -r slug bg _ <<< "${palette_str}"
    local entry
    entry=$(slug_to_json "${slug}" "${bg}")
    entries+=("${entry}")
  done

  python3 - "${ts}" "${out_dir}/index.json" "${entries[@]}" << 'IDXPY'
import sys, json

ts       = sys.argv[1]
out_path = sys.argv[2]
raw      = sys.argv[3:]   # one JSON string per theme

themes = [json.loads(e) for e in raw]
doc = {"version": 1, "generated_at": ts, "themes": themes}

with open(out_path, "w", encoding="utf-8") as fh:
    json.dump(doc, fh, indent=2)
    fh.write("\n")
print(f"index.json written: {len(themes)} themes → {out_path}")
IDXPY
}

# ─── Theme tests ──────────────────────────────────────────────────────────────
#
# For each theme in THEME_PALETTE_TABLE:
#   1. Record a 10-15s demo cast using the powerkit status bar colors.
#   2. Save cast to docs-site/static/asciinema/themes/<slug>.cast.
#   3. Run cast invariants (I1-I4) via the embedded Python helper.
#
# The palette table encodes: slug:bg:fg:accent:green:orange:cyan:muted:surface
# which covers all colors that appear in the powerkit status bar segments.
# Surface (statusbar-bg) is the 9th field.

test_themes() {
  info "Testing themes (powerkit-aware demo driver + ANSI status-bar invariants)..."

  local fail_dir="/tmp/aibox-cast-failures"
  mkdir -p "${fail_dir}"
  mkdir -p "${DOCS_CAST_DIR}"

  # Detect optional tools once; fall back gracefully if absent.
  local HAS_YAZI=0 HAS_VIM=0 HAS_STARSHIP=0 HAS_POWERKIT=0
  command -v yazi     &>/dev/null && HAS_YAZI=1
  command -v vim      &>/dev/null && HAS_VIM=1
  command -v starship &>/dev/null && HAS_STARSHIP=1
  [ -f /usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux ] && HAS_POWERKIT=1

  # Powerkit plugin path (system-installed by the runtime image)
  local POWERKIT_TMUX=/usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux
  local POWERKIT_ROOT=/usr/local/share/aibox/tmux/plugins/tmux-powerkit
  # Overrides file generated by aibox CLI for the active theme
  local POWERKIT_OVERRIDES="${HOME}/.config/tmux/aibox-powerkit-overrides.tmux"

  # Safe plugin list for demo recordings: no network calls, no API keys needed.
  # Omits: weather, github, kubernetes, terraform, modelstatus_*, externalip,
  #        ping, netspeed, ssh — all require outbound network or credentials.
  local SAFE_PLUGINS="git,uptime,datetime,hostname,cpu,loadavg,memory,disk"
  local SAFE_LINE1_RIGHT="uptime,datetime"
  local SAFE_LINE2_LEFT="git"
  local SAFE_LINE2_RIGHT="hostname,cpu,loadavg,memory,disk"

  for palette_str in "${THEME_PALETTE_TABLE[@]}"; do
    local slug bg fg accent green orange cyan muted surface
    IFS=: read -r slug bg fg accent green orange cyan muted surface <<< "${palette_str}"

    cleanup_demo_tmux "${slug}"
    # Cast is written directly to the docs-site static dir so it persists.
    local cast="${DOCS_CAST_DIR}/${slug}.cast"

    # ── Per-theme powerkit theme file ────────────────────────────────────────
    # Write a minimal aibox-powerkit-theme.sh for this theme slug so powerkit
    # can load the correct palette without touching the user's live theme file.
    local theme_tmpdir
    theme_tmpdir=$(mktemp -d /tmp/aibox-theme-XXXX)
    local theme_file="${theme_tmpdir}/aibox-powerkit-theme.sh"

    # Derive accent-family colors from the palette entry.
    # session-prefix-bg, session-copy-bg use accent + green; error-base uses
    # a muted-adjacent hue — just use the palette values we have.
    cat > "${theme_file}" << THEMEEOF
#!/usr/bin/env bash
# Generated by test-screencasts.sh for theme: ${slug}
declare -gA THEME_COLORS=(
    [background]="${bg}"
    [statusbar-bg]="${surface}"
    [statusbar-fg]="${fg}"
    [session-bg]="${accent}"
    [session-fg]="${bg}"
    [session-prefix-bg]="${orange}"
    [session-copy-bg]="${green}"
    [session-search-bg]="${orange}"
    [session-command-bg]="${muted}"
    [window-active-base]="${accent}"
    [window-active-style]="bold"
    [window-inactive-base]="${muted}"
    [window-inactive-style]="none"
    [window-activity-style]="italics"
    [window-bell-style]="bold"
    [window-zoomed-bg]="${green}"
    [pane-border-active]="${accent}"
    [pane-border-inactive]="${muted}"
    [ok-base]="${cyan}"
    [good-base]="${green}"
    [info-base]="${cyan}"
    [warning-base]="${orange}"
    [error-base]="${fg}"
    [disabled-base]="${muted}"
    [message-bg]="${surface}"
    [message-fg]="${fg}"
    [popup-bg]="${surface}"
    [popup-fg]="${fg}"
    [popup-border]="${accent}"
    [menu-bg]="${surface}"
    [menu-fg]="${fg}"
    [menu-selected-bg]="${accent}"
    [menu-selected-fg]="${bg}"
    [menu-border]="${accent}"
)
declare -gA THEME_EXTRA=(
    [orange]="${orange}"
    [magenta]="${muted}"
    [surface]="${surface}"
)
THEMEEOF

    # Recording socket path (tmux uses /tmp/tmux-UID/LABEL)
    local rec_socket_path="/tmp/tmux-$(id -u)/aibox-recordings"
    # Cache dir isolated from the user's live powerkit cache
    local pk_cache="${theme_tmpdir}/powerkit-cache"
    mkdir -p "${pk_cache}"

    # ── Demo driver ──────────────────────────────────────────────────────────
    # Spawns a dedicated tmux socket (aibox-recordings) so the user's live
    # session is never touched.  Sources the real powerkit plugin chain so the
    # status bar is rendered by the actual segment renderer, exercising the
    # separator and colour logic that causes the reported GH-segment bug.
    #
    # Graceful degradation:
    #   powerkit absent → falls back to literal format-string status bar
    #   yazi absent     → uses "ls -la ${PROJECT_ROOT}" in the left pane
    #   vim absent      → uses "cat ${PROJECT_ROOT}/aibox.toml" in the top-right
    #   starship absent → plain bash prompt in the bottom-right pane
    # ─────────────────────────────────────────────────────────────────────────
    local driver
    driver=$(mktemp /tmp/test-XXXX.sh)

    # Determine left-pane command
    local left_cmd
    if [ "${HAS_YAZI}" -eq 1 ]; then
      left_cmd="yazi ${PROJECT_ROOT}"
    else
      left_cmd="ls -la ${PROJECT_ROOT}; sleep infinity"
    fi

    # Determine top-right-pane command
    local tr_cmd
    if [ "${HAS_VIM}" -eq 1 ]; then
      tr_cmd="vim ${PROJECT_ROOT}/aibox.toml"
    else
      tr_cmd="cat ${PROJECT_ROOT}/aibox.toml; sleep infinity"
    fi

    # Determine bottom-right-pane command
    local br_cmd
    if [ "${HAS_STARSHIP}" -eq 1 ]; then
      br_cmd="bash --rcfile <(echo 'eval \"\$(starship init bash)\"')"
    else
      br_cmd="bash"
    fi

    cat > "${driver}" << DRIVEREOF
#!/usr/bin/env bash
# Demo driver — powerkit-aware.  Uses the real aibox-powerkit plugin chain so
# the recording exercises the actual segment renderer (separator glyphs, colour
# transitions) rather than literal tmux format strings.
#
# Safety: always uses -L aibox-recordings; never touches the user's live session.
export TERM=xterm-256color COLORTERM=truecolor
# Disable network-dependent powerkit features so API calls don't block recording.
export AIBOX_POWERKIT_GITHUB_DISABLED=1
export AIBOX_POWERKIT_WEATHER_DISABLED=1
export AIBOX_POWERKIT_KUBERNETES_DISABLED=1
export AIBOX_POWERKIT_TERRAFORM_DISABLED=1
# Point render scripts at the recording socket (not the live aibox.sock)
export AIBOX_TMUX_SOCKET="${rec_socket_path}"
# Isolate the powerkit cache from the live session's cache
export AIBOX_POWERKIT_CACHE_DIR="${pk_cache}"
export XDG_CACHE_HOME="${pk_cache}"
# Make HOME explicit so render scripts resolve ~/.local/bin correctly
export HOME="${HOME}"

SOCK=aibox-recordings
SESSION="demo-${slug}"

# Clean up any stale session on this socket
tmux -L "\${SOCK}" kill-session -t "\${SESSION}" 2>/dev/null || true

# ── Create 3-pane cowork-style layout ──────────────────────────────────────
# Left pane (full height): file navigator / ls fallback
tmux -L "\${SOCK}" new-session -d -s "\${SESSION}" -x 160 -y 45 \
  -n cowork "${left_cmd}"

# Split right half: top-right = editor
tmux -L "\${SOCK}" split-window -h -t "\${SESSION}:cowork" \
  -l 80 "${tr_cmd}"

# Split bottom of right half: shell pane
# Note: after the -h split, the second pane is active; split it vertically.
tmux -L "\${SOCK}" split-window -v -t "\${SESSION}:cowork" \
  -l 15 "${br_cmd}"

# Capture pane IDs for reliable send-keys targeting (indices vary by tmux version)
mapfile -t _PANES < <(tmux -L "\${SOCK}" list-panes -t "\${SESSION}:cowork" -F "#{pane_id}")
P_LEFT="\${_PANES[0]}"
P_EDITOR="\${_PANES[1]}"
P_SHELL="\${_PANES[2]}"

HAS_POWERKIT=${HAS_POWERKIT}

if [ "\${HAS_POWERKIT}" -eq 1 ]; then
  # ── Load the real powerkit plugin chain ──────────────────────────────────
  # 1. Source base tmux config so default options exist (minus plugin run-shells)
  tmux -L "\${SOCK}" set-option -g status-style "bg=${surface},fg=${fg}"
  tmux -L "\${SOCK}" set-option -g window-status-format " #I:#W "
  tmux -L "\${SOCK}" set-option -g window-status-current-format " #I:#W "
  tmux -L "\${SOCK}" set-option -g window-status-style "bg=${surface},fg=${muted}"
  tmux -L "\${SOCK}" set-option -g window-status-current-style "bg=${accent},fg=${bg},bold"
  tmux -L "\${SOCK}" set-option -g status-interval 10
  tmux -L "\${SOCK}" set-option -g default-terminal "tmux-256color"
  tmux -L "\${SOCK}" set-option -g pane-border-status top

  # 2. Set all @powerkit_* options that the plugin init reads
  tmux -L "\${SOCK}" set-option -g "@powerkit_theme" "custom"
  tmux -L "\${SOCK}" set-option -g "@powerkit_custom_theme_path" "${theme_file}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_separator_style" "normal"
  tmux -L "\${SOCK}" set-option -g "@powerkit_edge_separator_style" "rounded"
  tmux -L "\${SOCK}" set-option -g "@powerkit_elements_spacing" "both"
  tmux -L "\${SOCK}" set-option -g "@powerkit_status_interval" "10"
  tmux -L "\${SOCK}" set-option -g "@powerkit_transparent" "false"
  tmux -L "\${SOCK}" set-option -g "@powerkit_bar_layout" "double"
  tmux -L "\${SOCK}" set-option -g "@powerkit_status_order" "session,plugins"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_border_status" "top"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_border_unified" "false"
  tmux -L "\${SOCK}" set-option -g "@powerkit_active_pane_border_color" "${accent}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_inactive_pane_border_color" "${muted}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_border_status_bg" "${bg}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_scrollbars_style_fg" "${accent}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_scrollbars_style_bg" "${muted}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_pane_border_format" "#{?client_prefix,PREFIX,NORMAL} #{pane_title} #{pane_current_command}"
  # Use the safe (non-network) plugin subset for recordings
  tmux -L "\${SOCK}" set-option -g "@powerkit_plugins" "${SAFE_PLUGINS}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_line1_right" "${SAFE_LINE1_RIGHT}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_line2_left" "${SAFE_LINE2_LEFT}"
  tmux -L "\${SOCK}" set-option -g "@powerkit_line2_right" "${SAFE_LINE2_RIGHT}"

  # 3. Run the powerkit plugin initialiser on the recording socket.
  #    This writes the @_powerkit_* internal vars and (crucially) the
  #    status-format strings that invoke the real renderer.
  TMUX="\${AIBOX_TMUX_SOCKET},0,0" POWERKIT_ROOT="${POWERKIT_ROOT}" \
    bash "${POWERKIT_TMUX}" 2>/dev/null || true

  # 4. Source the aibox-powerkit-overrides (sets 2-row status-format strings
  #    and pane-border-format).  Guard: only source if the file exists.
  if [ -f "${POWERKIT_OVERRIDES}" ]; then
    tmux -L "\${SOCK}" source-file "${POWERKIT_OVERRIDES}" 2>/dev/null || true
  fi

  # 5. Force an immediate status refresh so the renderer populates the bar
  #    before the recording starts (powerkit fires on next status-interval
  #    otherwise, which is up to 10 s).
  tmux -L "\${SOCK}" refresh-client -S 2>/dev/null || true
  sleep 2

else
  # ── Fallback: literal format strings (no powerkit) ───────────────────────
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status on
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status-style "bg=${surface},fg=${fg}"
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status-left-length 40
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status-right-length 80
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status-left \
    "#[bg=${accent},fg=${bg},bold]  ${slug} #[bg=${surface},fg=${accent},nobold]"
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" status-right \
    "#[bg=${surface},fg=${muted}] GH #[bg=${muted},fg=${bg}]  main #[bg=${surface},fg=${muted}] Prefix #[bg=${muted},fg=${bg}] Ctrl-g #[bg=${surface},fg=${cyan}] pane #[bg=${cyan},fg=${bg}] 1 "
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" window-status-style \
    "bg=${surface},fg=${muted}"
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" window-status-current-style \
    "bg=${accent},fg=${bg},bold"
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" pane-border-style \
    "fg=${muted}"
  tmux -L "\${SOCK}" set-option -t "\${SESSION}" pane-active-border-style \
    "fg=${accent}"
fi

# ── Background activity + session killer ────────────────────────────────────
# Runs after layout renders; activity happens while asciinema records the attach.
(
  sleep 3

  # Navigate in left pane (yazi j/k keys, or ignored by ls)
  tmux -L "\${SOCK}" send-keys -t "\${P_LEFT}" "j" "" 2>/dev/null || true
  sleep 1
  tmux -L "\${SOCK}" send-keys -t "\${P_LEFT}" "j" "" 2>/dev/null || true
  sleep 0.5
  tmux -L "\${SOCK}" send-keys -t "\${P_LEFT}" "k" "" 2>/dev/null || true
  sleep 0.5

  # Switch to editor pane and scroll down
  tmux -L "\${SOCK}" select-pane -t "\${P_EDITOR}" 2>/dev/null || true
  sleep 0.5
  tmux -L "\${SOCK}" send-keys -t "\${P_EDITOR}" "3j" "" 2>/dev/null || true
  sleep 0.5

  # Switch to shell pane and run git log
  tmux -L "\${SOCK}" select-pane -t "\${P_SHELL}" 2>/dev/null || true
  sleep 0.5
  tmux -L "\${SOCK}" send-keys -t "\${P_SHELL}" \
    "git -C ${PROJECT_ROOT} log --oneline -5" Enter 2>/dev/null || true
  sleep 2

  # Hold final state then kill session to end the attach (and the recording)
  sleep 1
  tmux -L "\${SOCK}" kill-session -t "\${SESSION}" 2>/dev/null || true
) &

# Attach — asciinema records this terminal stream; exits when session is killed
tmux -L "\${SOCK}" attach-session -t "\${SESSION}" 2>/dev/null
true
DRIVEREOF
    chmod +x "${driver}"

    asciinema rec --cols 160 --rows 45 --idle-time-limit 1 --overwrite \
      -c "${driver}" "${cast}" 2>/dev/null || true
    rm -f "${driver}"
    rm -rf "${theme_tmpdir}"
    cleanup_demo_tmux "${slug}"

    if validate_cast "${cast}" "theme:${slug}" 10 2000; then
      check_cast_invariants "${cast}" "${palette_str}" "theme:${slug}"
    fi
  done

  # Write the index JSON after all recordings
  write_themes_index "${DOCS_CAST_DIR}"
  pass "themes index.json written"
}

# ─── Tool smoke tests ────────────────────────────────────────────────────────

test_tools() {
  info "Testing tools..."

  declare -A tools=(
    [tmux]="tmux -V"
    [yazi]="yazi --version"
    [vim]="vim --version"
    [lazygit]="lazygit --version"
    [git]="git --version"
    [gh]="gh --version"
  )

  for tool in "${!tools[@]}"; do
    if ! command -v "${tool}" &>/dev/null; then
      skip "tool:${tool} (not installed)"
      continue
    fi

    local cast="${TEST_DIR}/tool-${tool}.cast"
    local cmd="${tools[${tool}]}"
    asciinema rec --cols 80 --rows 10 --overwrite \
      -c "${cmd}" "${cast}" 2>/dev/null || true
    validate_cast "${cast}" "tool:${tool}" 2 100
  done
}

# ─── CLI tests ────────────────────────────────────────────────────────────────

test_cli() {
  info "Testing CLI..."

  local devbox=""
  if [ -x "${PROJECT_ROOT}/cli/target/release/aibox" ]; then
    devbox="${PROJECT_ROOT}/cli/target/release/aibox"
  elif [ -x "${PROJECT_ROOT}/cli/target/debug/aibox" ]; then
    devbox="${PROJECT_ROOT}/cli/target/debug/aibox"
  else
    skip "cli:init (aibox binary not found)"
    skip "cli:doctor (aibox binary not found)"
    return
  fi

  # Test init
  local workdir
  workdir=$(mktemp -d /tmp/test-init-XXXX)
  local cast="${TEST_DIR}/cli-init.cast"
  asciinema rec --cols 100 --rows 20 --overwrite \
    -c "cd ${workdir} && ${devbox} init test --context minimal 2>&1" \
    "${cast}" 2>/dev/null || true
  if [ -f "${workdir}/aibox.toml" ]; then
    pass "cli:init (aibox.toml created)"
  else
    fail "cli:init (aibox.toml not found)"
  fi
  rm -rf "${workdir}"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

info "Visual smoke tests (output: ${TEST_DIR})"
echo ""

MODE="${1:-all}"

case "${MODE}" in
  layouts) test_layouts ;;
  themes)  test_themes ;;
  tools)   test_tools ;;
  cli)     test_cli ;;
  all)
    test_layouts
    echo ""
    test_themes
    echo ""
    test_tools
    echo ""
    test_cli
    ;;
  *)
    echo "Usage: $0 [all|layouts|themes|tools|cli]" >&2
    exit 1
    ;;
esac

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
info "Results: ${PASSES} passed, ${FAILURES} failed, ${SKIPS} skipped"

# Cleanup
rm -rf "${TEST_DIR}"
cleanup_tmux

if [ "${FAILURES}" -gt 0 ]; then
  exit 1
fi
