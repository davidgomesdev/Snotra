use chatgpt::client::ChatGPT;
use tracing::{error, info};

pub struct AIAgent {
    chatgpt: ChatGPT
}

impl AIAgent {
    pub fn new(chatgpt: ChatGPT) -> Self {
        Self { chatgpt }
    }

    pub async fn query_chatgpt(&self, english: &str, german: &str) -> Option<String> {
        info!(
                "Querying ChatGPT for saying '{}' with '{}'",
                english, german
            );

        let result = self.chatgpt
            .send_message(format!(
                "In German, is '{}' the right way to say '{}'?",
                german, english
            ))
            .await;

        if let Err(error) = result {
            error!("Failed sending the query to ChatGPT. Error: {}", error);
            return None
        }

        let mut result = result.unwrap();
        let response = result.message_choices.pop().unwrap().message.content;

        Some(response)
    }
}
