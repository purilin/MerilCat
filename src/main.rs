use meril_cat::prelude::{EventManager, MerilBot, Messages, NapcatApi};
#[tokio::main]
async fn main() {
    let bot = MerilBot::init();
    let api = NapcatApi::get();
    let event = EventManager::get();
    event
        .reg_private_event(move |msg| async move {
            let user_id = msg.sender.user_id;
            if msg.raw_message.starts_with("/t") {
                let message = Messages::new().with_text("hello world");
                api.send_private_message(user_id, message).await;
            }
        })
        .await;
    bot.run().await;
}
