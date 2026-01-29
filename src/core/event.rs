use crate::types::{
    event_type::{
        AnyEvent,
        message_event::{GroupMessageEvent, MessageEvent, PrivateMessageEvent},
        meta_event::{HeartBeatEvent, LifeCycleEvent, MetaEvent},
    },
    signal_type::{SignalHub, SignalPort},
};
use serde_json::Value;
use std::sync::Arc;

pub struct EventManager {
    ws_port: SignalPort<Value>,
    hubs: EventHubs,
}

impl EventManager {
    pub fn new(ws_port: SignalPort<Value>) -> Arc<Self> {
        Arc::new(Self {
            ws_port,
            hubs: EventHubs::new(),
        })
    }

    async fn handle_event(&self) {
        let res_value = self.ws_port.recv().await.unwrap();
        let any_event_res = serde_json::from_value::<AnyEvent>(res_value.clone());
        let any_event = match any_event_res {
            Ok(message) => message,
            Err(e) => {
                tracing::warn!("[数据异常] 数据解析异常 {}, {}", e, res_value);
                return;
            }
        };
        let _ = self.hubs.all_event_hub.send(any_event.clone());
        match any_event {
            AnyEvent::Message(msg_event) => match msg_event {
                MessageEvent::Group(group_msg) => {
                    let _ = self.hubs.group_message_hub.send(group_msg.clone());
                    tracing::info!(
                        "[Group: {}-{}] [{}-{}]: {}",
                        group_msg.group_name,
                        group_msg.group_id,
                        group_msg.sender.nickname,
                        group_msg.sender.user_id,
                        group_msg.raw_message
                    );
                }
                MessageEvent::Private(private_msg) => {
                    let _ = self.hubs.private_message_hub.send(private_msg.clone());
                    tracing::info!(
                        "[Private] [{}-{}]: {}",
                        private_msg.sender.nickname,
                        private_msg.sender.user_id,
                        private_msg.raw_message
                    );
                }
            },
            AnyEvent::Meta(meta_event) => match meta_event {
                MetaEvent::LifeCycle(life_cycle) => {
                    let _ = self.hubs.lifecycle_hub.send(life_cycle.clone());
                    tracing::info!(
                        "[LifeCycle] [id = {}] Napcat Already Connected",
                        life_cycle.self_id
                    );
                }
                MetaEvent::HeartBeat(heart_beat) => {
                    let _ = self.hubs.heartbeat_hub.send(heart_beat.clone());
                    tracing::info!("[HeartBeat] [online = {}]", heart_beat.status.online);
                }
            },
            AnyEvent::Notice(notice_event) => {
                if !notice_event.status_text.is_empty() {
                    tracing::info!(
                        "[Notice] [self_id = {}] [type = {}] [user_id = {}] {}",
                        notice_event.self_id,
                        notice_event.notice_type,
                        notice_event.user_id,
                        notice_event.status_text
                    );
                }
            }
            AnyEvent::Other => {
                let pretty_str = serde_json::to_string_pretty(&res_value.clone()).unwrap();
                tracing::info!("\n[UndefineEvent]\n{}\n", pretty_str);
            }
        }
    }

    pub fn get_event_nexus(&self) -> Arc<EventNexus> {
        self.hubs.get_nexus()
    }

    pub fn run(self: Arc<Self>) {
        let arc_self = self.clone();
        let fut = async move {
            loop {
                arc_self.clone().handle_event().await;
            }
        };
        tokio::spawn(fut);
    }
}

pub struct EventHubs {
    all_event_hub: Arc<SignalHub<Arc<AnyEvent>>>,
    private_message_hub: Arc<SignalHub<Arc<PrivateMessageEvent>>>,
    group_message_hub: Arc<SignalHub<Arc<GroupMessageEvent>>>,
    heartbeat_hub: Arc<SignalHub<Arc<HeartBeatEvent>>>,
    lifecycle_hub: Arc<SignalHub<Arc<LifeCycleEvent>>>,
}

impl EventHubs {
    pub fn new() -> Self {
        Self {
            all_event_hub: Arc::new(SignalHub::new()),
            private_message_hub: Arc::new(SignalHub::new()),
            group_message_hub: Arc::new(SignalHub::new()),
            heartbeat_hub: Arc::new(SignalHub::new()),
            lifecycle_hub: Arc::new(SignalHub::new()),
        }
    }

    pub fn get_nexus(&self) -> Arc<EventNexus> {
        Arc::new(EventNexus {
            all_event_hub: self.all_event_hub.clone(),
            private_message_hub: self.private_message_hub.clone(),
            group_message_hub: self.group_message_hub.clone(),
            heartbeat_hub: self.heartbeat_hub.clone(),
            lifecycle_hub: self.lifecycle_hub.clone(),
        })
    }
}

impl Default for EventHubs {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventNexus {
    all_event_hub: Arc<SignalHub<Arc<AnyEvent>>>,
    private_message_hub: Arc<SignalHub<Arc<PrivateMessageEvent>>>,
    group_message_hub: Arc<SignalHub<Arc<GroupMessageEvent>>>,
    heartbeat_hub: Arc<SignalHub<Arc<HeartBeatEvent>>>,
    lifecycle_hub: Arc<SignalHub<Arc<LifeCycleEvent>>>,
}

impl EventNexus {
    pub fn new(
        all_event_hub: Arc<SignalHub<Arc<AnyEvent>>>,
        private_message_hub: Arc<SignalHub<Arc<PrivateMessageEvent>>>,
        group_message_hub: Arc<SignalHub<Arc<GroupMessageEvent>>>,
        heartbeat_hub: Arc<SignalHub<Arc<HeartBeatEvent>>>,
        lifecycle_hub: Arc<SignalHub<Arc<LifeCycleEvent>>>,
    ) -> Self {
        Self {
            all_event_hub,
            private_message_hub,
            group_message_hub,
            heartbeat_hub,
            lifecycle_hub,
        }
    }

    pub fn get_private_message_port(&self) -> SignalPort<Arc<PrivateMessageEvent>> {
        self.private_message_hub.get_port()
    }

    pub fn get_group_message_port(&self) -> SignalPort<Arc<GroupMessageEvent>> {
        self.group_message_hub.get_port()
    }

    pub fn get_heartbeat_port(&self) -> SignalPort<Arc<HeartBeatEvent>> {
        self.heartbeat_hub.get_port()
    }

    pub fn get_lifecycle_port(&self) -> SignalPort<Arc<LifeCycleEvent>> {
        self.lifecycle_hub.get_port()
    }

    pub fn get_all_event_port(&self) -> SignalPort<Arc<AnyEvent>> {
        self.all_event_hub.get_port()
    }
}

impl Clone for EventNexus {
    fn clone(&self) -> Self {
        Self {
            all_event_hub: self.all_event_hub.clone(),
            private_message_hub: self.private_message_hub.clone(),
            group_message_hub: self.group_message_hub.clone(),
            heartbeat_hub: self.heartbeat_hub.clone(),
            lifecycle_hub: self.lifecycle_hub.clone(),
        }
    }
}

impl Default for EventNexus {
    fn default() -> Self {
        Self::new(
            Arc::new(SignalHub::new()),
            Arc::new(SignalHub::new()),
            Arc::new(SignalHub::new()),
            Arc::new(SignalHub::new()),
            Arc::new(SignalHub::new()),
        )
    }
}
