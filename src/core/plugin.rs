use crate::{
    config::Config,
    core::event::EventNexus,
    plugins::{ai_chat::AiChatPlugin, get_help::HelpPlugin},
    prelude::ActionManager,
    types::plugin_type::PluginWrapper,
};
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
        let deepseek_token = Config::get_or_init().ai_deepseek_token();
        let ai_chat_plugin = PluginWrapper::new(AiChatPlugin::new(deepseek_token))
            .with_name("Ai Chat In QQ")
            .with_description("Any Triggle");
        self.clone().add_plugin(help_plugin).await;
        self.clone().add_plugin(ai_chat_plugin).await;
        tokio::spawn(self.clone().handle_plugin());
    }
}
