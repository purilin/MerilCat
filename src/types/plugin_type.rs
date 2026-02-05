use std::sync::Arc;

use crate::{core::event::EventNexus, prelude::ActionManager};

#[async_trait::async_trait]
pub trait BasePlugin: Send + Sync {
    async fn on_load(self: Arc<Self>) -> ();
    async fn on_update(
        self: Arc<Self>,
        event_nexus: Arc<EventNexus>,
        act: Arc<ActionManager>,
    ) -> ();
    async fn on_unload(self: Arc<Self>) -> ();
}

pub struct PluginWrapper {
    name: String,
    description: String,
    version: String,
    author: String,
    inner: Arc<dyn BasePlugin>,
}

impl PluginWrapper {
    pub fn new<T>(plugin: T) -> Self
    where
        T: BasePlugin + Sync + Send + 'static,
    {
        PluginWrapper {
            name: "None".to_string(),
            description: "None".to_string(),
            version: "0.0.0".to_string(),
            author: "None".to_string(),
            inner: Arc::new(plugin),
        }
    }

    pub fn get_info_str(&self) -> String {
        format!("->[{}]\n-->{}", self.name, self.description)
    }

    pub async fn on_plugin_load(&self) {
        self.inner.clone().on_load().await;
        tracing::info!("[插件已加载 name={}]", self.name);
    }

    pub fn with_name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_description(mut self, description: impl ToString) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_version(mut self, version: impl ToString) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn with_author(mut self, author: impl ToString) -> Self {
        self.author = author.to_string();
        self
    }

    pub async fn run(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) {
        self.inner.clone().on_load().await;
        loop {
            self.inner
                .clone()
                .on_update(event_nexus.clone(), act.clone())
                .await;
        }
        //self.inner.clone().on_unload().await;
    }
}
