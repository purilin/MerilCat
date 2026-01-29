use crate::{
    core::{adapter::NapcatAdapter, event::EventManager},
    plugin::PluginManager,
    prelude::ActionManager,
};
use std::sync::Arc;
pub struct MerilBot {
    pub adapter: Arc<NapcatAdapter>,
    pub event: Arc<EventManager>,
    pub action: Arc<ActionManager>,
    pub plugin: Arc<PluginManager>,
}

impl MerilBot {
    pub fn new() -> Self {
        tracing_subscriber::fmt::init();
        let adapter = NapcatAdapter::new();
        let event = EventManager::new(adapter.clone().get_event_port());
        let action = ActionManager::new(adapter.clone().get_action_port());
        let plugin = PluginManager::new(action.clone(), event.get_event_nexus());
        Self {
            event,
            adapter,
            action,
            plugin,
        }
    }

    pub async fn run(&self) {
        self.adapter.clone().run();
        self.event.clone().run();
        self.plugin.clone().run().await;
        let () = std::future::pending().await;
    }
}

impl Default for MerilBot {
    fn default() -> Self {
        Self::new()
    }
}
