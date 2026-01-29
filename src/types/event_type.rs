use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "post_type")]
pub enum AnyEvent {
    #[serde(rename = "message")]
    Message(message_event::MessageEvent),
    #[serde(rename = "meta_event")]
    Meta(meta_event::MetaEvent),
    #[serde(rename = "notice")]
    Notice(notice_event::NoticeEvent),
    #[serde(other)]
    Other,
}

pub mod meta_event {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(tag = "meta_event_type")]
    pub enum MetaEvent {
        #[serde(rename = "lifecycle")]
        LifeCycle(LifeCycleEvent),
        #[serde(rename = "heartbeat")]
        HeartBeat(HeartBeatEvent),
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct LifeCycleEvent {
        //pub meta_event_type: String,
        pub self_id: i64,
        pub sub_type: String,
        pub time: i64,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct HeartBeatEvent {
        pub interval: i64,
        //pub meta_event_type: String,
        pub self_id: i64,
        pub status: HeartBeatStatus,
        pub time: i64,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct HeartBeatStatus {
        pub good: bool,
        pub online: bool,
    }
}

pub mod message_event {
    use crate::types::message_type::MessageSegment;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct SenderInfo {
        pub user_id: i64,
        pub nickname: String,
        pub card: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct PrivateMessageEvent {
        pub message_id: i64,
        pub self_id: i64,
        pub time: i64,
        pub raw_message: String,
        pub sender: SenderInfo,
        pub message: Vec<MessageSegment>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct GroupMessageEvent {
        pub group_id: i64,
        pub message_id: i64,
        pub self_id: i64,
        pub time: i64,
        pub group_name: String,
        pub raw_message: String,
        pub sender: SenderInfo,
        pub message: Vec<MessageSegment>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct BaseMessageEvent {
        pub message_id: i64,
        pub self_id: i64,
        pub time: i64,
        pub raw_message: String,
        pub sender: SenderInfo,
        pub message: Vec<MessageSegment>,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    #[serde(tag = "message_type")]
    pub enum MessageEvent {
        #[serde(rename = "group")]
        Group(GroupMessageEvent),
        #[serde(rename = "private")]
        Private(PrivateMessageEvent),
    }

    impl From<PrivateMessageEvent> for MessageEvent {
        fn from(value: PrivateMessageEvent) -> Self {
            MessageEvent::Private(value)
        }
    }

    impl From<GroupMessageEvent> for MessageEvent {
        fn from(value: GroupMessageEvent) -> Self {
            MessageEvent::Group(value)
        }
    }

    impl From<PrivateMessageEvent> for BaseMessageEvent {
        fn from(value: PrivateMessageEvent) -> Self {
            Self {
                message_id: value.message_id,
                self_id: value.self_id,
                time: value.time,
                raw_message: value.raw_message,
                sender: value.sender,
                message: value.message,
            }
        }
    }

    impl From<GroupMessageEvent> for BaseMessageEvent {
        fn from(value: GroupMessageEvent) -> Self {
            Self {
                message_id: value.message_id,
                self_id: value.self_id,
                time: value.time,
                raw_message: value.raw_message,
                sender: value.sender,
                message: value.message,
            }
        }
    }
}

pub mod notice_event {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct NoticeEvent {
        pub group_id: u64,
        pub notice_type: String,
        pub self_id: u64,
        pub status_text: String,
        pub time: u64,
        pub user_id: u64,
    }
}
