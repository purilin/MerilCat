use crate::utils::parser::message_parser::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Serialize, Deserialize, Clone)]
pub struct NapcatRequestData {
    action: String,
    echo: String,
    params: Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateMessage {
    user_id: String,
    message: Message,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupMessage {
    group_id: String,
    message: Message,
}

impl PrivateMessage {
    pub fn new(user_id: impl ToString, message: Message) -> Self {
        Self {
            user_id: user_id.to_string(),
            message,
        }
    }
}

impl GroupMessage {
    pub fn new(group_id: impl Into<String>, message: Message) -> Self {
        Self {
            group_id: group_id.into(),
            message,
        }
    }
}

impl NapcatRequestData {
    pub fn new() -> Self {
        Self {
            action: "".into(),
            echo: "".into(),
            params: "".into(),
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    pub fn with_echo(mut self, text: impl Into<String>) -> Self {
        self.echo = text.into();
        self
    }

    pub fn with_params(mut self, data: impl Into<Value>) -> Self {
        self.params = data.into();
        self
    }
}
