use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct Message {
    message: Vec<MessageSegment>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            message: Vec::new(),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.message
            .push(MessageSegment::Text { text: text.into() });
        self
    }

    /// AT 某人
    pub fn with_at(mut self, qq: impl Into<String>) -> Self {
        self.message.push(MessageSegment::At { qq: qq.into() });
        self
    }

    /// 发送图片 (file 可以是本地路径、URL 或 base64)
    pub fn with_image(mut self, file: impl Into<String>) -> Self {
        self.message
            .push(MessageSegment::Image { file: file.into() });
        self
    }

    /// 发送表情
    pub fn with_face(mut self, id: i32) -> Self {
        self.message.push(MessageSegment::Face { id });
        self
    }

    /// 回复某条消息
    pub fn with_reply(mut self, message_id: i32) -> Self {
        self.message.push(MessageSegment::Reply { id: message_id });
        self
    }

    /// 发送语音
    pub fn with_record(mut self, file: impl Into<String>) -> Self {
        self.message
            .push(MessageSegment::Record { file: file.into() });
        self
    }

    /// 发送视频
    pub fn with_video(mut self, file: impl Into<String>) -> Self {
        self.message
            .push(MessageSegment::Video { file: file.into() });
        self
    }

    /// 掷骰子
    pub fn with_dice(mut self) -> Self {
        self.message.push(MessageSegment::Dice {});
        self
    }

    /// 猜拳
    pub fn with_rps(mut self) -> Self {
        self.message.push(MessageSegment::Rps {});
        self
    }

    /// 发送文件 (NapCat 特有)
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.message
            .push(MessageSegment::File { file: path.into() });
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum MessageSegment {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "at")]
    At { qq: String },

    #[serde(rename = "image")]
    Image { file: String },

    #[serde(rename = "face")]
    Face { id: i32 },

    #[serde(rename = "json")]
    Json { data: String },

    #[serde(rename = "record")]
    Record { file: String },

    #[serde(rename = "video")]
    Video { file: String },

    #[serde(rename = "reply")]
    Reply { id: i32 },

    #[serde(rename = "dice")]
    Dice {},

    #[serde(rename = "rps")]
    Rps {},

    #[serde(rename = "file")]
    File { file: String },

    #[serde(rename = "music")]
    Music(MusicData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)] // 音乐消息字段差异大，用自适应解析
pub enum MusicData {
    BuiltIn {
        #[serde(rename = "type")]
        kind: String, // "qq" 或 "163"
        id: String,
    },
    Custom {
        #[serde(rename = "type")]
        kind: String, // "custom"
        url: String,
        audio: String,
        title: String,
        image: Option<String>,
    },
}
