use crate::ai_agent::Agent;
use serenity::all::{Context, EventHandler, Message};
use serenity::async_trait;
use std::fmt::Display;
use tracing::{debug, error, info_span, trace, warn};

pub struct Bot<A: Agent> {
    ai_agent: A,
    allowed_users: Vec<String>,
}

impl<A: Agent> Bot<A> {
    pub fn new(ai_agent: A, allowed_users: String) -> Bot<A> {
        let allowed_users = allowed_users.split(",").map(|v| v.to_string()).collect();

        Bot {
            ai_agent,
            allowed_users,
        }
    }
}

#[async_trait]
impl<A: Agent> EventHandler for Bot<A> {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.guild_id.is_some() {
            debug!("Received message from server, ignoring (as I only respond to DMs)");
            return;
        }

        if !self.is_author_allowed(msg.author.bot, &msg.author.name) {
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

        let (german, english) =
            get_german_and_english_parts(message).expect("Should not have landed here.");

        let query_span = info_span!("chatgpt_query");
        let _query_guard = query_span.enter();

        let response = match self
            .ai_agent
            .validate_phrase_translation(german, english)
            .await
        {
            Some(value) => value,
            None => {
                trace_error(
                    msg.reply(ctx.http, "There was a problem querying ChatGPT.")
                        .await,
                    "Failed to send error response reply",
                );
                return;
            }
        };

        trace_error(
            msg.reply(ctx.http, response).await,
            "Failed to send response reply",
        );
    }
}

impl<A: Agent> Bot<A> {
    fn is_author_allowed(&self, is_author_bot: bool, author_name: &str) -> bool {
        if is_author_bot
            || !self
                .allowed_users
                .iter()
                .any(|allowed_user| allowed_user == author_name)
        {
            return false;
        }
        true
    }
}

fn get_german_and_english_parts(message: &str) -> Option<(&str, &str)> {
    let message: Vec<&str> = message.split("\n").collect();

    if message.len() < 2 {
        trace!("Message does not have at least 2 parts");
        return None;
    }

    if message.len() > 2 {
        warn!("Message has more than 2 parts, ignoring the extra");
    }

    let german = message[0];
    let english = message[1];

    Some((german, english))
}

pub fn trace_error<T, E: Display>(res: Result<T, E>, error_message: &str) {
    if let Err(error) = res {
        error!("{}. Error: {}", error_message, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_agent::{MockAIAgent, MockChatGPTLLM};

    #[test_log::test(tokio::test)]
    async fn when_author_is_a_bot_should_not_be_allowed() {
        let bot: Bot<MockAIAgent<MockChatGPTLLM>> =
            Bot::new(MockAIAgent::<MockChatGPTLLM>::new(), "".to_string());

        bot.is_author_allowed(true, "n/a");
    }

    #[test_log::test(tokio::test)]
    async fn when_author_is_not_in_allowed_list_should_not_be_allowed() {
        let bot: Bot<MockAIAgent<MockChatGPTLLM>> = Bot::new(
            MockAIAgent::<MockChatGPTLLM>::new(),
            "juff,ceff".to_string(),
        );

        bot.is_author_allowed(true, "cov");
    }

    #[test_log::test(tokio::test)]
    async fn when_author_is_in_allowed_list_should_be_allowed() {
        let bot: Bot<MockAIAgent<MockChatGPTLLM>> = Bot::new(
            MockAIAgent::<MockChatGPTLLM>::new(),
            "jeff,caff".to_string(),
        );

        bot.is_author_allowed(true, "caff");
    }

    #[test_log::test(tokio::test)]
    async fn when_message_has_two_parts_should_get_german_then_english() {
        let (german, english) = get_german_and_english_parts("etwas\nsomething").unwrap();

        assert_eq!(german, "etwas");
        assert_eq!(english, "something");
    }

    #[test_log::test(tokio::test)]
    async fn when_message_has_more_than_two_parts_should_ignore_extra() {
        let (german, english) = get_german_and_english_parts("etwas\nsomething\nmore").unwrap();

        assert_eq!(german, "etwas");
        assert_eq!(english, "something");
    }
}
