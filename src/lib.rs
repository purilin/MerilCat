pub mod bot;
pub mod config;
pub mod core;
pub mod utils;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {

    use crate::core::api::NapcatApi;
    use crate::core::event::EventManager;
    use crate::utils::parser::event_parser::message_event::{
        //GroupMessageEvent,
        PrivateMessageEvent,
    };
    use crate::utils::parser::message_parser::Messages;
    use crate::utils::parser::request_parser::PrivateMessage;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let bot = bot::MerilBot::init();
        let api = NapcatApi::get();
        let event = EventManager::get();
        event
            .reg_private_event(move |msg: PrivateMessageEvent| async move {
                if msg.raw_message == "hello" {
                    let new_msg = Messages::new().with_text("hnm");
                    let stv = PrivateMessage::new(123, new_msg.clone());
                    let stvv = serde_json::to_string(&stv).unwrap();
                    println!("{}", stvv);
                    api.send_private_message(msg.sender.user_id, new_msg).await;
                }
            })
            .await;
        let bot_arc = bot.clone();
        bot_arc.run().await;
    }
}
