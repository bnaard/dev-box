#!/usr/bin/env bash
# record-asciinema.sh — generate documentation screencasts using asciinema
#
# Records terminal sessions as .cast files (asciicast v2 format) that can be
# played back with the asciinema-player in docs, or converted to GIF via agg.
# These are compact documentation demos, not release validation; generated
# runtime layout coverage lives in cli/tests/e2e/visual_matrix.rs and
# scripts/release-runtime-smoke.sh.
#
# No sibling containers, no Docker socket, no Chromium — just a PTY.
#
# Prerequisites:
#   - asciinema (pip/uv: asciinema)
#   - agg (cargo install --git https://github.com/asciinema/agg) — optional, for GIF export
#   - tmux (for layout recordings)
#   - A running aibox container with tmux + starship installed (for prompt recordings)
#
# Usage:
#   ./scripts/record-asciinema.sh              # record all (layouts + themes + demos)
#   ./scripts/record-asciinema.sh layouts      # only layout recordings
#   ./scripts/record-asciinema.sh themes       # only theme tour recordings
#   ./scripts/record-asciinema.sh demos        # only CLI demo recordings
#   ./scripts/record-asciinema.sh gif          # generate GIFs via agg
#   ./scripts/record-asciinema.sh readme       # generate README animated GIF
#
# TODO (BACK-062): Add a `prompts` mode that records each Starship prompt preset.
# Each recording should show the prompt in a shell session inside an aibox container:
#   - Spin up a container with the given preset (aibox init --prompt <preset> && aibox up)
#   - Run a short scripted session: cd to a git repo, run a failing command, show the prompt
#   - Output files: prompt-default.cast, prompt-plain.cast, prompt-minimal.cast,
#                   prompt-nerd-font.cast, prompt-pastel.cast, prompt-bracketed.cast, prompt-arrow.cast
# This requires a running container environment so cannot be done in the dev-container build step.
# Docs placeholders: see docs-site/content/docs/customization/prompts.md (<!-- recording pending --> comments)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${PROJECT_ROOT}/docs-site/static/screencasts"

# Terminal dimensions for recordings
LAYOUT_COLS=160
LAYOUT_ROWS=45
DEMO_COLS=100
DEMO_ROWS=30

# Available themes (must match generated aibox theme names)
THEMES=(gruvbox-dark catppuccin-mocha catppuccin-latte dracula tokyo-night nord)

# Theme bg/fg colors (must match generated terminal theme palettes)
declare -A THEME_BG=(
  [gruvbox-dark]="#282828"
  [catppuccin-mocha]="#1E1E2E"
  [catppuccin-latte]="#EFF1F5"
  [dracula]="#282A36"
  [tokyo-night]="#1A1B26"
  [nord]="#2E3440"
)
declare -A THEME_FG=(
  [gruvbox-dark]="#D5C4A1"
  [catppuccin-mocha]="#CDD6F4"
  [catppuccin-latte]="#4C4F69"
  [dracula]="#F8F8F2"
  [tokyo-night]="#C0CAF5"
  [nord]="#D8DEE9"
)

info()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
ok()    { printf '\033[1;32m ✓\033[0m  %s\n' "$1"; }
warn()  { printf '\033[1;33m !\033[0m  %s\n' "$1"; }
die()   { printf '\033[1;31mError:\033[0m %s\n' "$1" >&2; exit 1; }

# Clean up any leftover documentation tmux session before recording.
cleanup_tmux() {
  tmux kill-session -t aibox-screencast 2>/dev/null || true
  sleep 0.5
}

# Trim a cast file: keep only events between first large render and last large render.
# Removes shell startup noise before tmux and exit noise after.
# Preserves any OSC escape sequences (terminal color settings) from early events.
trim_cast() {
  local cast="$1"
  python3 - "${cast}" << 'PYEOF'
import json, sys, re

cast_path = sys.argv[1]
lines = open(cast_path).readlines()
header = lines[0]
events = []
for line in lines[1:]:
    try:
        events.append(json.loads(line))
    except:
        pass

if not events:
    sys.exit(0)

# Collect OSC sequences (terminal color settings) from all events
osc_data = ""
for ev in events:
    # Match OSC 10 (fg) and OSC 11 (bg) sequences
    oscs = re.findall(r'\x1b\]1[01];#[A-Fa-f0-9]+\x1b\\\\?', ev[2])
    for osc in oscs:
        osc_data += osc

# Find first event with >500 bytes (tmux first render)
first = 0
for i, ev in enumerate(events):
    if len(ev[2]) > 500:
        first = i
        break

# Find last event with >500 bytes (last real render before exit)
last = len(events) - 1
for i in range(len(events) - 1, -1, -1):
    if len(events[i][2]) > 100:
        last = i
        break

# Rebase timestamps to start at 0
trimmed = events[first:last+1]
if trimmed:
    t0 = trimmed[0][0]
    for ev in trimmed:
        ev[0] = round(ev[0] - t0, 6)

# Prepend OSC sequences as the first event (at t=0) so player picks up colors
if osc_data and trimmed:
    trimmed.insert(0, [0.0, "o", osc_data])

with open(cast_path, 'w') as f:
    f.write(header)
    for ev in trimmed:
        f.write(json.dumps(ev) + '\n')
PYEOF
}

assert_cast_visible_status_text() {
  local cast="$1"
  local label="$2"

  python3 - "${cast}" << 'PYEOF' || die "${label}: missing visible tmux status/keybar text"
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
}

# ─── Layout recording ────────────────────────────────────────────────────────
# Records a tmux documentation demo session headlessly via asciinema.
# tmux runs in foreground inside asciinema's PTY; a background process
# kills it after DURATION seconds.

record_layout() {
  local layout="$1"
  local duration="${2:-5}"
  local output="${OUTPUT_DIR}/layout-${layout}.cast"

  info "Recording layout: ${layout} (${duration}s)..."
  cleanup_tmux

  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
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
(sleep ${duration} && tmux kill-session -t aibox-screencast 2>/dev/null) &
tmux attach-session -t aibox-screencast 2>/dev/null
true
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${LAYOUT_COLS}" \
    --rows "${LAYOUT_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}"
  trim_cast "${output}"
  assert_cast_visible_status_text "${output}" "layout-${layout}.cast"
  ok "layout-${layout}.cast ($(wc -l < "${output}") events)"
}

# ─── Theme tour recording ────────────────────────────────────────────────────
# Records the dev layout with a specific theme, cycling through tabs to show
# all themed tools: Yazi + Vim (tab 1), lazygit (tab 3), shell/starship (tab 4).
#
# Uses tmux window-selection commands to cycle through the generated workspace.

record_theme() {
  local theme="$1"
  local output="${OUTPUT_DIR}/theme-${theme}.cast"

  info "Recording theme: ${theme}..."
  cleanup_tmux

  # Look up theme colors for OSC injection
  local bg="${THEME_BG[${theme}]:-#000000}"
  local fg="${THEME_FG[${theme}]:-#FFFFFF}"

  # Driver script: set terminal bg/fg via OSC, then start tmux with theme-like
  # status colors and cycle representative windows.
  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
# Set terminal background/foreground to match theme (OSC 11/10)
printf '\033]11;${bg}\033\\\\'
printf '\033]10;${fg}\033\\\\'
tmux kill-session -t aibox-screencast 2>/dev/null || true
tmux new-session -d -s aibox-screencast -n dev 'yazi 2>/dev/null || bash'
tmux split-window -h -t aibox-screencast:dev 'vim 2>/dev/null || bash'
tmux new-window -t aibox-screencast -n git 'lazygit 2>/dev/null || git status --short --branch; sleep infinity'
tmux new-window -t aibox-screencast -n shell 'bash'
tmux set-option -t aibox-screencast status on
tmux set-option -t aibox-screencast status-style 'bg=${bg},fg=${fg}'
tmux set-option -t aibox-screencast status-left 'aibox ${theme}'
tmux set-option -t aibox-screencast status-right 'Prefix Ctrl-g | tmux'
(
  sleep 3
  tmux select-window -t aibox-screencast:git 2>/dev/null || true
  sleep 2
  tmux select-window -t aibox-screencast:shell 2>/dev/null || true
  sleep 2
  tmux select-window -t aibox-screencast:dev 2>/dev/null || true
  sleep 1
  tmux kill-session -t aibox-screencast 2>/dev/null || true
) &
tmux attach-session -t aibox-screencast 2>/dev/null
true
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${LAYOUT_COLS}" \
    --rows "${LAYOUT_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}"
  trim_cast "${output}"
  assert_cast_visible_status_text "${output}" "theme-${theme}.cast"
  ok "theme-${theme}.cast ($(wc -l < "${output}") events)"
}

# ─── CLI demo recording ──────────────────────────────────────────────────────
# Records a scripted CLI demo (e.g., aibox init) with simulated typing.

record_init_demo() {
  local output="${OUTPUT_DIR}/init-demo.cast"

  info "Recording demo: init..."

  local driver
  driver=$(mktemp /tmp/record-XXXX.sh)
  local workdir
  workdir=$(mktemp -d /tmp/demo-project-XXXX)

  cat > "${driver}" << DRIVER
#!/usr/bin/env bash
export TERM=xterm-256color
export COLORTERM=truecolor
export PATH="${PROJECT_ROOT}/cli/target/release:${PROJECT_ROOT}/cli/target/debug:\$PATH"
cd "${workdir}"

# Simulate typing: mkdir + cd
sleep 0.5
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in m k d i r ' ' m y - p r o j e c t ' ' '&' '&' ' ' c d ' ' m y - p r o j e c t; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
mkdir -p my-project && cd my-project

sleep 0.3
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in a i b o x ' ' i n i t ' ' m y - p r o j e c t ' ' - - y e s ' ' - - b a s e ' ' d e b i a n ' ' - - p r o f i l e ' ' h u m a n - d e v ' ' - - u s e r ' ' a i b o x ' ' - - t h e m e ' ' g r u v b o x ' ' - - p r o m p t ' ' d e f a u l t ' ' - - t m u x - s t a t u s ' ' e x t e n d e d ' ' - - a d d o n ' ' p y t h o n ' ' - - a d d o n - t o o l ' ' p y t h o n : p y t h o n = 3 . 1 4 ' ' - - a d d o n - t o o l ' ' p y t h o n : u v = 0 . 1 1 . 1 9 ' ' - - h a r n e s s ' ' c l a u d e ' ' - - p r o c e s s k i t - v e r s i o n ' ' v 0 . 2 6 . 1 5 ' ' - - n o - c o n t a i n e r; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
printf 'y\n' | aibox init my-project --yes --base debian --profile human-dev --user aibox --theme gruvbox --prompt default --tmux-status extended --addon python --addon-tool python:python=3.14 --addon-tool python:uv=0.12.5 --harness claude --processkit-version v0.26.15 --no-container 2>&1 || true

sleep 1
echo -ne '\033[32m❯\033[0m '
sleep 0.3
for c in c a t ' ' a i b o x . t o m l; do
  printf '%s' "\$c"
  sleep 0.06
done
echo
cat aibox.toml 2>/dev/null || echo "(aibox.toml would appear here)"

sleep 2
DRIVER
  chmod +x "${driver}"

  asciinema rec \
    --cols "${DEMO_COLS}" \
    --rows "${DEMO_ROWS}" \
    --overwrite \
    -c "${driver}" \
    "${output}" 2>/dev/null

  rm -f "${driver}"
  rm -rf "${workdir}"
  ok "init-demo.cast"
}

# ─── GIF export via agg ──────────────────────────────────────────────────────

generate_gifs() {
  local pattern="${1:-*.cast}"

  if ! command -v agg &>/dev/null; then
    warn "agg not found — skipping GIF generation (cargo install --git https://github.com/asciinema/agg)"
    return
  fi

  info "Generating GIFs..."
  for cast in "${OUTPUT_DIR}"/${pattern}; do
    [ -f "${cast}" ] || continue
    local name
    name=$(basename "${cast}" .cast)
    local gif="${OUTPUT_DIR}/${name}.gif"
    agg "${cast}" "${gif}" 2>/dev/null
    ok "${name}.gif ($(du -h "${gif}" | cut -f1))"
  done
}

generate_readme_gif() {
  local cast="${OUTPUT_DIR}/layout-dev.cast"
  local gif="${PROJECT_ROOT}/docs-site/static/screencasts/readme-dev-layout.gif"

  if ! command -v agg &>/dev/null; then
    warn "agg not found — skipping README GIF"
    return
  fi

  [ -f "${cast}" ] || die "layout-dev.cast not found — run 'layouts' first"

  info "Generating README GIF..."
  agg "${cast}" "${gif}" 2>/dev/null
  ok "readme-dev-layout.gif ($(du -h "${gif}" | cut -f1))"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

mkdir -p "${OUTPUT_DIR}"

MODE="${1:-all}"

case "${MODE}" in
  layouts)
    record_layout dev 5
    record_layout focus 5
    record_layout cowork 5
    ;;
  themes)
    for theme in "${THEMES[@]}"; do
      record_theme "${theme}"
    done
    ;;
  demos)
    record_init_demo
    ;;
  gif)
    generate_gifs
    ;;
  readme)
    generate_readme_gif
    ;;
  all)
    record_layout dev 5
    record_layout focus 5
    record_layout cowork 5
    for theme in "${THEMES[@]}"; do
      record_theme "${theme}"
    done
    record_init_demo
    info "All recordings complete."
    echo ""
    generate_gifs
    generate_readme_gif
    ;;
  *)
    die "Unknown mode: ${MODE} (use: all, layouts, themes, demos, gif, readme)"
    ;;
esac

echo ""
info "Cast files:"
ls -1 "${OUTPUT_DIR}"/*.cast 2>/dev/null || echo "  (none)"
echo ""
if ls "${OUTPUT_DIR}"/*.gif &>/dev/null 2>&1; then
  info "GIF files:"
  ls -1 "${OUTPUT_DIR}"/*.gif
fi
