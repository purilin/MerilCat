use std::{collections::HashMap, sync::OnceLock};

use tokio::sync::Mutex;
pub static GlobalState: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
