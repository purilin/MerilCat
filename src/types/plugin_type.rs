use std::sync::Arc;

use crate::{core::event::EventNexus, prelude::ActionManager};

use async_trait::async_trait;
use std::pin::Pin;

use crate::types::event_type::message_event::{GroupMessageEvent, PrivateMessageEvent};

#[async_trait::async_trait]
pub trait BasePlugin {
    async fn on_load(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) -> ();
    async fn on_update(
        self: Arc<Self>,
        event_nexus: Arc<EventNexus>,
        act: Arc<ActionManager>,
    ) -> ();
    async fn on_unload(
        self: Arc<Self>,
        event_nexus: Arc<EventNexus>,
        act: Arc<ActionManager>,
    ) -> ();
}

#[derive(Clone)]
pub enum Trigger {
    StartWith(String),
    Pattern(String),
    Always,
}

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync>>;

type UpdateHandler = dyn Fn(Arc<EventNexus>, Arc<ActionManager>) -> BoxedFuture + Send + Sync;
type GroupMsgHandler =
    dyn Fn(Arc<GroupMessageEvent>, Arc<ActionManager>) -> BoxedFuture + Send + Sync;
type PrivateMsgHandler =
    dyn Fn(Arc<PrivateMessageEvent>, Arc<ActionManager>) -> BoxedFuture + Send + Sync;

pub struct Plugin {
    name: String,
    description: String,
    version: String,
    author: String,
    trigger: Trigger,
    on_update_func: Arc<UpdateHandler>,
    on_group_message_func: Arc<GroupMsgHandler>,
    on_private_message_func: Arc<PrivateMsgHandler>,
}

impl Plugin {
    pub fn new() -> Self {
        Plugin {
            name: "None".to_string(),
            description: "None".to_string(),
            version: "0.0.0".to_string(),
            author: "None".to_string(),
            trigger: Trigger::Always,
            on_update_func: Arc::new(|_, _| Box::pin(async {})),
            on_group_message_func: Arc::new(|_, _| Box::pin(async {})),
            on_private_message_func: Arc::new(|_, _| Box::pin(async {})),
        }
    }

    pub fn get_info_str(&self) -> String {
        format!("->[{}]\n-->{}", self.name, self.description)
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

    pub fn with_trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn with_on_update_func<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Arc<EventNexus>, Arc<ActionManager>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        self.on_update_func = Arc::new(move |nexus, act| {
            let fut = func(nexus, act); // 调用原始函数，得到 Fut
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send + Sync>> // 包装成 dyn
        });
        self
    }

    pub fn with_on_group_message_func<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Arc<GroupMessageEvent>, Arc<ActionManager>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        self.on_group_message_func = Arc::new(move |msg, act| {
            let fut = func(msg, act);
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send + Sync>>
        });
        self
    }

    pub fn with_on_private_message_func<F, Fut>(mut self, func: F) -> Self
    where
        F: Fn(Arc<PrivateMessageEvent>, Arc<ActionManager>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        self.on_private_message_func = Arc::new(move |msg, act| {
            let fut = func(msg, act);
            Box::pin(fut) as Pin<Box<dyn Future<Output = ()> + Send + Sync>>
        });
        self
    }
}

#[async_trait]
impl BasePlugin for Plugin {
    async fn on_load(self: Arc<Self>, _event_nexus: Arc<EventNexus>, _act: Arc<ActionManager>) {
        tracing::info!(r#"[插件已加载] [{}: {}]"#, self.name, self.version,);
    }

    async fn on_update(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) {
        let pattern_regex = if let Trigger::Pattern(r) = self.trigger.clone() {
            regex::Regex::new(&r).unwrap()
        } else {
            regex::Regex::new("").unwrap()
        };

        let group_port = event_nexus.get_group_message_port();
        let private_port = event_nexus.get_private_message_port();

        loop {
            tokio::select! {
                Ok(group_msg) = group_port.recv() => {
                    match self.trigger.clone() {
                        Trigger::StartWith(start_str) => {
                            if group_msg.raw_message.starts_with(&start_str) {
                                (self.on_group_message_func)(group_msg, act.clone()).await;
                            };
                        },
                        Trigger::Pattern(_) => {
                            if pattern_regex.is_match(&group_msg.raw_message) {
                                (self.on_group_message_func)(group_msg, act.clone()).await;
                            };
                        },
                        Trigger::Always => {}
                    }
                },
                Ok(private_msg) = private_port.recv() => {
                    match self.trigger.clone() {
                        Trigger::StartWith(start_str) => {
                            if private_msg.raw_message.starts_with(&start_str) {
                                (self.on_private_message_func)(private_msg, act.clone()).await;
                            };

                        },
                        Trigger::Pattern(_) => {
                            if pattern_regex.is_match(&private_msg.raw_message) {
                                (self.on_private_message_func)(private_msg, act.clone()).await;
                            };
                        },
                        Trigger::Always => {}
                    }
                }
            }
        }
    }

    async fn on_unload(self: Arc<Self>, _event_nexus: Arc<EventNexus>, _act: Arc<ActionManager>) {}
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}
