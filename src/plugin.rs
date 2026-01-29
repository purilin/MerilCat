use crate::{
    core::event::EventNexus,
    prelude::{ActionManager, Message},
    types::plugin_type::{BasePlugin, Plugin, Trigger},
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PluginManager {
    plugins: RwLock<Vec<Arc<Plugin>>>,
    act: Arc<ActionManager>,
    event_nexus: Arc<EventNexus>,
}

impl PluginManager {
    pub fn new(act: Arc<ActionManager>, event_nexus: Arc<EventNexus>) -> Arc<Self> {
        Arc::new(PluginManager {
            plugins: RwLock::new(Vec::new()),
            act,
            event_nexus,
        })
    }

    async fn handle_plugin(self: Arc<Self>) {
        tracing::info!(
            "[插件加载] [数量: {}] 加载中...",
            self.plugins.read().await.len()
        );
        let plugins_arc = self.plugins.read().await.clone();
        for plugin in plugins_arc {
            let arc_plugin = plugin.clone();
            let event_nexus = self.event_nexus.clone();
            let act = self.act.clone();
            tokio::spawn(async move {
                arc_plugin
                    .clone()
                    .on_load(event_nexus.clone(), act.clone())
                    .await;
                arc_plugin
                    .clone()
                    .on_update(event_nexus.clone(), act.clone())
                    .await;
            });
        }
    }

    pub async fn add_plugin(self: Arc<Self>, plugin: Plugin) {
        let arc_self = self.clone();
        arc_self.plugins.write().await.push(Arc::new(plugin));
    }

    async fn get_plugin_info(self: Arc<Self>) -> String {
        let plugins = self.clone().plugins.read().await.clone();

        let mut help_str = String::from(">插件列表\n");
        for plugin in plugins {
            help_str.push_str(&format!("{}\n\n", plugin.get_info_str()));
        }
        help_str = help_str.trim().to_string();
        help_str.push_str("\n--->--->");
        help_str
    }

    pub async fn run(self: Arc<Self>) {
        let self_for_help_plugin = self.clone();
        let help_plugin = Plugin::new()
            .with_name("Get Help")
            .with_author("purilin")
            .with_description("/help")
            .with_trigger(Trigger::StartWith("/help".to_string()))
            .with_on_private_message_func(move |msg, act| {
                let cloned_self = self_for_help_plugin.clone();
                async move {
                    let help_info_cloned = cloned_self.clone().get_plugin_info().await;
                    act.send_private_message(
                        msg.sender.user_id,
                        Message::new().with_text(help_info_cloned),
                    )
                    .await;
                }
            });
        self.clone().add_plugin(help_plugin).await;
        tokio::spawn(self.clone().handle_plugin());
    }
}
