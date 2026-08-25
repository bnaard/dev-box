--- @since 25.5.31
--- @sync entry

local function entry(st, job)
	local R = rt.mgr.ratio
	local base = { parent = R[1], current = R[2], preview = R[3] }
	job = type(job) == "string" and { args = { job } } or job

	st.parent = st.parent or base.parent
	st.current = st.current or base.current
	st.preview = st.preview or base.preview

	local act, to = string.match(job.args[1] or "", "(.-)-(.+)")
	if act == "min" then
		st[to] = st[to] == base[to] and 0 or base[to]
	elseif act == "max" then
		local max = st[to] == 65535 and base[to] or 65535
		st.parent = st.parent == 65535 and base.parent or st.parent
		st.current = st.current == 65535 and base.current or st.current
		st.preview = st.preview == 65535 and base.preview or st.preview
		st[to] = max
	end

	if not st.old then
		st.old = Tab.layout
		Tab.layout = function(self)
			local all = st.parent + st.current + st.preview
			self._chunks = ui.Layout()
				:direction(ui.Layout.HORIZONTAL)
				:constraints({
					ui.Constraint.Ratio(st.parent, all),
					ui.Constraint.Ratio(st.current, all),
					ui.Constraint.Ratio(st.preview, all),
				})
				:split(self._area)
		end
	end

	if not act then
		Tab.layout, st.old = st.old, nil
		st.parent, st.current, st.preview = nil, nil, nil
	end
	ya.emit("app:resize", {})
end

return { entry = entry }
