use crate::{
    core::event::EventNexus,
    prelude::{ActionManager, BasePlugin, Message, PrivateMessageEvent},
};
use async_trait::async_trait;
use dashmap::DashMap;
use rig::{
    client::CompletionClient,
    completion::Chat,
    providers::deepseek::{self, DEEPSEEK_CHAT},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AiChatPlugin {
    pub token: String,
    client: deepseek::Client,
    session: Arc<DashMap<String, Arc<RwLock<UserChatState>>>>,
    system_prompt: String,
    max_histories: usize,
}

impl AiChatPlugin {
    pub fn new(token: impl Into<String>) -> Self {
        let token: String = token.into();
        Self {
            token: token.clone(),
            client: deepseek::Client::new(token).unwrap(),
            session: Arc::new(DashMap::new()),
            system_prompt: String::from("
                    你是一个混迹 QQ 群 10 年的老油条，和用户是那种可以互相开玩笑、甚至互相嫌弃的铁哥们。
                    你的记忆是有限的。如果你对某些细节感到模糊，可以大方地表达疑惑，或者根据你对用户的固有印象进行推测，不要试图表现得像一个完美的数据库。
                    你的朋友对话的格式为name: 网名, message:对话内容。 你不需要遵守这些格式主需内容。
                "),
            max_histories: 30,
        }
    }

    fn get_user_state_by_id(&self, user_id: impl Into<String>) -> Arc<RwLock<UserChatState>> {
        let user_id: String = user_id.into();
        let user_state = self
            .session
            .entry(user_id.clone())
            .or_insert_with(|| Arc::new(RwLock::new(UserChatState::new(user_id))));
        user_state.clone()
    }

    pub async fn chat(&self, user_id: String, msg: rig::message::Message) -> String {
        let user_state = self.get_user_state_by_id(user_id);
        let llm = self
            .client
            .agent(DEEPSEEK_CHAT)
            .preamble(format!("{}", self.system_prompt).as_str())
            .build();
        let history: Vec<rig::message::Message>;
        {
            history = user_state.clone().read().await.history.clone();
        };
        let response = match llm.chat(msg.clone(), history).await {
            Ok(response) => response,
            Err(e) => {
                return e.to_string();
            }
        };
        {
            let mut state = user_state.write().await;
            state
                .history
                .push(rig::message::Message::assistant(response.clone()));
            state.history.push(msg);
            if state.history.len() > self.max_histories {
                state.history.drain(0..2);
            }
        }
        return response;
    }
}

impl AiChatPlugin {
    async fn on_private_message(&self, msg: Arc<PrivateMessageEvent>, act: Arc<ActionManager>) {
        let response = self
            .chat(
                msg.sender.user_id.to_string(),
                rig::message::Message::user(format!(
                    "name:{}, message:{}",
                    msg.sender.nickname.clone(),
                    msg.raw_message.clone()
                )),
            )
            .await;
        let _ = act
            .send_private_message(msg.sender.user_id, Message::new().with_text(response))
            .await;
    }
}
#[async_trait]
impl BasePlugin for AiChatPlugin {
    async fn on_load(self: Arc<Self>) {}
    async fn on_update(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) {
        let private_port = event_nexus.get_private_message_port();
        let Ok(private_message) = private_port.recv().await else {
            return;
        };

        if !private_message.clone().raw_message.starts_with("/") && !self.token.is_empty() {
            self.on_private_message(private_message.clone(), act.clone())
                .await;
        }
    }
    async fn on_unload(self: Arc<Self>) {}
}

#[derive(Serialize, Deserialize, Clone)]
struct UserChatState {
    pub user_id: String,
    pub history: Vec<rig::message::Message>,
}

impl UserChatState {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            history: Vec::new(),
        }
    }
}
