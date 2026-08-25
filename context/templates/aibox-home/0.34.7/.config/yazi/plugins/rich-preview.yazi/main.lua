-- rich-preview.yazi — terminal-rich Markdown / JSON / RST / ipynb preview
--
-- Renders the file through Python's `rich` library (Markdown for .md/.markdown,
-- Syntax for everything else with line numbers) so the in-pane preview gets
-- the same visual fidelity as `glow` would for a full-pane preview.
--
-- Cache layer
-- -----------
-- Python startup + rich rendering is expensive (~100–300 ms per call). Without
-- a cache, every Alt-J / Alt-K scroll keystroke re-renders the whole document
-- to slice off a different window, which makes long-document scrolling feel
-- laggy. The plugin instead renders once into a cache file keyed by
-- (file path, mtime, terminal width) and reads windowed slices from the cache
-- on subsequent peeks. The cache lives under
-- `${XDG_CACHE_HOME:-~/.cache}/aibox-yazi-rich-preview/` and is purged per
-- source path whenever a new (mtime, width) combination renders.
--
-- Position indicator
-- ------------------
-- The Python side counts every rendered line (BEFORE windowing) and writes
-- the total as a `__YAZI_TOTAL__<N>` sentinel as the first line of the cache
-- file. The Lua side reads that, then renders a small bottom-right overlay
-- showing the current visible line range and the percentage scrolled — so the
-- user can see *where* in a long document they are while skimming with
-- Alt-J / Alt-K.

local M = {}

local PY_RENDER = [[
import io
import os
import pathlib
import sys
import tempfile

src = pathlib.Path(sys.argv[1])
width = int(sys.argv[2])
cache = pathlib.Path(sys.argv[3])
text = src.read_text(errors="replace")

def split_front_matter(value):
    lines = value.splitlines()
    if not lines or lines[0] not in {"+++", "---"}:
        return None, value, None
    fence = lines[0]
    try:
        end = lines.index(fence, 1)
    except ValueError:
        return None, value, None
    lexer = "toml" if fence == "+++" else "yaml"
    front_matter = "\n".join(lines[: end + 1])
    body = "\n".join(lines[end + 1 :]).lstrip("\n")
    return front_matter, body, lexer

try:
    from rich.console import Console
    from rich.markdown import Markdown
    from rich.syntax import Syntax
except Exception:
    # Fallback: emit a zero total so the Lua side knows the indicator is
    # meaningless, then dump the raw text. Still writes the cache so that
    # repeated peeks don't re-import.
    cache.parent.mkdir(parents=True, exist_ok=True)
    fd, tmpname = tempfile.mkstemp(prefix=cache.name + ".", dir=str(cache.parent))
    with os.fdopen(fd, "w") as fh:
        fh.write("__YAZI_TOTAL__0\n")
        fh.write(text)
    os.replace(tmpname, str(cache))
    sys.exit(0)

buf = io.StringIO()
console = Console(
    file=buf,
    width=width,
    force_terminal=True,
    color_system="truecolor",
    soft_wrap=False,
)
if src.suffix.lower() in {".md", ".markdown"}:
    front_matter, body, lexer = split_front_matter(text)
    if front_matter is not None:
        # Markdown soft-break rules collapse front-matter rows into one
        # paragraph. Render the fenced metadata verbatim, then render only the
        # document body as Markdown.
        console.print(Syntax(front_matter, lexer, theme="ansi_dark", word_wrap=False))
    if body:
        console.print(Markdown(body))
else:
    language = src.suffix.lstrip(".") or "text"
    console.print(Syntax(text, language, theme="ansi_dark", line_numbers=False, word_wrap=False))

lines = buf.getvalue().splitlines()

cache.parent.mkdir(parents=True, exist_ok=True)
# Write atomically so concurrent peeks never see a partial cache.
fd, tmpname = tempfile.mkstemp(prefix=cache.name + ".", dir=str(cache.parent))
try:
    with os.fdopen(fd, "w") as fh:
        fh.write("__YAZI_TOTAL__%d\n" % len(lines))
        for ln in lines:
            fh.write(ln + "\n")
    os.replace(tmpname, str(cache))
except Exception:
    try:
        os.unlink(tmpname)
    except OSError:
        pass
    raise

# Purge older cache files for the same source path (different mtime / width).
prefix = cache.name.rsplit("-", 2)[0]
for sibling in cache.parent.iterdir():
    if sibling.name.startswith(prefix + "-") and sibling != cache:
        try:
            sibling.unlink()
        except OSError:
            pass
]]

local function cache_root()
	local base = os.getenv("XDG_CACHE_HOME")
	if not base or base == "" then
		base = (os.getenv("HOME") or "/tmp") .. "/.cache"
	end
	return base .. "/aibox-yazi-rich-preview"
end

local function path_key(url)
	-- Hash the entire path into a bounded, filesystem-safe key. The previous
	-- implementation truncated a hex-encoded path to its first 32 characters,
	-- so files sharing the same first 16 path bytes collided whenever their
	-- mtimes and preview widths also matched.
	local s = tostring(url)
	local h1, h2 = 5381, 52711
	for i = 1, #s do
		local byte = s:byte(i)
		h1 = (h1 * 33 + byte) % 4294967296
		h2 = (h2 * 65599 + byte) % 4294967296
	end
	return string.format("%08x%08x", h1, h2)
end

local function cache_file(url, mtime, width)
	return string.format(
		"%s/%s-%d-%d.cache",
		cache_root(),
		path_key(url),
		mtime,
		width
	)
end

local function read_window(path, skip, height)
	local f = io.open(path, "r")
	if not f then
		return nil
	end
	local first = f:read("*l") or ""
	local total = tonumber(first:match("^__YAZI_TOTAL__(%d+)"))
	if total == nil then
		f:close()
		return nil
	end
	local out, i = {}, 0
	for line in f:lines() do
		i = i + 1
		if i > skip then
			out[#out + 1] = line
			if #out >= height then
				break
			end
		end
	end
	f:close()
	return total, table.concat(out, "\n"), i
end

function M:peek(job)
	local preview_opts = require("preview-options"):options()
	local cha = job.file.cha
	local mtime = math.floor((cha and cha.mtime) or 0)
	local cpath = cache_file(job.file.url, mtime, job.area.w)

	local function fallback()
		return require("code"):peek(job)
	end

	-- Cache miss → spawn Python once to render to disk. Subsequent peeks at
	-- different skip offsets re-enter and find the cache populated.
	if not fs.cha(Url(cpath)) then
		local child = Command("python3")
			:env("COLUMNS", tostring(job.area.w))
			:arg({
				"-c",
				PY_RENDER,
				tostring(job.file.url),
				tostring(job.area.w),
				cpath,
			})
			:stdout(Command.PIPED)
			:stderr(Command.PIPED)
			:spawn()

		if not child then
			return fallback()
		end
		local status = child:wait()
		if not status or not status.success then
			return fallback()
		end
	end

	local content_h = math.max(1, job.area.h - 1)
	local total, lines, last = read_window(cpath, job.skip, content_h)
	if total == nil then
		return fallback()
	end

	if job.skip > 0 and last < job.skip + content_h then
		ya.emit("peek", { math.max(0, last - content_h), only_if = job.file.url, upper_bound = true })
		return
	end

	lines = lines:gsub("\t", string.rep(" ", rt.preview.tab_size))
	if preview_opts.numbers then
		local numbered, current = {}, job.skip + 1
		for line in (lines .. "\n"):gmatch("(.-)\n") do
			numbered[#numbered + 1] = string.format("%5d │ %s", current, line)
			current = current + 1
		end
		lines = table.concat(numbered, "\n")
	end

	-- Build the position indicator. `total == 0` (renderer fell back to
	-- raw text) → suppress the indicator since it would be misleading.
	local indicator_text
	if total > 0 then
		local visible_first = job.skip + 1
		local visible_last = math.min(total, job.skip + content_h)
		local pct
		if total <= content_h then
			pct = 100
		else
			-- Bias so reaching the bottom shows 100, top shows 0.
			pct = math.floor(0.5 + (job.skip / math.max(1, total - content_h)) * 100)
		end
		indicator_text = string.format(" L%d–%d / %d  %d%% ", visible_first, visible_last, total, pct)
	else
		indicator_text = " (no position info) "
	end

	-- Right-align the indicator on the last row.
	local pad = job.area.w - #indicator_text
	if pad < 0 then
		pad = 0
		indicator_text = string.sub(indicator_text, 1, job.area.w)
	end
	local indicator_line = string.rep(" ", pad) .. indicator_text

	local content_area = ui.Rect({
		x = job.area.x,
		y = job.area.y,
		w = job.area.w,
		h = content_h,
	})
	local indicator_area = ui.Rect({
		x = job.area.x,
		y = job.area.y + content_h,
		w = job.area.w,
		h = 1,
	})

	-- Yazi 26.x exposes `ya.preview_widget` (singular). It accepts either a
	-- single widget or a table of widgets — the latter is how we overlay the
	-- bottom-row position indicator on top of the content area.
	ya.preview_widget(job, {
		ui.Text.parse(lines)
			:area(content_area)
			:wrap(preview_opts.wrap and ui.Wrap.YES or ui.Wrap.NO),
		ui.Text(indicator_line):area(indicator_area):style(ui.Style():reverse()),
	})
end

function M:seek(job)
	require("code"):seek(job)
end

return M
