use crate::{
    types::action_type::NapcatRequestData, types::message_type::Message,
    types::signal_type::SignalPort,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub struct ActionManager {
    ws_port: SignalPort<Value>,
}

impl ActionManager {
    pub fn new(ws_port: SignalPort<Value>) -> Arc<Self> {
        Arc::new(Self { ws_port })
    }

    pub async fn request(&self, data: NapcatRequestData) {
        let value = if let Ok(value) = serde_json::to_value(&data) {
            value
        } else {
            return;
        };
        tracing::info!(
            "<<[{}] {}",
            value["action"].as_str().unwrap_or(""),
            value["params"]
        );
        let _ = self.ws_port.send(value.clone());
    }

    pub async fn send_private_message(&self, user_id: i64, message: Message) {
        let value = json!({
            "user_id": user_id,
            "message": message
        });
        let act = "send_private_msg";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        let _ = self.request(data).await;
    }

    pub async fn send_group_message(&self, group_id: i64, message: Message) {
        let value = json!({
            "group_id": group_id,
            "message": message
        });
        let act = "send_group_msg";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        let _ = self.request(data).await;
    }

    pub async fn send_like(&self, user_id: i64, times: i32) {
        let value = json!({
            "user_id": user_id,
            "times": times
        });
        let act = "send_like";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        let _ = self.request(data).await;
    }
}
