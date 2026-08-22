local M = {}

local cache_home = os.getenv("XDG_CACHE_HOME") or (os.getenv("HOME") .. "/.cache")
local state_dir = cache_home .. "/aibox-yazi-preview-options"
local number_flag = state_dir .. "/line-numbers"
local wrap_flag = state_dir .. "/line-wrap"

function M:options()
	return {
		numbers = fs.cha(Url(number_flag)) ~= nil,
		wrap = fs.cha(Url(wrap_flag)) ~= nil,
	}
end

local function toggle(path)
	if fs.cha(Url(path)) then
		local ok, err = fs.remove("file", Url(path))
		return ok and false or nil, err
	end
	local ok, err = fs.create("dir_all", Url(state_dir))
	if not ok then return nil, err end
	ok, err = fs.write(Url(path), "")
	return ok and true or nil, err
end

function M:entry(job)
	job = type(job) == "string" and { args = { job } } or job
	local action = job.args[1]
	local label, enabled, err
	if action == "toggle-numbers" then
		label = "Line numbers"
		enabled, err = toggle(number_flag)
	elseif action == "toggle-wrap" then
		label = "Line wrapping"
		enabled, err = toggle(wrap_flag)
	else
		return
	end
	if enabled == nil then
		ya.notify({ title = "Preview", content = tostring(err), level = "error", timeout = 5 })
		return
	end
	ya.notify({ title = "Preview", content = label .. (enabled and " on" or " off"), timeout = 2 })
	local hovered = cx.active.current.hovered
	if hovered then ya.emit("peek", { 0, only_if = hovered.url }) end
end

function M:peek(job)
	local opts = M:options()
	local child = Command("bat")
		:arg({ "--color=always", "--paging=never", "--style=" .. (opts.numbers and "numbers" or "plain"),
			"--wrap=never", "--", tostring(job.file.url) })
		:stdout(Command.PIPED):stderr(Command.PIPED):spawn()
	if not child then return require("code"):peek(job) end

	local limit, consumed, lines = job.area.h, 0, {}
	repeat
		local next, event = child:read_line()
		if event == 1 then return require("code"):peek(job) end
		if event ~= 0 then break end
		next = next:gsub("\r?\n$", "")
		consumed = consumed + 1
		if consumed > job.skip then lines[#lines + 1] = next end
	until #lines >= limit

	child:start_kill()
	if job.skip > 0 and consumed < job.skip + limit then
		ya.emit("peek", { math.max(0, consumed - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text.parse(table.concat(lines, "\n"))
			:area(job.area):wrap(opts.wrap and ui.Wrap.YES or ui.Wrap.NO))
	end
end

function M:seek(job) require("code"):seek(job) end

return M
