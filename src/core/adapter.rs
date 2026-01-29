use crate::config::Config;
use crate::types::signal_type::{SignalHub, SignalPort};
use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    routing::any,
};
use serde_json::Value;
use std::sync::Arc;

pub struct NapcatAdapter {
    ws_event_hub: Arc<SignalHub<Value>>,
    ws_action_hub: Arc<SignalHub<Value>>,
}

impl NapcatAdapter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ws_event_hub: Arc::new(SignalHub::new()),
            ws_action_hub: Arc::new(SignalHub::new()),
        })
    }

    pub fn run(self: &Arc<Self>) {
        let arc_self = self.clone();
        let fut = async move {
            arc_self.clone().connect().await;
        };
        tokio::spawn(fut);
    }

    pub fn get_event_port(&self) -> SignalPort<Value> {
        self.ws_event_hub.get_port()
    }

    pub fn get_action_port(&self) -> SignalPort<Value> {
        self.ws_action_hub.get_port()
    }

    async fn connect(&self) {
        let addr = Config::get_or_init().websocket_addr();
        let m_state = (self.ws_event_hub.clone(), self.ws_action_hub.clone());
        type MyState = State<(Arc<SignalHub<Value>>, Arc<SignalHub<Value>>)>;
        let router: Router<()> = Router::new()
            .route(
                "/ws",
                any(
                    move |ws: WebSocketUpgrade, State(hubs): MyState| async move {
                        ws.on_upgrade(move |ws: WebSocket| async move {
                            Self::handle_socket(ws, hubs).await
                        })
                    },
                ),
            )
            .with_state(m_state);
        let listener = tokio::net::TcpListener::bind(addr.clone()).await.unwrap();
        tracing::info!("[初始化] 成功启用服务 {}", addr);
        axum::serve(listener, router).await.unwrap();
    }

    async fn handle_socket(
        mut ws: WebSocket,
        hubs: (Arc<SignalHub<Value>>, Arc<SignalHub<Value>>),
    ) {
        let ws_event_hub = hubs.0.clone();
        let ws_action_hub = hubs.1.clone();

        loop {
            tokio::select! {
                res = ws.recv() => {
                    let response  = if let Some(response) = res {
                        response
                    } else {
                        continue
                    };

                    let message = match response {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::warn!("[接收] 网络读取异常 {}", e);
                            continue;
                        }
                    };

                    let res_str = if let Ok(res_str) = message.to_text() {
                        res_str
                    } else{
                        tracing::warn!("[数据跳过] 可能接受了非文本类型");
                        continue
                    };

                    let res_value: Value = if let Ok(res_value)= serde_json::from_str(res_str) {
                        res_value
                    } else {
                        tracing::warn!("[数据异常] 转化Value失败: {}", res_str);
                        continue;
                    };

                    if res_value.get("echo").is_some() {
                        let _ = ws_action_hub.send(res_value);
                    } else {
                        let _ = ws_event_hub.send(res_value);
                    }
                },
                res = ws_event_hub.recv() => {
                    let res_value = if let Some(res_value) = res {
                        res_value
                    } else {
                        continue;
                    };
                    let res_str = if let Some(res_str) = res_value.as_str() {
                        res_str
                    } else {
                        continue;
                    };
                    let msg = Message::Text(res_str.into());
                    let _ = ws.send(msg).await;
                },
                res = ws_action_hub.recv() => {
                    let res_value = if let Some(res_value) = res {
                        res_value
                    } else {
                        continue;
                    };
                    let res_str = res_value.to_string();
                    let msg = Message::Text(res_str.into());
                    let _ = ws.send(msg).await;
                }
            }
        }
    }
}
