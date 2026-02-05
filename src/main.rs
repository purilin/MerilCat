use meril_cat::prelude::MerilBot;
#[tokio::main]
async fn main() {
    let bot = MerilBot::new();
    bot.run().await;
}
