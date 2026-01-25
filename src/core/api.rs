use crate::utils::channels::WebSocketSessionManager;
use crate::utils::parser::{
    message_parser::Message,
    request_parser::{NapcatRequestData, PrivateMessage},
};
use std::sync::OnceLock;

pub struct NapcatApi {
    ws_ssmanager: WebSocketSessionManager,
}

static INSTANCE: OnceLock<NapcatApi> = OnceLock::new();
impl NapcatApi {
    fn new(ws_ssmanager: WebSocketSessionManager) -> Self {
        Self { ws_ssmanager }
    }

    pub fn init(ws_ssmanager: WebSocketSessionManager) -> &'static Self {
        INSTANCE.get_or_init(|| Self::new(ws_ssmanager))
    }

    pub fn get() -> &'static Self {
        INSTANCE.get().unwrap()
    }

    pub async fn request(&self, data: NapcatRequestData) {
        let text_json = serde_json::to_string(&data).unwrap();
        self.ws_ssmanager.send(text_json);
    }

    pub async fn send_private_message(&self, user_id: i64, message: Message) {
        let private_msg = PrivateMessage::new(user_id, message);
        let json_value = serde_json::to_value(private_msg).unwrap();
        let act = "send_private_msg";
        let data = NapcatRequestData::new()
            .with_action(act)
            .with_params(json_value);
        self.request(data).await
    }
}
