local M = {}

function M:peek(job)
	local source = [[
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
width = int(sys.argv[2])
text = path.read_text(errors="replace")

try:
    from rich.console import Console
    from rich.markdown import Markdown
    from rich.syntax import Syntax
except Exception:
    print(text)
    raise SystemExit(0)

console = Console(width=width, force_terminal=True, color_system="truecolor", soft_wrap=False)

if path.suffix.lower() in {".md", ".markdown"}:
    console.print(Markdown(text))
else:
    language = path.suffix.lstrip(".") or "text"
    console.print(Syntax(text, language, theme="ansi_dark", line_numbers=True, word_wrap=False))
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

	local limit = job.area.h
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
	until i >= job.skip + limit

	child:start_kill()
	if job.skip > 0 and i < job.skip + limit then
		ya.emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		lines = lines:gsub("	", string.rep(" ", rt.preview.tab_size))
		ya.preview_widget(
			job,
			ui.Text.parse(lines):area(job.area):wrap(rt.preview.wrap == "yes" and ui.Wrap.YES or ui.Wrap.NO)
		)
	end
end

function M:seek(job)
	require("code"):seek(job)
end

return M
