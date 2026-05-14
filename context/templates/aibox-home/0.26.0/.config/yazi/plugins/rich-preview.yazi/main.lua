-- rich-preview.yazi — terminal-rich Markdown / JSON / RST / ipynb preview
--
-- Renders the file through Python's `rich` library (Markdown for .md/.markdown,
-- Syntax for everything else with line numbers) so the in-pane preview gets
-- the same visual fidelity as `glow` would for a full-pane preview.
--
-- Position indicator
-- ------------------
-- The Python side counts every rendered line (BEFORE windowing) and prints
-- the total as a `__YAZI_TOTAL__<N>` sentinel on its first stdout line.
-- The Lua side reads that, then renders a small bottom-right overlay
-- showing the current visible line range and the percentage scrolled —
-- so the user can see *where* in a long document they are while
-- skimming with Alt-J / Alt-K.

local M = {}

function M:peek(job)
	local source = [[
import pathlib
import sys
import io

path = pathlib.Path(sys.argv[1])
width = int(sys.argv[2])
text = path.read_text(errors="replace")

try:
    from rich.console import Console
    from rich.markdown import Markdown
    from rich.syntax import Syntax
except Exception:
    # Fallback: emit a zero total so the Lua side knows the indicator
    # is meaningless, then dump the raw text.
    sys.stdout.write("__YAZI_TOTAL__0\n")
    sys.stdout.write(text)
    raise SystemExit(0)

# Render into a buffer first so we can count total rendered lines for
# the position indicator. `record=True` would also work but using
# explicit buffer keeps memory usage predictable for large docs.
buf = io.StringIO()
console = Console(
    file=buf,
    width=width,
    force_terminal=True,
    color_system="truecolor",
    soft_wrap=False,
)
if path.suffix.lower() in {".md", ".markdown"}:
    console.print(Markdown(text))
else:
    language = path.suffix.lstrip(".") or "text"
    console.print(Syntax(text, language, theme="ansi_dark", line_numbers=True, word_wrap=False))

rendered = buf.getvalue()
# Splitlines without keepends so we can count exactly. Re-attach \n
# per line on output so the Lua side reads them one at a time.
lines = rendered.splitlines()
sys.stdout.write("__YAZI_TOTAL__%d\n" % len(lines))
for ln in lines:
    sys.stdout.write(ln + "\n")
sys.stdout.flush()
]]

	local child = Command("python3")
		:env("COLUMNS", tostring(job.area.w))
		:arg({
			"-c",
			source,
			tostring(job.file.url),
			tostring(job.area.w),
		})
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return require("code"):peek(job)
	end

	-- Read the sentinel line. If it's missing the renderer fell over and
	-- we hand off to the built-in code previewer for safety.
	local first_line, first_event = child:read_line()
	if first_event ~= 0 or first_line == nil then
		child:start_kill()
		return require("code"):peek(job)
	end
	local total = tonumber(first_line:match("^__YAZI_TOTAL__(%d+)") or "")
	if total == nil then
		child:start_kill()
		return require("code"):peek(job)
	end

	-- Reserve the last row for the position indicator. The content
	-- pane is one row shorter than `job.area`.
	local content_h = math.max(1, job.area.h - 1)

	local i, lines = 0, ""
	repeat
		local next, event = child:read_line()
		if event == 1 then
			return require("code"):peek(job)
		elseif event ~= 0 then
			break
		end

		i = i + 1
		if i > job.skip then
			lines = lines .. next
		end
	until i >= job.skip + content_h

	child:start_kill()
	if job.skip > 0 and i < job.skip + content_h then
		ya.emit("peek", { math.max(0, i - content_h), only_if = job.file.url, upper_bound = true })
		return
	end

	lines = lines:gsub("	", string.rep(" ", rt.preview.tab_size))

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

	-- Right-align the indicator on the last row. Pad the left edge with
	-- spaces so the text sits flush right and the bar bg covers the row
	-- to give it a status-line feel.
	local pad = job.area.w - #indicator_text
	if pad < 0 then
		pad = 0
		indicator_text = string.sub(indicator_text, 1, job.area.w)
	end
	local indicator_line = string.rep(" ", pad) .. indicator_text

	-- The content area is the upper region; the indicator gets the bottom row.
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

	ya.preview_widgets(job, {
		ui.Text.parse(lines)
			:area(content_area)
			:wrap(rt.preview.wrap == "yes" and ui.Wrap.YES or ui.Wrap.NO),
		ui.Text(indicator_line):area(indicator_area):style(ui.Style():reverse()),
	})
end

function M:seek(job)
	require("code"):seek(job)
end

return M
