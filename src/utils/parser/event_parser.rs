use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "post_type")]
pub enum AnyEvent {
    #[serde(rename = "message")]
    Message(message_event::MessageEvent),
    #[serde(rename = "meta_event")]
    Meta(meta_event::MetaEvent),
    #[serde(other)]
    Other,
}

pub mod meta_event {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    #[serde(tag = "meta_event_type")]
    pub enum MetaEvent {
        #[serde(rename = "lifecycle")]
        LifeCycle(LifeCycleEvent),
        #[serde(rename = "heartbeat")]
        HeartBeat(HeartBeatEvent),
    }

    #[derive(Deserialize, Serialize)]
    pub struct LifeCycleEvent {
        //pub meta_event_type: String,
        pub self_id: i64,
        pub sub_type: String,
        pub time: i64,
    }

    #[derive(Deserialize, Serialize)]
    pub struct HeartBeatEvent {
        pub interval: i64,
        //pub meta_event_type: String,
        pub self_id: i64,
        pub status: HeartBeatStatus,
        pub time: i64,
    }

    #[derive(Deserialize, Serialize)]
    pub struct HeartBeatStatus {
        pub good: bool,
        pub online: bool,
    }
}

pub mod message_event {
    use crate::utils::parser::message_parser::MessageSegment;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone)]
    pub struct SenderInfo {
        pub user_id: i64,
        pub nickname: String,
        pub card: String,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct PrivateMessageEvent {
        pub message_id: i64,
        pub self_id: i64,
        pub time: i64,
        //pub message_type: String,
        pub raw_message: String,
        pub sender: SenderInfo,
        pub message: Vec<MessageSegment>,
    }

    #[derive(Deserialize, Serialize, Clone)]
    pub struct GroupMessageEvent {
        pub group_id: i64,
        pub message_id: i64,
        pub self_id: i64,
        pub time: i64,
        //pub message_type: String,
        pub group_name: String,
        pub raw_message: String,
        pub sender: SenderInfo,
        pub message: Vec<MessageSegment>,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(tag = "message_type")]
    pub enum MessageEvent {
        #[serde(rename = "group")]
        Group(GroupMessageEvent),
        #[serde(rename = "private")]
        Private(PrivateMessageEvent),
    }
}
