use crate::core::action::ActionManager;
use crate::core::event::EventNexus;
use crate::types::event_type::message_event::PrivateMessageEvent;
use crate::types::message_type::Message;
use crate::types::plugin_type::{BasePlugin, PluginWrapper};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
pub struct HelpPlugin {
    plugins: Arc<RwLock<Vec<Arc<PluginWrapper>>>>,
}

impl HelpPlugin {
    pub fn new(plugins: Arc<RwLock<Vec<Arc<PluginWrapper>>>>) -> Self {
        Self { plugins }
    }

    async fn on_private_message(&self, msg: Arc<PrivateMessageEvent>, act: Arc<ActionManager>) {
        if msg.raw_message.starts_with("/help") {
            let mut info = String::from("[PluginList]\n");
            for plugin in self.plugins.read().await.iter() {
                info.push_str(&format!("{}\n\n", plugin.get_info_str()));
            }
            let info = info.trim().to_string();
            let _ = act
                .send_private_message(msg.sender.user_id, Message::new().with_text(info))
                .await;
        }
    }
}

#[async_trait]
impl BasePlugin for HelpPlugin {
    async fn on_load(self: Arc<Self>) {}
    async fn on_update(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) {
        let private_port = event_nexus.get_private_message_port();
        let Ok(private_message) = private_port.recv().await else {
            return;
        };
        self.on_private_message(private_message.clone(), act.clone())
            .await;
    }
    async fn on_unload(self: Arc<Self>) {}
}
