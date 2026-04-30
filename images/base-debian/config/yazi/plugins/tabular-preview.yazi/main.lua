local M = {}

local function shell_quote(value)
	return "'" .. tostring(value):gsub("'", "'\\''") .. "'"
end

local function preview_command(job, command)
	local child = Command("sh")
		:arg({ "-lc", command })
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

function M:peek(job)
	local path = tostring(job.file.url)
	local quoted = shell_quote(path)
	local lower = path:lower()
	local command

	if lower:match("%.xlsx?$") then
		command = table.concat({
			"if ! command -v in2csv >/dev/null 2>&1 || ! command -v csvlook >/dev/null 2>&1; then",
			"  echo 'Spreadsheet preview requires: aibox set addon data-preview enabled --apply'; exit 0;",
			"fi;",
			"in2csv " .. quoted .. " 2>/dev/null | csvlook --max-rows 200 --max-columns 40 2>&1",
		}, " ")
	elseif lower:match("%.tsv$") then
		command = table.concat({
			"if ! command -v csvlook >/dev/null 2>&1; then",
			"  echo 'TSV preview requires: aibox set addon data-preview enabled --apply'; exit 0;",
			"fi;",
			"csvlook --tabs --max-rows 200 --max-columns 40 " .. quoted .. " 2>&1",
		}, " ")
	else
		command = table.concat({
			"if ! command -v csvlook >/dev/null 2>&1; then",
			"  echo 'CSV preview requires: aibox set addon data-preview enabled --apply'; exit 0;",
			"fi;",
			"csvlook --max-rows 200 --max-columns 40 " .. quoted .. " 2>&1",
		}, " ")
	end

	preview_command(job, command)
end

function M:seek(job)
	require("code"):seek(job)
end

return M
