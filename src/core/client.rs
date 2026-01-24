use crate::utils::channels::WebSocketMessageBus;
use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    routing::any,
};
use std::sync::Arc;
pub struct NapcatClient {
    host: String,
    port: String,
    msg_bus: Arc<WebSocketMessageBus>,
}

impl NapcatClient {
    pub fn new(msg_bus: WebSocketMessageBus) -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: "3000".to_string(),
            msg_bus: Arc::new(msg_bus),
        }
    }

    pub async fn connect(&self) {
        let addr = format!("{}:{}", self.host, self.port);
        let m_state = self.msg_bus.clone();
        let router: Router<()> = Router::new()
            .route(
                "/ws",
                any(
                    move |ws: WebSocketUpgrade, State(msg_bus): State<Arc<WebSocketMessageBus>>| async move {
                        ws.on_upgrade(move |ws: WebSocket| async move {
                            Self::handle_socket(ws, msg_bus).await
                        })
                    },
                ),
            )
            .with_state(m_state);
        let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
        println!("ws connectting, addr: {}", addr);
        axum::serve(listener, router).await.unwrap();
    }

    async fn handle_socket(mut ws: WebSocket, msg_bus: Arc<WebSocketMessageBus>) {
        loop {
            tokio::select! {
                res = ws.recv() => match res {
                    Some(Ok(msg)) => {
                        if let Ok(msg_str) = msg.to_text() {
                            msg_bus.send(msg_str.to_string());
                        } else {
                            break;
                        }
                    },
                    _ => {break;}
                },
                res = msg_bus.recv() => {
                    if let Some(msg_str) = res {
                        let msg = Message::Text(msg_str.into());
                        ws.send(msg).await.unwrap();
                    }
                }
            }
        }
    }

    pub fn get_receiver(&self) -> tokio::sync::broadcast::Receiver<String> {
        self.msg_bus.get_receiver()
    }
}
