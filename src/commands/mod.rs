use dashmap::DashMap;
use once_cell::sync::Lazy;
use crate::helpers::command_def::{CommandDef, InteractionHandler};

mod monitor;

pub const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: monitor::MONITOR,
        builder: monitor::monitor_builder,
        handler: |c, i| Box::pin(async move { monitor::monitor(c, i).await }),
        re_register: false,
        whitelisted_servers: None,
    }
];

static COMMAND_MAP: Lazy<DashMap<String, InteractionHandler>> = Lazy::new(|| {
    let map = DashMap::new();

    for cmd in COMMANDS {
        map.insert(cmd.name.to_string(), cmd.handler);
    }

    map
});

pub fn get_handler(command_name: &str) -> Option<InteractionHandler> {
    COMMAND_MAP
        .get(command_name)
        .as_ref()
        .map(|entry| *entry.value())
}