use std::env;

use futures::StreamExt;
use lliminal::llm::{anthropic::{AnthropicLlmClient, AnthropicLlmClientConfig}, CompletionRequest, LlmClient, Message, SystemPrompt, UserMessagePart};
use url::Url;

#[tokio::main]
async fn main() {
    env_logger::init();

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
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    println!("Sending completion request...");
    println!("");

    let request = CompletionRequest {
        system: vec![
            SystemPrompt { content: "You're a world-class poet. Answer in rhymes.".to_string() }
        ],
        messages: vec![
            Message::User { parts: vec![
                UserMessagePart { content: lliminal::llm::UserMessageContent::Text { text: buffer } }
            ] }
        ]
    };

    let mut response = client.complete(&request).await;
    while let Some(response) = response.next().await {
        let response = response.expect("Seems to have an error");
        println!("");
        println!("=== (Partial) Response ===");
        println!("");
        for message in response {
            print_message(&message);
        }
    }
}

fn print_message(message: &Message) {
    match message {
        lliminal::llm::Message::User { parts } => {
            for part in parts {
                match part.content.clone() {
                    lliminal::llm::UserMessageContent::Text { text } => println!("User: {}", text),
                }
            }
        },
        lliminal::llm::Message::Assistant { parts } => {
            for part in parts {
                match part.content.clone() {
                    lliminal::llm::AssistantMessageContent::Text { text } => println!("Assistant: {}", text),
                }
            }
        }
    }
}
