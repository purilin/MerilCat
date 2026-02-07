use crate::{
    types::action_type::NapcatRequestData, types::message_type::Message,
    types::signal_type::SignalPort,
};
use dashmap::DashMap;
use serde_json::{Value, json};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::oneshot;
use tokio::time;

pub struct ActionManager {
    ws_port: SignalPort<Value>,
    pending_requestions: Arc<DashMap<String, oneshot::Sender<Value>>>,
    pending_atomic: AtomicU64,
}

impl ActionManager {
    pub fn new(ws_port: SignalPort<Value>) -> Arc<Self> {
        Arc::new(Self {
            ws_port,
            pending_requestions: Arc::new(DashMap::new()),
            pending_atomic: AtomicU64::new(0),
        })
    }

    pub async fn request(&self, data: NapcatRequestData) -> Result<Value, &str> {
        let key = self
            .pending_atomic
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let data = data.with_echo(key.to_string());
        let value = if let Ok(value) = serde_json::to_value(&data) {
            value
        } else {
            return Err("Serde Error");
        };
        tracing::info!(
            "<<[{}] {}",
            value["action"].as_str().unwrap_or(""),
            value["params"]
        );
        let _ = self.ws_port.send(value.clone());
        let (tx, rx) = oneshot::channel::<Value>();
        self.pending_requestions.insert(key.to_string(), tx);
        let Ok(Ok(res)) = time::timeout(time::Duration::from_secs(10), rx).await else {
            tracing::warn!("[TimeOutError] ActionTimeOut");
            return Err("Time Out Error");
        };
        Ok(res)
    }

    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: Message,
    ) -> Result<Value, &str> {
        let value = json!({
            "user_id": user_id,
            "message": message
        });
        let act = "send_private_msg";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        self.request(data).await
    }

    pub async fn send_group_message(&self, group_id: i64, message: Message) -> Result<Value, &str> {
        let value = json!({
            "group_id": group_id,
            "message": message
        });
        let act = "send_group_msg";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        self.request(data).await
    }

    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<String, &str> {
        let value = json!({
            "user_id": user_id,
            "times": times
        });
        let act = "send_like";
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        let message = self.request(data).await.ok().and_then(|value| {
            value
                .get("message")
                .and_then(|message| message.get("message"))
                .and_then(|message| message.as_str())
                .map(|message| message.to_string())
        });
        message.ok_or("Send Like Error.")
    }

    pub async fn send_private_poke(&self, user_id: i64) -> Result<Value, &str> {
        let act = "friend_poke";
        let value = json!({
            "user_id": user_id,
        });
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        self.request(data).await
    }

    pub async fn send_group_poke(&self, group_id: i64, user_id: i64) -> Result<Value, &str> {
        let act = "group_poke";
        let value = json!({
            "user_id": user_id,
            "group_id": group_id,
        });
        let data = NapcatRequestData::new().with_action(act).with_params(value);
        self.request(data).await
    }

    pub fn run(self: Arc<Self>) {
        let fut = async move {
            loop {
                let Ok(res) = self.ws_port.recv().await else {
                    tracing::warn!("[Response Err]");
                    continue;
                };
                tracing::info!("[Reply] {}", res.to_string());
                let echo = if let Some(echo) = res.get("echo") {
                    echo.as_str()
                } else {
                    continue;
                };

                echo.map(|echo| {
                    if let Some((_, tx)) = self.pending_requestions.remove(&echo.to_string()) {
                        let _ = tx.send(res.clone());
                    };
                });
            }
        };
        tokio::spawn(fut);
    }
}
