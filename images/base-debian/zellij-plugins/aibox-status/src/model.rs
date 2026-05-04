const GROUP_SEPARATOR: &str = "  ";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSnapshot {
    pub memory_current: String,
    pub memory_limit: String,
    pub oom_kill: String,
    pub memory_high: String,
    pub memory_max_events: String,
    pub cpu_throttle: String,
    pub process_count: String,
    pub ai_agent_count: String,
    pub processkit_mode: String,
    pub processkit_mcp_count: String,
    pub disk_available: String,
    pub uptime: String,
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
            process_count: "...".to_string(),
            ai_agent_count: "...".to_string(),
            processkit_mode: "...".to_string(),
            processkit_mcp_count: "...".to_string(),
            disk_available: "...".to_string(),
            uptime: "...".to_string(),
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
                snapshot.cpu_throttle = cpu.strip_prefix("thr").unwrap_or(cpu).trim().to_string();
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
            process_count: json_field(&value, "processes"),
            ai_agent_count: json_field(&value, "ai_agents"),
            processkit_mode: json_field(&value, "processkit_mode"),
            processkit_mcp_count: json_field(&value, "processkit_mcp"),
            disk_available: json_field(&value, "disk_available"),
            uptime: json_field(&value, "container_uptime"),
            git_state: json_field(&value, "git_state"),
            migrations: json_field(&value, "migrations"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
    pub mode: String,
    pub snapshot: RuntimeSnapshot,
    pub show_key_hints: bool,
    pub show_runtime_status: bool,
    pub message: Option<String>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            mode: "Normal".to_string(),
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
        let row = render_key_hints(&state.mode, cols);
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

pub fn render_key_hints(mode: &str, cols: usize) -> String {
    let mode_key = mode.to_ascii_lowercase();
    let groups = if mode_key.contains("tmux") {
        vec![
            Segment::new("LEADER C-g/Esc", 0),
            Segment::new("PANES h j k l  n d r x f z e", 1),
            Segment::new("TABS t w [ ] 1-5 i o", 1),
            Segment::new("TOOLS s m p", 2),
            Segment::new("SCROLL u /", 2),
            Segment::new("QUIT q", 3),
        ]
    } else if mode_key.contains("scroll") || mode_key.contains("search") {
        vec![
            Segment::new("SCROLL j/k d/u f/b g/G", 0),
            Segment::new("SEARCH /", 1),
            Segment::new("EXIT Esc C-c C-g", 1),
        ]
    } else if mode_key.contains("resize") {
        vec![
            Segment::new("RESIZE +/- arrows", 0),
            Segment::new("EXIT Esc C-g", 1),
        ]
    } else {
        vec![
            Segment::new("C-g leader", 0),
            Segment::new("Alt-h/j/k/l panes", 1),
            Segment::new("Alt-[ ] tabs", 1),
            Segment::new("Alt-1..5 jump", 2),
            Segment::new("Alt-p float", 2),
            Segment::new("C-q quit", 3),
        ]
    };

    fit_segments(&groups, cols)
}

pub fn render_runtime_status(state: &RenderState, cols: usize) -> String {
    let snapshot = &state.snapshot;
    let mut groups = vec![
        Segment::new(
            format!(
                "MEM {}/{} oom{} hi{} max{}",
                snapshot.memory_current,
                snapshot.memory_limit,
                snapshot.oom_kill,
                snapshot.memory_high,
                snapshot.memory_max_events
            ),
            0,
        ),
        Segment::new(format!("CPU thr{}", snapshot.cpu_throttle), 1),
        Segment::new(
            format!(
                "PROC {} ai{} pk:{}/{}",
                snapshot.process_count,
                snapshot.ai_agent_count,
                snapshot.processkit_mode,
                snapshot.processkit_mcp_count
            ),
            1,
        ),
        Segment::new(format!("FS {}", snapshot.disk_available), 2),
        Segment::new(format!("UP {}", snapshot.uptime), 2),
        Segment::new(
            format!("PROJ git:{} mig{}", snapshot.git_state, snapshot.migrations),
            2,
        ),
    ];

    if let Some(message) = state.message.as_deref() {
        groups.push(Segment::new(format!("MSG {message}"), 3));
    }

    fit_segments(&groups, cols)
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
            snapshot.git_state = value.to_string();
        } else if let Some(value) = token.strip_prefix("mig") {
            snapshot.migrations = value.to_string();
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Segment {
    text: String,
    priority: u8,
}

impl Segment {
    fn new(text: impl Into<String>, priority: u8) -> Self {
        Self {
            text: text.into(),
            priority,
        }
    }
}

fn fit_segments(segments: &[Segment], cols: usize) -> String {
    if cols == 0 {
        return String::new();
    }

    let mut visible: Vec<&Segment> = segments.iter().collect();
    while !visible.is_empty() {
        let line = visible
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<Vec<_>>()
            .join(GROUP_SEPARATOR);
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

    truncate_to_width(&segments[0].text, cols)
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
            "MEM 512.0 MiB/8.0 GiB oom0 hi1 max2 | CPU thr3/12s | PROC 42 ai2 pk:gateway/1 | FS 28G | UP 1h5m | PROJ git:dirty mig4",
        );

        assert_eq!(snapshot.memory_current, "512.0 MiB");
        assert_eq!(snapshot.memory_limit, "8.0 GiB");
        assert_eq!(snapshot.oom_kill, "0");
        assert_eq!(snapshot.memory_high, "1");
        assert_eq!(snapshot.memory_max_events, "2");
        assert_eq!(snapshot.cpu_throttle, "3/12s");
        assert_eq!(snapshot.process_count, "42");
        assert_eq!(snapshot.ai_agent_count, "2");
        assert_eq!(snapshot.processkit_mode, "gateway");
        assert_eq!(snapshot.processkit_mcp_count, "1");
        assert_eq!(snapshot.disk_available, "28G");
        assert_eq!(snapshot.uptime, "1h5m");
        assert_eq!(snapshot.git_state, "dirty");
        assert_eq!(snapshot.migrations, "4");
    }

    #[test]
    fn parses_plugin_json_status_line() {
        let snapshot = RuntimeSnapshot::from_aibox_status_json(
            r#"{"memory_current":"512.0 MiB","memory_max":"8.0 GiB","oom_kill":"0","memory_high":"1","memory_max_events":"2","cpu_throttling":"3/12s","processes":"42","ai_agents":"2","processkit_mode":"gateway","processkit_mcp":"1","disk_available":"28G","container_uptime":"1h5m","git_state":"dirty","migrations":"4"}"#,
        )
        .expect("valid plugin JSON should parse");

        assert_eq!(snapshot.memory_current, "512.0 MiB");
        assert_eq!(snapshot.memory_limit, "8.0 GiB");
        assert_eq!(snapshot.processkit_mode, "gateway");
        assert_eq!(snapshot.migrations, "4");
    }

    #[test]
    fn runtime_status_respects_width() {
        let state = RenderState {
            snapshot: RuntimeSnapshot::from_aibox_status_plain(
                "MEM 512.0 MiB/8.0 GiB oom0 hi0 max0 | CPU thr0/0s | PROC 42 ai2 pk:gateway/1 | FS 28G | UP 1h5m | PROJ git:dirty mig0",
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
    fn render_rows_can_hide_either_surface() {
        let state = RenderState {
            show_key_hints: false,
            show_runtime_status: true,
            ..RenderState::default()
        };

        let rows = render_rows(&state, 80);
        assert_eq!(rows.len(), 1, "hidden key row should not allocate a row");
        assert!(
            rows[0].starts_with("MEM"),
            "remaining row should be runtime status"
        );
    }

    #[test]
    fn default_native_rows_are_visible_at_normal_width() {
        let rows = render_rows(&RenderState::default(), 80);

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
