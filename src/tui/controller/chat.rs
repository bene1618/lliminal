use std::env;

use lliminal::llm::{anthropic::{AnthropicLlmClient, AnthropicLlmClientConfig}, CompletionRequest, LlmClient};
use tokio::sync::{mpsc, watch};
use tokio_stream::StreamExt;
use tui_input::Input;
use url::Url;

use crate::tui::viewmodel::Chat;

use super::Controller;

pub struct ChatController {
    pub chat: watch::Sender<Chat>,
    pub chat_input: watch::Sender<Input>,
    pub self_sender: Option<mpsc::UnboundedSender<ChatCommand>>
}

impl Controller<ChatCommand> for ChatController {
    fn handle(&self, event: ChatCommand) {
        match event {
            ChatCommand::Submit => {
                let old_input = self.chat_input.send_replace(Input::default());
                self.chat.send_modify(|chat| {
                    chat.submit_user_input(old_input.value());
                });
                let chat_sender = self.chat.clone();
                let self_sender = self.self_sender.clone();
                tokio::spawn(async move {
                    ChatController::call_llm(chat_sender, self_sender.expect("Must call launch before handling commands")).await
                });
            },
            ChatCommand::WaitForUser => {
                self.chat.send_modify(|chat| {
                    chat.wait_for_user();
                });
            }
        }
    }

    fn register_self_sender(&mut self, sender: mpsc::UnboundedSender<ChatCommand>) {
        self.self_sender.replace(sender);
    }
}

impl ChatController {
    async fn call_llm(chat: watch::Sender<Chat>, chat_controller: mpsc::UnboundedSender<ChatCommand>) {
        let mut client = AnthropicLlmClient {
            config: AnthropicLlmClientConfig {
                base_url: Url::parse(
                              &env::var("ANTHROPIC_URL").unwrap_or("https://api.anthropic.com".to_string())
                          ).expect("Invalid URL provided"),
                api_key: env::var("ANTHROPIC_API_KEY").expect("Must set ANTHROPIC_API_KEY env var"),
                model: "claude-3-5-haiku-latest".to_string(),
                max_tokens: 1024
            }
        };
        let messages = chat.borrow().messages.clone();
        let request = CompletionRequest {
            system: vec![],
            messages: messages.clone()
        };

        let mut response = client.complete(&request).await;
        while let Some(response) = response.next().await {
            let mut response = response.expect("Seems to have an error");
            let mut new_messages = messages.clone();
            new_messages.append(&mut response);
            chat.send_modify(|c| {
                c.messages = new_messages
            });
            chat_controller.send(ChatCommand::WaitForUser).unwrap();
        }
    }
}

pub enum ChatCommand {
    Submit,
    WaitForUser
}

