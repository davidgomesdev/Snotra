use chatgpt::client::ChatGPT;
use chatgpt::types::CompletionResponse;
use serenity::all::{Context, CurrentUser, EventHandler, GatewayIntents, Message, Settings};
use serenity::{Client, async_trait};
use std::env;

#[tokio::main]
async fn main() {
    let openai_token = env::var("OPENAI_TOKEN").expect("Missing OPENAI_TOKEN env var!");
    let discord_token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN env var!");
    let discord_allowed_users = env::var("DISCORD_ALLOWED_USERNAMES")
        .expect("Missing DISCORD_ALLOWED_USERNAMES env var! (comma-separated)");

    let chatgpt = ChatGPT::new(openai_token).expect("Failed to connect to ChatGPT");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut discord = Client::builder(discord_token, intents)
        .event_handler(Bot::new(chatgpt, discord_allowed_users))
        .await
        .expect("Error creating discord client");

    discord
        .start()
        .await
        .expect("Failed to start discord client");
}

struct Bot {
    chatgpt: ChatGPT,
    allowed_users: Vec<String>,
}

impl Bot {
    fn new(chatgpt: ChatGPT, allowed_users: String) -> Bot {
        let allowed_users = allowed_users.split(",").map(|v| v.to_string()).collect();

        Bot {
            chatgpt,
            allowed_users,
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !self.allowed_users.contains(&msg.author.name) {
            return;
        }

        msg.reply(ctx.http, "Olá!")
            .await
            .expect("Failed to reply to user");
    }
}
