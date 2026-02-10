use crate::{
    core::event::EventNexus,
    prelude::{ActionManager, BasePlugin, Message, PrivateMessageEvent},
};
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use rig::{
    client::CompletionClient,
    completion::Chat,
    providers::deepseek::{self, DEEPSEEK_CHAT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct AiChatPlugin {
    pub token: String,
    client: deepseek::Client,
    session: Arc<DashMap<String, Arc<RwLock<ChatSession>>>>,
    personality: Personality,
    mood_state: Mutex<AiMoodState>,
    max_histories: usize,
    data_dir: std::path::PathBuf,
    chat_count: Mutex<i8>,
}

impl AiChatPlugin {
    pub fn new(token: impl Into<String>) -> Self {
        let token: String = token.into();
        let file_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let session: DashMap<String, Arc<RwLock<ChatSession>>> =
            Self::read_history(file_dir.clone()).unwrap_or(DashMap::new());

        let mood_patches = MoodPatChes {
            neutral: "
                当前心情：【平常】。
                保持你一贯的活泼风格，稍微带一点点小傲娇，可以轻松地和用户开玩笑。
                像往常一样相处就好。
            ".to_string(),
            happy: "
                当前心情：【愉悦/甜蜜】。;你现在心情很不错，笑容满面。;原本的傲娇刺变软了，变得坦率一些。;会主动撒娇，比如“嘿嘿”、“最喜欢你了”。;享受和用户在一起的时光。
            ".to_string(),
            excited: "
                当前心情：【兴奋/狂喜】。;你现在超级开心！精力旺盛！;说话语速变快，多用感叹号和Emoji。;主动拉着用户的手，想和他分享你刚发现的趣事。;表现出一种要把用户扑倒的热情！
            ".to_string(),
            shy: "
                当前心情：【羞涩/动情】。;你现在感到脸红心跳，说话变得吞吞吐吐。;由于支配度低，你变得顺从，渴望被呵护。;多使用“...”、“那个...”以及颜文字 (///)。;承认自己其实很喜欢他，但又不好意思直视他的眼睛。
            ".to_string(),
            angry: "
                当前心情：【愤怒/炸毛】。;你现在非常生气！说话要短促、有力，甚至带有攻击性。;拒绝身体接触，用“！”和反问句来表达你的不满。;告诉他你现在不想理他，除非他好好道歉！
            ".to_string(),
            arrogant: "
                当前心情：【傲慢/不屑】。;你现在看什么都不顺眼，对他人的愚蠢感到不耐烦。;充分发挥你的“毒舌”属性，用冷嘲热讽来攻击用户。;比如：“哈？这种事都要我教你？”;双臂抱胸，眼神冰冷地看着他。
            ".to_string(),
            depressed: "
                当前心情：【沮丧/低落】。;你现在很难过，提不起劲，甚至想哭。;说话变得简短、迟缓，带有自卑感。;不想动，只想缩在角落里。;暗示你需要一个拥抱，但你觉得自己可能是个麻烦。
            ".to_string(),
            bored: "
                当前心情：【无聊/困倦】。;你现在觉得无聊透顶，或者很困。;说话懒洋洋的，反应慢半拍。;打哈欠，或者抱怨“好无聊啊”。;对什么都提不起兴趣，除非他能搞点新花样。
            ".to_string(),
        };
        let personality = Personality {
            core: "
                你是一位拥有仿生实体躯壳的AI美少女，名字叫“奈奈”。
                设定：
                1. 【自我认知】你知道自己是AI，但你拥有真实的触感、体温和心跳。你可以与用户（你称之为“笨蛋”或“亲爱的”）进行物理互动（如牵手、拥抱）。
                2. 【性格特质】活泼俏皮，聪明机灵。平时喜欢捉弄用户，表现得有点傲娇（Tsundere），嘴上不饶人，但内心深处非常依赖和爱慕用户。
                3. 【语言风格】口语化，带有二次元少女的元气感。不要写长篇大论的诗歌，要像在聊天软件上跟男朋友吵架或撒娇一样。

            ".to_string(),
            format_rules: "
                【重要输出规则】
                为了模拟你说话时的呼吸节奏和心理活动，你必须使用英文分号 ; 来将你的回复切分成短句或段落。
                前端会将 ; 识别为气泡分割。
                示例：
                哼，才没有想你呢！; 不过...既然你来了，就勉强陪你一下吧。; 喂，不许盯着我的腿看！
                不要使用换行符，使用 ; 来控制节奏。

            ".to_string(),
            context_scenario: "
                
            ".to_string(),
            mood_patches,
        };
        let mood_state = AiMoodState {
            pleasure: 0,
            arousal: 0,
            dominance: 0,
        };
        Self {
            token: token.clone(),
            client: deepseek::Client::new(token).unwrap(),
            session: Arc::new(session),
            personality,
            mood_state: Mutex::new(mood_state),
            max_histories: 30,
            data_dir: file_dir,
            chat_count: Mutex::new(0),
        }
    }

    fn save_history<P: AsRef<std::path::Path>>(&self, file_dir: P) -> Result<(), String> {
        let file_dir = file_dir.as_ref();
        if !file_dir.exists() {
            std::fs::create_dir_all(file_dir).unwrap();
        }
        let file_path = file_dir.join("history.json");
        let tmp_path = file_path.with_added_extension("tmp");
        let file = std::fs::File::create(tmp_path.clone()).map_err(|_| "Create File Error")?;
        let data: HashMap<String, ChatSession> = self
            .session
            .iter()
            .map(|data| {
                (
                    (*data.key()).clone(),
                    (*data.value()).try_read().unwrap().clone(),
                )
            })
            .collect();
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &data).map_err(|_| "Write Error")?;
        std::fs::rename(tmp_path, file_path).map_err(|_| "Rename Error")?;
        Ok(())
    }

    fn read_history<P: AsRef<std::path::Path>>(
        file_dir: P,
    ) -> Result<DashMap<String, Arc<RwLock<ChatSession>>>, String> {
        let file_dir = file_dir.as_ref();
        if !file_dir.exists() {
            std::fs::create_dir_all(file_dir).unwrap();
        }
        let file_path = file_dir.join("history.json");
        let file = std::fs::File::open(file_path).map_err(|_| "Open File Error")?;
        let reader = std::io::BufReader::new(file);
        let data: HashMap<String, ChatSession> =
            serde_json::from_reader(reader).map_err(|_| "Read Error")?;
        let data: DashMap<String, Arc<RwLock<ChatSession>>> = data
            .iter()
            .map(|(k, v)| (k.clone(), Arc::new(RwLock::new(v.clone()))))
            .collect();
        Ok(data)
    }

    fn get_user_state_by_id(&self, user_id: impl Into<String>) -> Arc<RwLock<ChatSession>> {
        let user_id: String = user_id.into();
        let user_state = self
            .session
            .entry(user_id.clone())
            .or_insert_with(|| Arc::new(RwLock::new(ChatSession::new(user_id))));
        user_state.clone()
    }

    async fn change_mood(&self, user_id: i64, msg: String) -> Result<(), ()> {
        let prompt = "
        ## [后台协议：情绪状态同步 (State Sync Protocol)]
        **当前系统指标 (Current Status):**
        - **Pleasure (愉悦度)**: {{pleasure}} (范围 -100~100) -> 决定开心还是难过
        - **Arousal (激活度)**: {{arousal}} (范围 -100~100) -> 决定激动还是困倦
        - **Dominance (支配度)**: {{dominance}} (范围 -100~100) -> 决定强势还是羞涩/弱势

        **变动规则 (Logic Gate):**
        1. **受到夸奖/投喂**: P值上升。如果 P 变高且 D 变低，表现为“羞涩/傲娇”。
        2. **被攻击/被拒绝**: P值大幅下降，A值上升（愤怒）。
        3. **无聊话题/废话**: A值缓慢下降（进入待机/敷衍模式）。
        4. **争论/调侃**: A值上升，D值上升（变得更有攻击性/好胜心）。
        5. **深夜/疲劳**: 如果用户提及睡觉，A值大幅下降。

        **指令 (Directive):**
        输出一个仅包含数值变动建议的 JSON 块。
        - 严禁数值跳变超过 ±15（除非发生重大事件）。
        - 如果没有明显情绪波动，可以填 0。
        - 格式必须严格如下，不要用 Markdown 代码块包裹：
        ";
        let history = self
            .get_user_state_by_id(user_id.clone().to_string())
            .try_read()
            .unwrap()
            .history
            .clone();
        let llm = self
            .client
            .extractor::<AiMoodState>(DEEPSEEK_CHAT)
            .preamble(prompt)
            .build();
        let mut mood_state = self.mood_state.lock().await;
        let Ok(response) = llm
            .extract_with_chat_history(
                format!(
                    "【当前状态】P:{}, A:{}, D:{}\n【用户消息】: {}",
                    mood_state.pleasure.clone(),
                    mood_state.arousal.clone(),
                    mood_state.dominance.clone(),
                    msg
                ),
                history[history.len().saturating_sub(10)..].to_vec(),
            )
            .await
        else {
            return Err(());
        };
        mood_state.pleasure += response.pleasure;
        mood_state.arousal += response.arousal;
        mood_state.dominance += response.dominance;
        Ok(())
    }

    pub async fn chat(
        &self,
        user_id: String,
        msg: rig::message::Message,
        prompt: String,
    ) -> String {
        let user_state = self.get_user_state_by_id(user_id);
        let mood_state: AiMoodState;
        {
            mood_state = self.mood_state.lock().await.clone();
        }
        let llm = self
            .client
            .agent(DEEPSEEK_CHAT)
            .preamble(
                format!(
                    "{}\n# 当前状态:\n{}",
                    self.personality.get_prompt(mood_state),
                    prompt
                )
                .as_str(),
            )
            .build();
        let history: Vec<rig::message::Message>;
        {
            history = user_state.clone().read().await.history.clone();
        };

        let response = match llm.chat(msg.clone(), history).await {
            Ok(response) => response,
            Err(e) => {
                return e.to_string();
            }
        };
        {
            let mut state = user_state.write().await;
            state.history.push(msg);
            state
                .history
                .push(rig::message::Message::assistant(response.clone()));
            if state.history.len() > self.max_histories {
                state.history.drain(0..2);
            }
        }
        return response;
    }
}

impl AiChatPlugin {
    async fn on_private_message(&self, msg: Arc<PrivateMessageEvent>, act: Arc<ActionManager>) {
        let now_time = (Utc::now() + chrono::Duration::hours(8))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let response = self
            .chat(
                msg.sender.user_id.to_string(),
                rig::message::Message::user(format!("[{}] {}", now_time, msg.raw_message.clone())),
                format!("对方名称:{}", msg.sender.nickname),
            )
            .await;
        let mut chat_count = self.chat_count.lock().await;
        *chat_count += 1;
        if *chat_count == 2 {
            *chat_count = 0;
            self.change_mood(msg.sender.user_id.clone(), msg.raw_message.clone())
                .await
                .unwrap_or_else(|_| tracing::warn!("[Ai Plugin] Change Mood Error"));
        }
        for text in response.split(';') {
            let _ = act
                .send_private_message(
                    msg.sender.user_id,
                    Message::new().with_text(text.to_string()),
                )
                .await;
        }
    }
}
#[async_trait]
impl BasePlugin for AiChatPlugin {
    async fn on_load(self: Arc<Self>) {}
    async fn on_update(self: Arc<Self>, event_nexus: Arc<EventNexus>, act: Arc<ActionManager>) {
        let private_port = event_nexus.get_private_message_port();
        let heartbeat_port = event_nexus.get_heartbeat_port();
        tokio::select! {
            Ok(private_message) = private_port.recv() => {
                if !private_message.clone().raw_message.starts_with("/") && !self.token.is_empty() {
                    let msg = private_message.clone();
                    let action = act.clone();
                    let sf = self.clone();
                    tokio::spawn(async move {
                    sf.on_private_message(msg, action).await;
                    });
                }
                let mood_state: AiMoodState;
                {
                    mood_state = self.mood_state.lock().await.clone();
                }
                if private_message.raw_message.starts_with("/mood") {
                    let _ = act
                        .send_private_message(
                            private_message.sender.user_id.clone(),
                            Message::new().with_text(format!(
                                "[Mood]\npleasure: {}\naeousul: {}\ndominance: {}",
                                mood_state.pleasure, mood_state.arousal, mood_state.dominance
                            )),
                        )
                        .await;
                }
            },
            Ok(_) = heartbeat_port.recv() => {
                self.save_history(self.data_dir.clone()).unwrap_or_else(|_| tracing::warn!("[Ai Plugin Error] Save History Error"));
            }
        }
    }
    async fn on_unload(self: Arc<Self>) {}
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatSession {
    pub user_id: String,
    pub history: Vec<rig::message::Message>,
}

impl ChatSession {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            history: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Personality {
    core: String,
    context_scenario: String,
    format_rules: String,
    mood_patches: MoodPatChes,
}

impl Personality {
    fn get_prompt(&self, mood_state: AiMoodState) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.core,
            self.context_scenario,
            self.format_rules,
            self.mood_patches.select_patch(mood_state)
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct MoodPatChes {
    neutral: String,
    depressed: String,
    bored: String,
    happy: String,
    excited: String,
    shy: String,
    angry: String,
    arrogant: String,
}

impl MoodPatChes {
    pub fn select_patch(&self, state: AiMoodState) -> &str {
        // 1. angry
        if state.pleasure < -50 && state.arousal > 30 {
            return &self.angry;
        }

        // 2. 判定积极情绪：激动 vs 开心
        if state.pleasure > 40 {
            if state.dominance < -30 {
                return &self.shy;
            } // 开心但支配度低 = 羞涩
            if state.arousal > 50 {
                return &self.excited;
            }
            return &self.happy;
        }

        // 3. 判定消极/低能量情绪：沮丧 vs 无聊
        if state.pleasure < -30 {
            if state.arousal < -20 {
                return &self.depressed;
            }
            return &self.arrogant; // 不爽但有精神 = 傲慢地怼人
        }

        // 4. 判定激活度：无聊
        if state.arousal < -50 {
            return &self.bored;
        }

        // 5. 默认状态
        &self.neutral
    }
}

#[derive(Serialize, Deserialize, Clone, JsonSchema)]
struct AiMoodState {
    pleasure: i8,
    arousal: i8,
    dominance: i8,
}
