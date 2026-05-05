const SEGMENT_SEPARATOR: &str = " ▸ ";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshot {
    pub memory_current: String,
    pub memory_limit: String,
    pub oom_kill: String,
    pub memory_high: String,
    pub memory_max_events: String,
    pub cpu_throttle: String,
    pub load_average: String,
    pub process_count: String,
    pub ai_agent_count: String,
    pub processkit_mode: String,
    pub processkit_mcp_count: String,
    pub disk_available: String,
    pub uptime: String,
    pub git_branch: String,
    pub git_state: String,
    pub migrations: String,
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self {
            memory_current: "...".to_string(),
            memory_limit: "...".to_string(),
            oom_kill: "...".to_string(),
            memory_high: "...".to_string(),
            memory_max_events: "...".to_string(),
            cpu_throttle: "...".to_string(),
            load_average: "...".to_string(),
            process_count: "...".to_string(),
            ai_agent_count: "...".to_string(),
            processkit_mode: "...".to_string(),
            processkit_mcp_count: "...".to_string(),
            disk_available: "...".to_string(),
            uptime: "...".to_string(),
            git_branch: "...".to_string(),
            git_state: "...".to_string(),
            migrations: "...".to_string(),
        }
    }
}

impl RuntimeSnapshot {
    pub fn from_aibox_status_plain(line: &str) -> Self {
        let mut snapshot = Self::default();

        for section in line.split('|').map(str::trim) {
            if let Some(memory) = section.strip_prefix("MEM ") {
                parse_memory(memory, &mut snapshot);
            } else if let Some(cpu) = section.strip_prefix("CPU ") {
                parse_cpu(cpu, &mut snapshot);
            } else if let Some(processes) = section.strip_prefix("PROC ") {
                parse_processes(processes, &mut snapshot);
            } else if let Some(fs) = section.strip_prefix("FS ") {
                snapshot.disk_available = fs.trim().to_string();
            } else if let Some(up) = section.strip_prefix("UP ") {
                snapshot.uptime = up.trim().to_string();
            } else if let Some(project) = section.strip_prefix("PROJ ") {
                parse_project(project, &mut snapshot);
            }
        }

        snapshot
    }

    pub fn from_aibox_status_json(line: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        Some(Self {
            memory_current: json_field(&value, "memory_current"),
            memory_limit: json_field(&value, "memory_max"),
            oom_kill: json_field(&value, "oom_kill"),
            memory_high: json_field(&value, "memory_high"),
            memory_max_events: json_field(&value, "memory_max_events"),
            cpu_throttle: json_field(&value, "cpu_throttling"),
            load_average: json_field(&value, "load_average"),
            process_count: json_field(&value, "processes"),
            ai_agent_count: json_field(&value, "ai_agents"),
            processkit_mode: json_field(&value, "processkit_mode"),
            processkit_mcp_count: json_field(&value, "processkit_mcp"),
            disk_available: json_field(&value, "disk_available"),
            uptime: json_field(&value, "container_uptime"),
            git_branch: json_field(&value, "git_branch"),
            git_state: json_field(&value, "git_state"),
            migrations: json_field(&value, "migrations"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
    pub mode: String,
    pub active_keys: Vec<String>,
    pub snapshot: RuntimeSnapshot,
    pub show_key_hints: bool,
    pub show_runtime_status: bool,
    pub message: Option<String>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            mode: "Normal".to_string(),
            active_keys: Vec::new(),
            snapshot: RuntimeSnapshot::default(),
            show_key_hints: true,
            show_runtime_status: true,
            message: None,
        }
    }
}

pub fn render_rows(state: &RenderState, cols: usize) -> Vec<String> {
    let mut rows = Vec::with_capacity(2);
    if state.show_key_hints {
        let row = render_key_hints(state, cols);
        if !row.is_empty() {
            rows.push(row);
        }
    }
    if state.show_runtime_status {
        let row = render_runtime_status(state, cols);
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

pub fn render_key_hints(state: &RenderState, cols: usize) -> String {
    render_segments(&key_hint_segments(state), cols)
}

pub fn render_runtime_status(state: &RenderState, cols: usize) -> String {
    let snapshot = &state.snapshot;
    let mut segments = vec![
        Segment::new(mode_label(&state.mode), "", 0),
        Segment::new(
            "MEM",
            format!(
                "{}/{} OOM kills {}",
                compact_units(&snapshot.memory_current),
                compact_units(&snapshot.memory_limit),
                snapshot.oom_kill
            ),
            0,
        ),
        Segment::new("CPU", format!("throttle {}", snapshot.cpu_throttle), 2),
        Segment::new("LOAD", snapshot.load_average.clone(), 3),
        Segment::new(
            "PROC",
            format!(
                "total {} AI {}",
                snapshot.process_count, snapshot.ai_agent_count
            ),
            1,
        ),
        Segment::new(
            "MCP",
            format!(
                "{} {}",
                snapshot.processkit_mode, snapshot.processkit_mcp_count
            ),
            1,
        ),
        Segment::new(
            "GIT",
            format!("{} {}", snapshot.git_branch, snapshot.git_state),
            2,
        ),
        Segment::new("MIG", format!("open {}", snapshot.migrations), 1),
        Segment::new("FS", format!("free {}", snapshot.disk_available), 3),
        Segment::new("UP", snapshot.uptime.clone(), 4),
    ];

    if snapshot.memory_high != "0" || snapshot.memory_max_events != "0" {
        segments.insert(
            2,
            Segment::new(
                "MEM events",
                format!(
                    "high {} max {}",
                    snapshot.memory_high, snapshot.memory_max_events
                ),
                2,
            ),
        );
    }

    if let Some(message) = state.message.as_deref() {
        segments.push(Segment::new("MSG", message, 4));
    }

    render_segments(&segments, cols)
}

fn key_hint_segments(state: &RenderState) -> Vec<Segment> {
    let mode = state.mode.to_ascii_lowercase();
    if mode.contains("tmux") {
        return vec![
            Segment::new("LEADER", "C-g/Esc exit", 0),
            Segment::new("PANES", "h left j down k up l right", 0),
            Segment::new("SPLIT", "n new d down r right x close", 1),
            Segment::new("VIEW", "f full z frames e float", 3),
            Segment::new("TABS", "t new w close [/] prev/next 1-5 jump", 2),
            Segment::new("STATUS", "v runtime b keys", 1),
            Segment::new("TOOLS", "s files m sessions p float", 3),
            Segment::new("QUIT", "q", 4),
        ];
    }

    if mode.contains("scroll") || mode.contains("search") {
        return vec![
            Segment::new(mode_label(&state.mode), "scrollback", 0),
            Segment::new("MOVE", "j down k up d half-down u half-up", 0),
            Segment::new("PAGE", "f page-down b page-up g top G bottom", 1),
            Segment::new("SEARCH", "/ find n next N prev", 1),
            Segment::new("EXIT", "Esc C-c C-g", 0),
        ];
    }

    if mode.contains("resize") {
        return vec![
            Segment::new("RESIZE", "+/- grow/shrink", 0),
            Segment::new("DIRECTION", "h left j down k up l right", 0),
            Segment::new("REVERSE", "H/J/K/L shrink side", 1),
            Segment::new("EXIT", "Esc C-g", 0),
        ];
    }

    let mut segments = vec![Segment::new(mode_label(&state.mode), "", 0)];
    if key_available(state, "Ctrl g") {
        segments.push(Segment::new("C-g", "leader", 0));
    }
    if any_key_available(state, &["Alt h", "Alt j", "Alt k", "Alt l"]) {
        segments.push(Segment::new("PANES", "Alt-h/j/k/l move", 0));
    }
    if any_key_available(state, &["Alt [", "Alt ]", "Alt 1"]) {
        segments.push(Segment::new("TABS", "Alt-[/] prev/next Alt-1..5 jump", 1));
    }
    if key_available(state, "Alt p") {
        segments.push(Segment::new("FLOAT", "Alt-p toggle", 2));
    }
    if segments.len() == 1 {
        segments.extend([
            Segment::new("C-g", "leader", 0),
            Segment::new("PANES", "Alt-h/j/k/l move", 0),
            Segment::new("TABS", "Alt-[/] prev/next Alt-1..5 jump", 1),
            Segment::new("FLOAT", "Alt-p toggle", 2),
        ]);
    }
    segments
}

fn key_available(state: &RenderState, key: &str) -> bool {
    state.active_keys.is_empty()
        || state
            .active_keys
            .iter()
            .any(|active| active.eq_ignore_ascii_case(key))
}

fn any_key_available(state: &RenderState, keys: &[&str]) -> bool {
    keys.iter().any(|key| key_available(state, key))
}

fn mode_label(mode: &str) -> String {
    let mode = mode.to_ascii_lowercase();
    if mode.contains("tmux") {
        "LEADER".to_string()
    } else if mode.contains("entersearch") {
        "SEARCH".to_string()
    } else if mode.contains("scroll") {
        "SCROLL".to_string()
    } else if mode.contains("resize") {
        "RESIZE".to_string()
    } else if mode.contains("search") {
        "SEARCH".to_string()
    } else if mode.contains("locked") {
        "LOCKED".to_string()
    } else {
        "NORMAL".to_string()
    }
}

fn parse_memory(memory: &str, snapshot: &mut RuntimeSnapshot) {
    let Some((usage, rest)) = memory.split_once(" oom") else {
        return;
    };

    if let Some((current, limit)) = usage.trim().split_once('/') {
        snapshot.memory_current = current.trim().to_string();
        snapshot.memory_limit = limit.trim().to_string();
    }

    for token in rest.split_whitespace() {
        if let Some(value) = token.strip_prefix("hi") {
            snapshot.memory_high = value.to_string();
        } else if let Some(value) = token.strip_prefix("max") {
            snapshot.memory_max_events = value.to_string();
        } else if !token.is_empty() && snapshot.oom_kill == "..." {
            snapshot.oom_kill = token.to_string();
        }
    }
}

fn parse_cpu(cpu: &str, snapshot: &mut RuntimeSnapshot) {
    for token in cpu.split_whitespace() {
        if let Some(value) = token.strip_prefix("thr") {
            snapshot.cpu_throttle = value.to_string();
        } else if let Some(value) = token.strip_prefix("load") {
            snapshot.load_average = value.to_string();
        } else if snapshot.cpu_throttle == "..." {
            snapshot.cpu_throttle = token.to_string();
        }
    }
}

fn json_field(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("...")
        .to_string()
}

fn parse_processes(processes: &str, snapshot: &mut RuntimeSnapshot) {
    for token in processes.split_whitespace() {
        if let Some(value) = token.strip_prefix("ai") {
            snapshot.ai_agent_count = value.to_string();
        } else if let Some(value) = token.strip_prefix("pk:") {
            if let Some((mode, count)) = value.split_once('/') {
                snapshot.processkit_mode = mode.to_string();
                snapshot.processkit_mcp_count = count.to_string();
            }
        } else if snapshot.process_count == "..." {
            snapshot.process_count = token.to_string();
        }
    }
}

fn parse_project(project: &str, snapshot: &mut RuntimeSnapshot) {
    for token in project.split_whitespace() {
        if let Some(value) = token.strip_prefix("git:") {
            if let Some((branch, state)) = value.rsplit_once(':') {
                snapshot.git_branch = branch.to_string();
                snapshot.git_state = state.to_string();
            } else {
                snapshot.git_state = value.to_string();
            }
        } else if let Some(value) = token.strip_prefix("mig") {
            snapshot.migrations = value.to_string();
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Segment {
    label: String,
    value: String,
    priority: u8,
}

impl Segment {
    fn new(label: impl Into<String>, value: impl Into<String>, priority: u8) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            priority,
        }
    }

    fn text(&self) -> String {
        if self.value.is_empty() {
            format!("[ {} ]", self.label)
        } else {
            format!("[ {} {} ]", self.label, self.value)
        }
    }
}

fn render_segments(segments: &[Segment], cols: usize) -> String {
    if cols == 0 {
        return String::new();
    }

    let mut visible: Vec<&Segment> = segments.iter().collect();
    while !visible.is_empty() {
        let line = visible
            .iter()
            .map(|segment| segment.text())
            .collect::<Vec<_>>()
            .join(SEGMENT_SEPARATOR);
        if char_count(&line) <= cols {
            return line;
        }

        let remove_at = visible
            .iter()
            .enumerate()
            .max_by_key(|(_, segment)| segment.priority)
            .map(|(index, _)| index);
        if let Some(index) = remove_at {
            visible.remove(index);
        } else {
            break;
        }
    }

    truncate_to_width(&segments[0].text(), cols)
}

fn compact_units(value: &str) -> String {
    value
        .replace(" KiB", "K")
        .replace(" MiB", "M")
        .replace(" GiB", "G")
        .replace(" TiB", "T")
}

fn truncate_to_width(text: &str, cols: usize) -> String {
    if char_count(text) <= cols {
        return text.to_string();
    }
    if cols <= 1 {
        return "~".chars().take(cols).collect();
    }

    let mut output: String = text.chars().take(cols - 1).collect();
    output.push('~');
    output
}

fn char_count(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_shell_status_line() {
        let snapshot = RuntimeSnapshot::from_aibox_status_plain(
            "MEM 512.0 MiB/8.0 GiB oom0 hi1 max2 | CPU thr3/12s load0.42 | PROC 42 ai2 pk:gateway/1 | FS 28G | UP 1h5m | PROJ git:main:dirty mig4",
        );

        assert_eq!(snapshot.memory_current, "512.0 MiB");
        assert_eq!(snapshot.memory_limit, "8.0 GiB");
        assert_eq!(snapshot.oom_kill, "0");
        assert_eq!(snapshot.memory_high, "1");
        assert_eq!(snapshot.memory_max_events, "2");
        assert_eq!(snapshot.cpu_throttle, "3/12s");
        assert_eq!(snapshot.load_average, "0.42");
        assert_eq!(snapshot.process_count, "42");
        assert_eq!(snapshot.ai_agent_count, "2");
        assert_eq!(snapshot.processkit_mode, "gateway");
        assert_eq!(snapshot.processkit_mcp_count, "1");
        assert_eq!(snapshot.disk_available, "28G");
        assert_eq!(snapshot.uptime, "1h5m");
        assert_eq!(snapshot.git_branch, "main");
        assert_eq!(snapshot.git_state, "dirty");
        assert_eq!(snapshot.migrations, "4");
    }

    #[test]
    fn parses_plugin_json_status_line() {
        let snapshot = RuntimeSnapshot::from_aibox_status_json(
            r#"{"memory_current":"512.0 MiB","memory_max":"8.0 GiB","oom_kill":"0","memory_high":"1","memory_max_events":"2","cpu_throttling":"3/12s","load_average":"0.42","processes":"42","ai_agents":"2","processkit_mode":"gateway","processkit_mcp":"1","disk_available":"28G","container_uptime":"1h5m","git_branch":"main","git_state":"dirty","migrations":"4"}"#,
        )
        .expect("valid plugin JSON should parse");

        assert_eq!(snapshot.memory_current, "512.0 MiB");
        assert_eq!(snapshot.memory_limit, "8.0 GiB");
        assert_eq!(snapshot.load_average, "0.42");
        assert_eq!(snapshot.processkit_mode, "gateway");
        assert_eq!(snapshot.git_branch, "main");
        assert_eq!(snapshot.migrations, "4");
    }

    #[test]
    fn runtime_status_uses_readable_labels() {
        let state = RenderState {
            snapshot: RuntimeSnapshot::from_aibox_status_plain(
                "MEM 512.0 MiB/8.0 GiB oom0 hi0 max0 | CPU thr0/0s load0.18 | PROC 42 ai2 pk:gateway/1 | FS 28G | UP 1h5m | PROJ git:main:dirty mig0",
            ),
            ..RenderState::default()
        };

        let line = render_runtime_status(&state, 160);
        assert!(line.contains("[ NORMAL ]"));
        assert!(line.contains("OOM kills 0"), "{line}");
        assert!(line.contains("AI 2"), "{line}");
        assert!(!line.contains("oom0"), "{line}");
        assert!(!line.contains("ai2"), "{line}");
    }

    #[test]
    fn runtime_status_respects_width() {
        let state = RenderState {
            snapshot: RuntimeSnapshot::from_aibox_status_plain(
                "MEM 512.0 MiB/8.0 GiB oom0 hi0 max0 | CPU thr0/0s load0.18 | PROC 42 ai2 pk:gateway/1 | FS 28G | UP 1h5m | PROJ git:main:dirty mig0",
            ),
            ..RenderState::default()
        };

        let line = render_runtime_status(&state, 48);
        assert!(
            line.chars().count() <= 48,
            "line should fit within requested width: {line}"
        );
        assert!(
            line.contains("MEM"),
            "highest-priority memory group should remain visible: {line}"
        );
    }

    #[test]
    fn leader_keybar_explains_actions() {
        let state = RenderState {
            mode: "Tmux".to_string(),
            ..RenderState::default()
        };

        let line = render_key_hints(&state, 180);
        assert!(line.contains("[ LEADER C-g/Esc exit ]"), "{line}");
        assert!(line.contains("h left j down k up l right"), "{line}");
        assert!(line.contains("n new d down r right x close"), "{line}");
        assert!(line.contains("v runtime b keys"), "{line}");
    }

    #[test]
    fn normal_keybar_uses_arrow_segments_and_filters_known_keys() {
        let state = RenderState {
            active_keys: vec![
                "Ctrl g".to_string(),
                "Alt h".to_string(),
                "Alt ]".to_string(),
            ],
            ..RenderState::default()
        };

        let line = render_key_hints(&state, 120);
        assert!(line.contains("▸"), "{line}");
        assert!(line.contains("[ C-g leader ]"), "{line}");
        assert!(line.contains("[ PANES Alt-h/j/k/l move ]"), "{line}");
        assert!(
            line.contains("[ TABS Alt-[/] prev/next Alt-1..5 jump ]"),
            "{line}"
        );
        assert!(!line.contains("FLOAT"), "{line}");
    }

    #[test]
    fn render_rows_can_hide_either_surface() {
        let state = RenderState {
            show_key_hints: false,
            show_runtime_status: true,
            ..RenderState::default()
        };

        let rows = render_rows(&state, 80);
        assert_eq!(rows.len(), 1, "hidden key row should not allocate a row");
        assert!(
            rows[0].starts_with("[ NORMAL ]"),
            "remaining row should be runtime status"
        );
    }

    #[test]
    fn default_native_rows_are_visible_at_normal_width() {
        let rows = render_rows(&RenderState::default(), 100);

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| !row.trim().is_empty()));
        assert!(rows[0].contains("C-g"));
        assert!(rows[1].contains("MEM"));
    }

    #[test]
    fn hidden_or_zero_width_rows_do_not_allocate_blank_lines() {
        let hidden = RenderState {
            show_key_hints: false,
            show_runtime_status: false,
            ..RenderState::default()
        };

        assert!(render_rows(&hidden, 80).is_empty());
        assert!(render_rows(&RenderState::default(), 0).is_empty());
    }
}
