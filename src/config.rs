use getset::{CloneGetters, Getters, Setters};
use std::{collections::HashMap, sync::OnceLock};
use tokio::sync::Mutex;
pub static GLOBALSTATE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

#[derive(Getters, CloneGetters, Setters)]
pub struct Config {
    #[getset(get = "pub", set = "pub")]
    bot_id: i64,
    #[getset(get = "pub", set = "pub")]
    root_id: i64,
    #[getset(get = "pub", set = "pub")]
    websocket_addr: String,
    #[getset(get = "pub", set = "pub")]
    http_addr: String,
    #[getset(get = "pub", set = "pub")]
    napcat_webui_token: String,
    #[getset(get = "pub", set = "pub")]
    napcat_websocket_token: String,
    #[getset(get = "pub", set = "pub")]
    napcat_http_token: String,
}

static INSTANCE: OnceLock<Config> = OnceLock::new();
impl Config {
    fn new() -> Self {
        Self {
            bot_id: 0,
            root_id: 0,
            websocket_addr: "0.0.0.0:3000".into(),
            http_addr: "0.0.0.0:3001".into(),
            napcat_webui_token: "".into(),
            napcat_websocket_token: "".into(),
            napcat_http_token: "".into(),
        }
    }

    pub fn get_or_init() -> &'static Self {
        INSTANCE.get_or_init(Self::new)
    }
}
