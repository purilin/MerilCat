use std::{
    pin::Pin,
    sync::{Arc, OnceLock},
};
use tokio::select;
use tokio::sync::Mutex;

use crate::utils::{
    channels::WebSocketSessionManager,
    parser::event_parser::{AnyEvent, message_event::MessageEvent, meta_event::MetaEvent},
};

pub mod func_types {
    use std::pin::Pin;
    use std::sync::Arc;

    use crate::utils::parser::event_parser::message_event::{
        GroupMessageEvent, PrivateMessageEvent,
    };
    pub type PrivateMessageFunction =
        Arc<dyn Fn(PrivateMessageEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

    pub type GroupMessageFunction =
        Arc<dyn Fn(GroupMessageEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
}

pub struct EventManager {
    ws_ssmanager: WebSocketSessionManager,
    private_msg_event_pool: Mutex<Vec<func_types::PrivateMessageFunction>>,
    group_msg_event_pool: Mutex<Vec<func_types::GroupMessageFunction>>,
}
static INSTANCE: OnceLock<EventManager> = OnceLock::new();
impl EventManager {
    fn new(ws_session_manager: WebSocketSessionManager) -> Self {
        Self {
            ws_ssmanager: ws_session_manager,
            private_msg_event_pool: Mutex::new(Vec::new()),
            group_msg_event_pool: Mutex::new(Vec::new()),
        }
    }
    pub fn init(ws_session_manager: WebSocketSessionManager) -> &'static Self {
        INSTANCE.get_or_init(move || Self::new(ws_session_manager))
    }

    pub fn get() -> &'static Self {
        INSTANCE.get().unwrap()
    }

    pub async fn reg_private_event<F, Fut>(&self, func: F)
    where
        F: Fn(crate::utils::parser::event_parser::message_event::PrivateMessageEvent) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_func =
            Arc::new(move |msg| Box::pin(func(msg)) as Pin<Box<dyn Future<Output = ()> + Send>>);
        let mut mut_msgpool = self.private_msg_event_pool.lock().await;
        mut_msgpool.push(boxed_func);
    }

    pub async fn reg_group_event<F, Fut>(&self, func: F)
    where
        F: Fn(crate::utils::parser::event_parser::message_event::GroupMessageEvent) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_func =
            Arc::new(move |msg| Box::pin(func(msg)) as Pin<Box<dyn Future<Output = ()> + Send>>);
        let mut mut_msgpool = self.group_msg_event_pool.lock().await;
        mut_msgpool.push(boxed_func);
    }

    pub async fn handle_event(&self) {
        loop {
            select! {
                res = self.ws_ssmanager.recv() => match res {
                    Ok(msg_str) => {
                        match serde_json::from_str::<AnyEvent>(msg_str.as_str()) {
                            Ok(AnyEvent::Message(type_msg)) => {
                                match type_msg {
                                    MessageEvent::Group(msg)=> {
                                        println!("[group={}]{}({}): {}",msg.group_id ,msg.sender.nickname, msg.sender.user_id, msg.raw_message);
                                        let group_msg_pool = {let pool = self.group_msg_event_pool.lock().await; pool.clone() };
                                        for func in group_msg_pool.iter() {
                                            let fut = func(msg.clone());
                                            tokio::spawn(fut);
                                        }
                                    },
                                    MessageEvent::Private(msg) => {
                                        println!("[private]{}({}): {}", msg.sender.nickname, msg.sender.user_id, msg.raw_message);
                                        let private_msg_pool = {let pool = self.private_msg_event_pool.lock().await; pool.clone()};
                                        for func in private_msg_pool.iter() {
                                            let fut = func(msg.clone());
                                            tokio::spawn(fut);
                                        }
                                    }
                                }
                            },
                            Ok(AnyEvent::Meta(meta)) => {
                                match meta {
                                    MetaEvent::LifeCycle(_) => {
                                        println!("Already connect to napcat!");
                                    },
                                    MetaEvent::HeartBeat(heart) => {
                                        println!("[heart: {}], online={}", heart.interval, heart.status.online);
                                    }
                                }
                            },
                            Ok(AnyEvent::Other) => {
                                println!("Other Event");
                            },
                            Err(_) => {
                                println!("\n[Parser Event Error!] \n{} \n",msg_str);
                            }
                        }
                    },
                    _ => {
                        break;
                    }
                },
            }
        }
    }
}
