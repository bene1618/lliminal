use super::{CompletionRequest, CompletionResponse, LlmClient, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

const API_VERSION: &str = "2023-06-01";

pub struct AnthropicLlmClient {
    config: AnthropicLlmClientConfig
}

pub struct AnthropicLlmClientConfig {
    base_url: Url,
    api_key: String,
    model: String,
    max_tokens: u32
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<MessageContent>
}

impl LlmClient for AnthropicLlmClient {
    async fn complete(&mut self, request: CompletionRequest) -> Result<CompletionResponse> {
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
        }).reduce(|acc, e| acc + &e);
        Ok(CompletionResponse { message: super::AssistantMessage { content: content, tool_calls: vec![] } })
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

impl From<&super::Message> for Message {
    fn from(value: &super::Message) -> Self {
        match value {
            super::Message::User { parts } => Message {
                role: MessageRole::User,
                content: parts.iter().cloned().map(|p| {
                    match p {
                        super::UserMessagePart::Text(content) => MessageContent::Text {
                            text: content
                        },
                        super::UserMessagePart::Image { data, media_type } => MessageContent::Image {
                            source: ImageSource {
                                data: BASE64_STANDARD.encode(data),
                                media_type: match media_type {
                                    super::ImageMediaType::JPEG => "image/jpeg",
                                    super::ImageMediaType::PNG => "image/png",
                                    super::ImageMediaType::GIF => "image/gif",
                                    super::ImageMediaType::WEBP => "image/webp",
                                }.to_string(),
                                encoding_type: "base64".to_string()
                            }
                        },
                        super::UserMessagePart::Audio { .. } => panic!("Audio is not supported"),
                        super::UserMessagePart::File { data, media_type } => MessageContent::Document {
                            source: match media_type {
                                super::FileMediaType::PlainText => DocumentSource {
                                    data: String::from_utf8(data.to_vec()).unwrap(),
                                    media_type: "text/plain".to_string(),
                                    encoding_type: "text".to_string()
                                },
                                super::FileMediaType::PDF => DocumentSource {
                                    data: BASE64_STANDARD.encode(data),
                                    media_type: "application/pdf".to_string(),
                                    encoding_type: "base64".to_string()
                                }
                            }
                        }
                    }
                }).collect()
            },
            super::Message::Assistant(assistant_message) => Message {
                role: MessageRole::Assistant,
                content: if let Some(content) = &assistant_message.content {
                    vec![MessageContent::Text { text: content.clone() }]
                } else {
                    assistant_message.tool_calls.iter().cloned().map(|tool_call| MessageContent::ToolUse {
                        id: tool_call.id,
                        input: serde_json::from_str(&tool_call.function_args_json).unwrap(),
                        name: tool_call.function_name
                    }).collect()
                }
            },
            super::Message::Tool { content, tool_call_id } => Message {
                role: MessageRole::User,
                content: vec![MessageContent::ToolResult {
                    tool_use_id: tool_call_id.clone(),
                    content: content.clone(),
                    is_error: false
                }]
            }
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
    use bytes::Bytes;
    use mockito::Matcher;
    use url::Url;

    use crate::llm::{AssistantMessage, LlmClient, Message, SystemPrompt, ToolCall, UserMessagePart};

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
                    UserMessagePart::Text("Part 1".to_string()),
                    UserMessagePart::Image { data: Bytes::from(&b"0x120x17"[..]), media_type: crate::llm::ImageMediaType::PNG },
                    UserMessagePart::File { data: Bytes::from(&b"0xf10xeb"[..]), media_type: crate::llm::FileMediaType::PDF },
                    UserMessagePart::File { data: Bytes::from("Hello world"), media_type: crate::llm::FileMediaType::PlainText },
                ] },
                Message::Assistant(AssistantMessage {
                    content: Some("Response".to_string()),
                    tool_calls: vec![
                        ToolCall { id: "abc".to_string(), function_name: "func".to_string(), function_args_json: r#"{"key": "value"}"#.to_string() }
                    ]
                }),
                Message::Tool { content: "Tool response".to_string(), tool_call_id: "abc".to_string() },
                Message::User { parts: vec![
                    UserMessagePart::Text("Part 2".to_string())
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
        },
        {
          "type": "image",
          "source": {
            "data": "MHgxMjB4MTc=",
            "media_type": "image/png",
            "type": "base64"
          }
        },
        {
          "type": "document",
          "source": {
            "data": "MHhmMTB4ZWI=",
            "media_type": "application/pdf",
            "type": "base64"
          }
        },
        {
          "type": "document",
          "source": {
            "data": "Hello world",
            "media_type": "text/plain",
            "type": "text"
          }
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
          "type": "tool_result",
          "tool_use_id": "abc",
          "content": "Tool response",
          "is_error": false
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


        let result = anthropic_client.complete(request).await;

        mock.assert();

        assert_eq!(result.unwrap().message.content, Some("My response".to_string()));
    }
}
