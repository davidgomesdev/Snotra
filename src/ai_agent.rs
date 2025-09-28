use async_trait::async_trait;
use chatgpt::client::ChatGPT;
use mockall::automock;
use std::any::Any;
use tracing::{debug, error, info};

pub struct AIAgent<L: LLM> {
    llm: L,
}

impl<L: LLM> AIAgent<L> {
    pub fn new(llm: L) -> Self {
        Self { llm }
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn validate_phrase_translation(&self, german: &str, english: &str) -> Option<String>;
    async fn ask_word_difference(&self, first: &str, second: &str) -> Option<String>;
}

#[automock]
#[async_trait]
impl<L: LLM> Agent for AIAgent<L> {
    async fn validate_phrase_translation(&self, german: &str, english: &str) -> Option<String> {
        info!(
            "Querying ChatGPT for saying '{}' with '{}'",
            english, german
        );

        self.query_llm(format!(
            "In German, is '{german}' the right way to say '{english}'? If not, explain why and mark the differences in bold.",
        )).await
    }

    async fn ask_word_difference(&self, first: &str, second: &str) -> Option<String> {
        info!(
            "Querying ChatGPT for difference between '{}' and '{}'",
            first, second
        );

        self.query_llm(format!(
            "In German, what is the difference between '{first}' and '{second}'?"
        ))
        .await
    }
}

impl<L: LLM> AIAgent<L> {
    pub async fn query_llm(&self, query: String) -> Option<String> {
        debug!("Sending query {}", query);

        let result = self.llm.send_message(query).await;

        if let Err(error) = result {
            error!("Failed sending the query to ChatGPT. Error: {:?}", error);
            return None;
        }

        let response = result.unwrap();

        debug!(
            "Query finished with model '{}'. Response was: '{}'",
            response.model, response.content
        );

        Some(response.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::eq};

    #[test_log::test(tokio::test)]
    async fn when_validating_a_german_phrase_to_english_should_send_the_right_query() {
        let mut llm_mock = MockLLM::new();
        llm_mock
            .expect_send_message()
            .with(eq(
                "In German, is 'diese wort' the right way to say 'this word'? If not, explain why and mark the differences in bold."
                    .to_string(),
            ))
            .return_once(move |_| {
                Ok(LLMResponse::new(
                    "mock".to_string(),
                    "Yes, you are right!".to_string(),
                ))
            });

        let agent = AIAgent::new(llm_mock);
        let response = agent
            .validate_phrase_translation("diese wort", "this word")
            .await
            .expect("Failed!");

        assert_eq!("Yes, you are right!".to_string(), response);
    }

    #[test_log::test(tokio::test)]
    async fn when_asking_difference_of_two_words_should_send_the_right_query() {
        let mut llm_mock = MockLLM::new();
        llm_mock
            .expect_send_message()
            .with(eq(
                "In German, what is the difference between 'etwas' and 'sache'?".to_string(),
            ))
            .return_once(move |_| {
                Ok(LLMResponse::new(
                    "mock".to_string(),
                    "Etwas is something and sache is thing".to_string(),
                ))
            });

        let agent = AIAgent::new(llm_mock);
        let response = agent
            .ask_word_difference("etwas", "sache")
            .await
            .expect("Failed!");

        assert_eq!(
            "Etwas is something and sache is thing".to_string(),
            response
        );
    }

    #[test_log::test(tokio::test)]
    async fn when_translation_query_fails_should_return_none() {
        let mut llm_mock = MockLLM::new();
        llm_mock
            .expect_send_message()
            .return_once(move |_| Err(Box::new(())));

        let agent = AIAgent::new(llm_mock);
        assert_eq!(
            agent
                .validate_phrase_translation("etwas", "something")
                .await,
            None
        );
    }

    #[test_log::test(tokio::test)]
    async fn when_difference_query_fails_should_return_none() {
        let mut llm_mock = MockLLM::new();
        llm_mock
            .expect_send_message()
            .return_once(move |_| Err(Box::new(())));

        let agent = AIAgent::new(llm_mock);
        assert_eq!(agent.ask_word_difference("etwas", "sache").await, None);
    }

    mock! {
        LLM {}
        #[async_trait]
        impl LLM for LLM {
             async fn send_message(
                &self,
                message: String,
            ) -> Result<LLMResponse, Box<dyn Any + Send + Sync>> ;
        }
    }
}

#[async_trait]
pub trait LLM: Send + Sync {
    async fn send_message(
        &self,
        message: String,
    ) -> Result<LLMResponse, Box<dyn Any + Send + Sync>>;
}

pub struct LLMResponse {
    pub model: String,
    pub content: String,
}

impl LLMResponse {
    pub fn new(model: String, content: String) -> Self {
        Self { model, content }
    }
}

pub struct ChatGPTLLM {
    chat_gpt: ChatGPT,
}

impl ChatGPTLLM {
    pub fn new(chat_gpt: ChatGPT) -> Self {
        Self { chat_gpt }
    }
}

#[async_trait]
#[automock]
impl LLM for ChatGPTLLM {
    async fn send_message(
        &self,
        message: String,
    ) -> Result<LLMResponse, Box<dyn Any + Send + Sync>> {
        match self.chat_gpt.send_message(message).await {
            Ok(mut response) => Ok(LLMResponse::new(
                response.model,
                response.message_choices.pop().unwrap().message.content,
            )),
            Err(error) => Err(Box::new(error)),
        }
    }
}
