use crate::ai_agent::AIAgent;
use crate::trace_error;
use serenity::all::{Context, EventHandler, Message};
use serenity::async_trait;
use tracing::{info_span, trace, warn};

pub struct Bot {
    ai_agent: AIAgent,
    allowed_users: Vec<String>,
}

impl Bot {
    pub fn new(ai_agent: AIAgent, allowed_users: String) -> Bot {
        let allowed_users = allowed_users.split(",").map(|v| v.to_string()).collect();

        Bot {
            ai_agent,
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

        let message = &msg.content;

        let span = info_span!("chatgpt_query");
        let _guard = span.enter();

        if !message.contains("\n") {
            trace_error(
                msg.reply(
                    &ctx.http,
                    "The message needs to be separated by a new line, like so:\n<English>\n<Geman>",
                )
                .await,
                "Failed to send format message",
            )
        }

        let mut message: Vec<&str> = message.split("\n").collect();

        if message.len() < 2 {
            trace!("Message does not have at least 2 parts");
            return;
        }

        if message.len() > 2 {
            warn!("Message has more than 2 parts, ignoring the extra");
        }

        let english = message.pop().unwrap();
        let german = message.pop().unwrap();

        let response = match self.ai_agent.query_chatgpt(english, german).await {
            Some(value) => value,
            None => return,
        };

        trace_error(
            msg.reply(ctx.http, response).await,
            "Failed to send response reply",
        );
    }
}
