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
    system_prompts: DashMap<String, String>,
    max_histories: usize,
}

impl AiChatPlugin {
    pub fn new(token: impl Into<String>) -> Self {
        let token: String = token.into();
        Self {
            token: token.clone(),
            client: deepseek::Client::new(token).unwrap(),
            session: Arc::new(DashMap::new()),
            system_prompts: DashMap::new(),
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

    pub async fn add_personal(&mut self, personal_name: String, prompt: String) {
        self.system_prompts.insert(personal_name, prompt);
    }

    pub async fn read_personal_name_list(&self) -> Vec<String> {
        let mut name_list: Vec<String> = Vec::new();
        for prompt in self.system_prompts.clone() {
            name_list.push(prompt.0.clone());
        }
        name_list
    }

    pub async fn clear_history(&mut self, user_id: String) {
        self.get_user_state_by_id(user_id)
            .write()
            .await
            .history
            .clear();
    }

    pub async fn chat(&self, user_id: String, msg: rig::message::Message) -> String {
        let user_state = self.get_user_state_by_id(user_id);
        let llm = self.client.agent(DEEPSEEK_CHAT).build();
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
                rig::message::Message::user(msg.raw_message.clone()),
            )
            .await;
        act.send_private_message(msg.sender.user_id, Message::new().with_text(response))
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

        if !private_message.raw_message.starts_with("/") && !self.token.is_empty() {
            self.on_private_message(private_message, act).await;
        }
    }
    async fn on_unload(self: Arc<Self>) {}
}

#[derive(Serialize, Deserialize, Clone)]
struct UserChatState {
    pub user_id: String,
    pub history: Vec<rig::message::Message>,
    pub current_personal: String,
}

impl UserChatState {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            history: Vec::new(),
            current_personal: "你是一个有用的助手".to_string(),
        }
    }
}
