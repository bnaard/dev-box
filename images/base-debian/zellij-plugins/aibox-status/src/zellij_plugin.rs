use std::collections::BTreeMap;

use zellij_tile::prelude::*;
use zellij_tile::shim::plugin_api::action::ProtobufPluginConfiguration;
use zellij_tile::shim::plugin_api::event::ProtobufEvent;
use zellij_tile::shim::plugin_api::pipe_message::ProtobufPipeMessage;
use zellij_tile::shim::prost::Message;

use crate::model::{render_rows, RenderState, RuntimeSnapshot};

const REQUEST_KIND: &str = "aibox_status_request";
const REFRESH_SECONDS: f64 = 5.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum RowRole {
    Keybar,
    #[default]
    Status,
}

#[derive(Default)]
struct AiboxStatusPlugin {
    state: RenderState,
    request_id: u64,
    role: RowRole,
    hidden: bool,
}

impl AiboxStatusPlugin {
    fn refresh(&mut self) {
        if self.role != RowRole::Status || self.hidden {
            return;
        }
        self.request_id += 1;

        let mut context = BTreeMap::new();
        context.insert(REQUEST_KIND.to_string(), self.request_id.to_string());

        run_command(
            &[
                "sh",
                "-lc",
                "command -v aibox-status >/dev/null 2>&1 && aibox-status --plugin-json || /usr/local/bin/aibox-status --plugin-json",
            ],
            context,
        );
    }

    fn apply_configuration(&mut self, configuration: BTreeMap<String, String>) {
        if let Some(value) = configuration
            .get("role")
            .or_else(|| configuration.get("row"))
            .map(String::as_str)
        {
            self.role = match value {
                "keybar" | "keys" => RowRole::Keybar,
                "status" | "runtime" => RowRole::Status,
                _ => RowRole::Status,
            };
        }
        self.state.show_key_hints = self.role == RowRole::Keybar;
        self.state.show_runtime_status = self.role == RowRole::Status;
        if let Some(value) = configuration.get("show_key_hints") {
            self.state.show_key_hints = value != "false";
        }
        if let Some(value) = configuration.get("show_runtime_status") {
            self.state.show_runtime_status = value != "false";
        }
    }

    fn toggle_role(&mut self, role: RowRole) -> bool {
        if self.role != role {
            return false;
        }

        if self.hidden {
            self.hidden = false;
            show_self(false);
        } else {
            self.hidden = true;
            hide_self();
        }
        true
    }
}

impl ZellijPlugin for AiboxStatusPlugin {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.apply_configuration(configuration);
        if self.role == RowRole::Status {
            request_permission(&[
                PermissionType::RunCommands,
                PermissionType::ReadApplicationState,
            ]);
        } else {
            request_permission(&[PermissionType::ReadApplicationState]);
        }
        subscribe(&[
            EventType::ModeUpdate,
            EventType::Timer,
            EventType::RunCommandResult,
            EventType::Visible,
            EventType::PermissionRequestResult,
        ]);
        set_selectable(false);
        self.refresh();
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(mode_info) => {
                self.state.mode = format!("{:?}", mode_info.mode);
                true
            }
            Event::Timer(_) => {
                self.refresh();
                false
            }
            Event::RunCommandResult(exit_code, stdout, stderr, context)
                if context.contains_key(REQUEST_KIND) =>
            {
                if exit_code == Some(0) {
                    let line = String::from_utf8_lossy(&stdout);
                    self.state.snapshot = RuntimeSnapshot::from_aibox_status_json(line.trim())
                        .unwrap_or_else(|| RuntimeSnapshot::from_aibox_status_plain(line.trim()));
                    self.state.message = None;
                } else {
                    let stderr = String::from_utf8_lossy(&stderr);
                    self.state.message = Some(format!("status exit {:?}: {}", exit_code, stderr));
                }
                set_timeout(REFRESH_SECONDS);
                true
            }
            Event::PermissionRequestResult(_) => {
                self.refresh();
                false
            }
            Event::Visible(true) => {
                self.refresh();
                true
            }
            Event::Visible(false) => false,
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "aibox_toggle_keys" => self.toggle_role(RowRole::Keybar),
            "aibox_toggle_runtime" => self.toggle_role(RowRole::Status),
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        if self.hidden {
            return;
        }
        for (y, line) in render_rows(&self.state, cols)
            .into_iter()
            .take(rows)
            .enumerate()
        {
            print_text_with_coordinates(visible_row_text(line), 0, y, Some(cols), Some(1));
        }
    }
}

fn visible_row_text(line: String) -> Text {
    Text::new(line)
}

thread_local! {
    static STATE: std::cell::RefCell<AiboxStatusPlugin> = std::cell::RefCell::new(Default::default());
}

#[no_mangle]
pub extern "C" fn main() {
    std::panic::set_hook(Box::new(|info| {
        report_panic(info);
    }));
}

#[no_mangle]
pub extern "C" fn load() {
    STATE.with(|state| {
        let protobuf_bytes: Vec<u8> = object_from_stdin().unwrap();
        let protobuf_configuration =
            ProtobufPluginConfiguration::decode(protobuf_bytes.as_slice()).unwrap();
        let plugin_configuration = BTreeMap::try_from(&protobuf_configuration).unwrap();
        state.borrow_mut().load(plugin_configuration);
    });
}

#[no_mangle]
pub extern "C" fn update() -> bool {
    STATE.with(|state| {
        let protobuf_bytes: Vec<u8> = object_from_stdin().unwrap();
        let protobuf_event = ProtobufEvent::decode(protobuf_bytes.as_slice()).unwrap();
        let event = Event::try_from(protobuf_event).unwrap();
        state.borrow_mut().update(event)
    })
}

#[no_mangle]
pub extern "C" fn pipe() -> bool {
    STATE.with(|state| {
        let protobuf_bytes: Vec<u8> = object_from_stdin().unwrap();
        let protobuf_pipe_message = ProtobufPipeMessage::decode(protobuf_bytes.as_slice()).unwrap();
        let pipe_message = PipeMessage::try_from(protobuf_pipe_message).unwrap();
        state.borrow_mut().pipe(pipe_message)
    })
}

#[no_mangle]
pub extern "C" fn render(rows: i32, cols: i32) {
    STATE.with(|state| {
        state.borrow_mut().render(rows as usize, cols as usize);
    });
}

#[no_mangle]
pub extern "C" fn plugin_version() {
    println!("{}", VERSION);
}
