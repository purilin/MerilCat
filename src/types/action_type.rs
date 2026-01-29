use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Serialize, Deserialize, Clone)]

pub struct NapcatRequestData {
    action: String,
    echo: String,
    params: Value,
}

impl NapcatRequestData {
    pub fn new() -> Self {
        Self {
            action: String::new(),
            echo: String::new(),
            params: Value::Null,
        }
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = action.into();
        self
    }

    pub fn with_echo(mut self, text: impl Into<String>) -> Self {
        self.echo = text.into();
        self
    }

    pub fn with_params(mut self, data: impl Into<Value>) -> Self {
        self.params = data.into();
        self
    }
}

impl Default for NapcatRequestData {
    fn default() -> Self {
        Self::new()
    }
}
