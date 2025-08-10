use std::fmt::Display;
use crate::ai_agent::{AIAgent, LLM};
use serenity::all::{Context, EventHandler, Message};
use serenity::async_trait;
use tracing::{error, info_span, trace, warn};

pub struct Bot<L: LLM> {
    ai_agent: AIAgent<L>,
    allowed_users: Vec<String>,
}

impl <L:LLM> Bot<L> {
    pub fn new(ai_agent: AIAgent<L>, allowed_users: String) -> Bot<L> {
        let allowed_users = allowed_users.split(",").map(|v| v.to_string()).collect();

        Bot {
            ai_agent,
            allowed_users,
        }
    }
}

#[async_trait]
impl <L: LLM> EventHandler for Bot<L> {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !self.allowed_users.contains(&msg.author.name) {
            return;
        }

        let message = &msg.content;

        let span = info_span!("message_received");
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

        let query_span = info_span!("chatgpt_query");
        let _query_guard = query_span.enter();

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

pub fn trace_error<T, E: Display>(res: Result<T, E>, error_message: &str) {
    if let Err(error) = res {
        error!("{}. Error: {}", error_message, error);
    }
}
