use crate::{
    core::event::EventNexus,
    prelude::{ActionManager, Message, PrivateMessageEvent},
    types::plugin_type::{BasePlugin, PluginWrapper},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PluginManager {
    plugins: Arc<RwLock<Vec<Arc<PluginWrapper>>>>,
    act: Arc<ActionManager>,
    event_nexus: Arc<EventNexus>,
}

impl PluginManager {
    pub fn new(act: Arc<ActionManager>, event_nexus: Arc<EventNexus>) -> Arc<Self> {
        Arc::new(PluginManager {
            plugins: Arc::new(RwLock::new(Vec::new())),
            act,
            event_nexus,
        })
    }

    async fn handle_plugin(self: Arc<Self>) {
        tracing::info!(
            "[插件加载] [数量: {}] 加载中...",
            self.plugins.read().await.len()
        );
        let plugins = self.plugins.read().await.clone();
        for plugin in plugins {
            let plugin = plugin.clone();
            plugin.clone().on_plugin_load().await;
            tokio::spawn(plugin.run(self.event_nexus.clone(), self.act.clone()));
        }
    }

    pub async fn add_plugin(self: Arc<Self>, plugin: PluginWrapper) {
        let arc_self = self.clone();
        arc_self.plugins.write().await.push(Arc::new(plugin));
    }

    pub async fn run(self: Arc<Self>) {
        let help_plugin = PluginWrapper::new(HelpPlugin::new(self.plugins.clone()))
            .with_name("GetHelpList")
            .with_description("/help");
        self.clone().add_plugin(help_plugin).await;
        tokio::spawn(self.clone().handle_plugin());
    }
}

struct HelpPlugin {
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
            act.send_private_message(msg.sender.user_id, Message::new().with_text(info))
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
