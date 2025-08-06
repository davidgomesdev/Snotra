use snotra::ai_agent::AIAgent;
use snotra::discord::Bot;
use chatgpt::client::ChatGPT;
use serenity::all::{Context, CurrentUser, EventHandler, GatewayIntents, Message, Settings};
use serenity::{Client, async_trait};
use std::env;
use std::fmt::Display;
use tracing::{error, info, info_span, trace, warn};
use snotra::tracing::setup_loki;

#[tokio::main]
async fn main() {
    let openai_token = env::var("OPENAI_TOKEN").expect("Missing OPENAI_TOKEN env var!");
    let discord_token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN env var!");
    let discord_allowed_users = env::var("DISCORD_ALLOWED_USERNAMES")
        .expect("Missing DISCORD_ALLOWED_USERNAMES env var! (comma-separated)");

    let chatgpt = ChatGPT::new(openai_token).expect("Failed to connect to ChatGPT");

    setup_loki().await;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let ai_agent = AIAgent::new(chatgpt);
    let mut discord = Client::builder(discord_token, intents)
        .event_handler(Bot::new(ai_agent, discord_allowed_users))
        .await
        .expect("Error creating discord client");

    discord
        .start()
        .await
        .expect("Failed to start discord client");
}
