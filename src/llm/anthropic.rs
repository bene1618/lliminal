use std::vec::IntoIter;

use crate::llm::AssistantMessageContent;

use super::{AssistantMessagePart, CompletionRequest, LlmClient, UserMessagePart};
use futures::stream::{self, Iter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

const API_VERSION: &str = "2023-06-01";

pub struct AnthropicLlmClient {
    pub config: AnthropicLlmClientConfig
}

pub struct AnthropicLlmClientConfig {
    pub base_url: Url,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<MessageContent>
}

impl LlmClient for AnthropicLlmClient {
    type Response = Iter<IntoIter<super::Result<Vec<super::Message>>>>;

    async fn complete(&mut self, request: &CompletionRequest) -> Self::Response {
        const PATH: &str = "/v1/messages";
        let request = MessagesRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            system: request.system.iter().map(|prompt| prompt.into()).collect(),
            messages: request.messages.iter().map(|message| message.into()).collect(),
        };
        let client = reqwest::Client::new();
        let url = self.config.base_url.join(PATH).unwrap();
        let response: MessagesResponse = client.post(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&request)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let content = response.content.iter().filter_map(|c| {
            if let MessageContent::Text { text } = c {
                Some(text.to_owned())
            } else {
                None
            }
        }).reduce(|acc, e| acc + &e).unwrap();
        stream::iter(vec![
            Ok(vec![super::Message::Assistant {
                parts: vec![super::AssistantMessagePart {
                    complete: true,
                    content: AssistantMessageContent::Text { text: content }
                }]
            }])
        ])
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    system: Vec<SystemPrompt>,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: MessageRole,
    content: Vec<MessageContent>
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum MessageRole {
    User,
    Assistant
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageContent {
    Text { text: String },
    Image { source: ImageSource },
    Document { source: DocumentSource },
    ToolUse { id: String, input: Value, name: String },
    ToolResult { tool_use_id: String, content: String, is_error: bool }
}

#[derive(Serialize, Deserialize)]
struct ImageSource {
    data: String,
    media_type: String,
    #[serde(rename = "type")] encoding_type: String
}

#[derive(Serialize, Deserialize)]
struct DocumentSource {
    data: String,
    media_type: String,
    #[serde(rename = "type")] encoding_type: String
}

#[derive(Serialize)]
struct SystemPrompt {
    text: String,
    #[serde(rename = "type")] encoding_type: String
}

impl Message {
    fn from_user_message_parts(parts: &Vec<UserMessagePart>) -> Self {
        Message {
            role: MessageRole::User,
            content: parts.iter().cloned().map(|p| match p.content {
                super::UserMessageContent::Text { text } => MessageContent::Text { text }
            }).collect()
        }
    }

    fn from_assistant_message_parts(parts: &Vec<AssistantMessagePart>) -> Self {
        Message {
            role: MessageRole::Assistant,
            content: parts.iter().cloned().filter_map(|p| match p {
                AssistantMessagePart { complete: false, content: _ } => None,
                AssistantMessagePart {
                    complete: true,
                    content: AssistantMessageContent::Text { text }
                } => Some(MessageContent::Text { text })
            }).collect()
        }
    }
}

impl From<&super::Message> for Message {
    fn from(value: &super::Message) -> Self {
        match value {
            super::Message::User { parts } => Message::from_user_message_parts(parts),
            super::Message::Assistant { parts } => Message::from_assistant_message_parts(parts),
        }
    }
}

impl From<&super::SystemPrompt> for SystemPrompt {
    fn from(value: &super::SystemPrompt) -> Self {
        SystemPrompt { text: value.content.clone(), encoding_type: "text".to_string() }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use mockito::Matcher;
    use url::Url;

    use crate::llm::{AssistantMessageContent, AssistantMessagePart, LlmClient, Message, SystemPrompt, UserMessageContent, UserMessagePart};

    use super::AnthropicLlmClient;

    #[tokio::test]
    async fn test_completion() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mut anthropic_client = AnthropicLlmClient {
            config: super::AnthropicLlmClientConfig { base_url: Url::parse(&url).unwrap(), api_key: "test".to_string(), model: "model".to_string(), max_tokens: 1024 },
        };
        let request = crate::llm::CompletionRequest {
            system: vec![
                SystemPrompt { content: "Answer in some way".to_string() }
            ],
            messages: vec![
                Message::User { parts: vec![
                    UserMessagePart { content: UserMessageContent::Text { text: "Part 1".to_string() } },
                ] },
                Message::Assistant { parts: vec![
                    AssistantMessagePart {
                        complete: true,
                        content: AssistantMessageContent::Text { text: "Response".to_string() },
                    }
                ] },
                Message::User { parts: vec![
                    UserMessagePart { content: UserMessageContent::Text { text: "Part 2".to_string() } },
                ] }
            ]
        };

        let mock = server.mock("POST", "/v1/messages")
            .match_body(Matcher::JsonString(r#"
{
  "model": "model",
  "max_tokens": 1024,
  "system": [
    {
      "text": "Answer in some way",
      "type": "text"
    }
  ],
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "Part 1"
        }
      ]
    },
    {
      "role": "assistant",
      "content": [
        {
          "type": "text",
          "text": "Response"
        }
      ]
    },
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "Part 2"
        }
      ]
    }
  ]
}
            "#.to_string()))
            .with_status(200)
            .with_body(r#"{"content": [{"type": "text", "text": "My response"}]}"#)
            .create();

        let result = anthropic_client.complete(&request).await.next().await.expect("Did not contain response");

        mock.assert();

        assert_eq!(*result.unwrap().first().unwrap(), Message::Assistant { parts: vec![
            AssistantMessagePart { complete: true, content: AssistantMessageContent::Text { text: "My response".to_string() } }
        ] });
    }
}
