use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use crate::{
    core::event::EventManager,
    prelude::{Message, NapcatApi},
    utils::parser::event_parser::message_event::{GroupMessageEvent, PrivateMessageEvent},
};
pub struct PluginManager {
    plugins: Arc<RwLock<Vec<Plugin>>>,
}

#[derive(Clone)]
pub struct Plugin {
    name: String,
    introduction: String,
    command: String,
}

static INSTANCE: OnceLock<PluginManager> = OnceLock::new();
impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn init() -> &'static Self {
        INSTANCE.get_or_init(|| Self::new())
    }

    pub async fn reg_init() {
        Plugin::new("GetHelp")
            .with_command("/help")
            .with_introduction("Get PluginList")
            .reg_private_plugin(Self::plugin_help)
            .await;
    }

    pub fn get() -> &'static Self {
        INSTANCE.get().unwrap()
    }

    async fn plugin_help(msg: PrivateMessageEvent) {
        let mut res_str = String::new();
        let plugins = PluginManager::get().plugins.clone();
        for plugin in plugins.read().await.iter() {
            let cur_str = format!("{}\n", plugin.to_text());
            res_str.push_str(&cur_str);
        }
        let message = Message::new().with_text(res_str);
        NapcatApi::get()
            .send_private_message(msg.sender.user_id, message)
            .await;
    }

    pub async fn add_plugin(&self, plugin: Plugin) {
        println!("Registing plugin:{}", plugin.name);
        let mut mng = self.plugins.write().await;
        println!("Registed plugin:{}", plugin.name);
        mng.push(plugin);
    }
}

impl Plugin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            introduction: "I am Lazy!".into(),
            command: "None".into(),
        }
    }

    pub fn to_text(&self) -> String {
        format!(
            "[{}]\n> {}\n> {}",
            self.name, self.command, self.introduction
        )
        .trim()
        .to_string()
    }

    /// command == "Any": *
    /// command == "None": None
    /// command == "/help": start_with("/help")
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = command.into();
        self
    }

    pub fn with_introduction(mut self, inttroduction: impl Into<String>) -> Self {
        self.introduction = inttroduction.into();
        self
    }

    pub async fn reg_private_plugin<F, Fut>(self, func: F)
    where
        F: Fn(crate::utils::parser::event_parser::message_event::PrivateMessageEvent) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let arc_func = Arc::new(func);
        let event = EventManager::get();
        let command_str = self.command.clone();
        event
            .reg_private_event(move |msg| {
                let arc_func_c = arc_func.clone();
                let command_str_c = command_str.clone();
                async move {
                    if command_str_c == "Any" {
                        arc_func_c(msg).await;
                    } else if command_str_c == "None" {
                    } else if msg.raw_message.starts_with(&command_str_c) {
                        arc_func_c(msg).await;
                    }
                }
            })
            .await;
        PluginManager::get().add_plugin(self).await;
    }

    pub async fn reg_group_plugin<F, Fut>(self, func: F)
    where
        F: Fn(crate::utils::parser::event_parser::message_event::GroupMessageEvent) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let arc_func = Arc::new(func);
        let event = EventManager::get();
        let command_str = self.command.clone();
        event
            .reg_group_event(move |msg| {
                let arc_func_c = arc_func.clone();
                let command_str_c = command_str.clone();
                async move {
                    if command_str_c == "Any" {
                        arc_func_c(msg).await;
                    } else if command_str_c == "None" {
                    } else if msg.raw_message.starts_with(&command_str_c) {
                        arc_func_c(msg).await;
                    }
                }
            })
            .await;
        PluginManager::get().add_plugin(self).await;
    }
}
