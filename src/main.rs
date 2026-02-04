use meril_cat::prelude::{MerilBot, Message};
use meril_cat::prelude::{Plugin, Trigger};
#[tokio::main]
async fn main() {
    let bot = MerilBot::new();
    bot.plugin
        .clone()
        .add_plugin(
            Plugin::new()
                .with_name("SendLike")
                .with_description("/like")
                .with_trigger(Trigger::StartWith("/like".into()))
                .with_on_private_message_func(|msg, act| async move {
                    let like_msg = act.send_like(msg.sender.user_id, 10).await;
                    act.send_private_message(
                        msg.sender.user_id,
                        Message::new().with_text(like_msg),
                    )
                    .await;
                }),
        )
        .await;
    bot.run().await;
}
