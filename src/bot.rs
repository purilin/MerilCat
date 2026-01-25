use crate::core::api::NapcatApi;
use crate::core::client::NapcatClient;
use crate::core::event::EventManager;
use crate::plugin::PluginManager;
use crate::utils::channels::{WebSocketMessageBus, WebSocketSessionManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::{Mutex, broadcast, mpsc};
pub struct MerilBot {
    ws_session_manager: WebSocketSessionManager,
    client: NapcatClient,
}

static INSTANCE: OnceLock<Arc<MerilBot>> = OnceLock::new();
impl MerilBot {
    fn new() -> Self {
        let (mpsc_tx, mpsc_rx) = mpsc::unbounded_channel::<String>();
        let (broadcast_tx, broadcast_rx) = broadcast::channel(256);
        let ws_bus = WebSocketMessageBus::new(broadcast_tx, mpsc_rx);
        let ws_ssm = WebSocketSessionManager::new(mpsc_tx, broadcast_rx);

        Self {
            ws_session_manager: ws_ssm,
            client: NapcatClient::new(ws_bus),
        }
    }

    pub fn init() -> &'static Arc<MerilBot> {
        let arc_self = INSTANCE.get_or_init(|| Arc::new(MerilBot::new()));

        //init
        {
            let event_manager = EventManager::init(arc_self.get_ssmanager());
            tokio::spawn(async move { event_manager.handle_event().await });
            PluginManager::init();
            tokio::spawn(async move { PluginManager::reg_init().await });

            let _ = NapcatApi::init(arc_self.get_ssmanager());
            let _ = crate::config::GlobalState.get_or_init(|| Mutex::new(HashMap::new()));
        };
        INSTANCE.get().unwrap()
    }

    pub fn get() -> &'static Arc<MerilBot> {
        INSTANCE.get().unwrap()
    }

    pub fn get_ssmanager(&self) -> WebSocketSessionManager {
        WebSocketSessionManager::new(
            self.ws_session_manager.get_sender(),
            self.client.get_receiver(),
        )
    }

    pub async fn run(&self) {
        self.client.connect().await;
    }
}
