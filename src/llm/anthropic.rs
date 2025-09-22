use crate::llm::{AssistantMessageContent, LlmError};

use super::{AssistantMessagePart, CompletionRequest, LlmClient, Result, UserMessagePart};
use eventsource_stream::{Event, EventStreamError, Eventsource};
use futures::{channel::mpsc, stream::IntoStream, SinkExt, Stream, StreamExt, TryStreamExt};
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

impl LlmClient for AnthropicLlmClient {
    type Response = IntoStream<mpsc::UnboundedReceiver<Result<Vec<super::Message>>>>;

    async fn complete(&mut self, request: &CompletionRequest) -> Self::Response {
        const PATH: &str = "/v1/messages";
        let request = MessagesRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            system: request.system.iter().map(Into::into).collect(),
            messages: request.messages.iter().map(Into::into).collect(),
            stream: true
        };
        let client = reqwest::Client::new();
        let url = self.config.base_url.join(PATH).expect("Cannot parse Anthropic request URL");

        let (mut sender, receiver) = mpsc::unbounded::<Result<Vec<super::Message>>>();

        match client.post(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&request)
            .send()
            .await
        {
            Ok(response) => {
                tokio::spawn(async move {
                    let response_eventsource = response.bytes_stream().eventsource();
                    handle_response(response_eventsource, sender).await;
                });
            },
            Err(_) => {
                sender.send(Err(LlmError::ConnectionError)).await.expect("Unable to send result");
            },
        }

        receiver.into_stream()
    }
}

async fn handle_response<T>(mut eventsource: T, mut sender: mpsc::UnboundedSender<Result<Vec<super::Message>>>)
    where T: Stream<Item = std::result::Result<Event, EventStreamError<reqwest::Error>>> + Unpin
{
    let mut state_holder = StreamingResponseStateHolder::new();
    while let Some(event) = eventsource.next().await {
        match event {
            Ok(event) => {
                if let Some(current_result) = state_holder.handle_event(&event.event, &event.data) {
                    sender.send(current_result).await.expect("Unable to send result");
                }
                if state_holder.is_completed() {
                    break;
                }
            },
            Err(_) => {
                sender.send(Err(LlmError::UnexpectedResponse)).await.expect("Unable to send result");
            }
        }
    }
}

struct StreamingResponseStateHolder {
    state: StreamingResponseState,
    response_parts: Vec<AssistantMessageContent>
}

enum StreamingResponseState {
    Init,
    MessageTransferring,
    ContentBlockStarted { current_content: AssistantMessageContent },
    MessageCompleted,
    ResponseCompleted
}

#[derive(Deserialize)]
struct ContentBlockDeltaEvent {
    delta: ContentBlockDelta
}

#[derive(Deserialize)]
struct ContentBlockDelta {
    text: String
}

impl StreamingResponseStateHolder {
    fn new() -> Self {
        Self { state: StreamingResponseState::Init, response_parts: vec![] }
    }

    fn handle_event(&mut self, event: &str, data: &str) -> Option<Result<Vec<super::Message>>> {
        match (&self.state, event) {
            (StreamingResponseState::Init, "message_start") => {
                self.state = StreamingResponseState::MessageTransferring;
                None
            },
            (StreamingResponseState::MessageTransferring, "content_block_start") => {
                self.state = StreamingResponseState::ContentBlockStarted {
                    current_content: AssistantMessageContent::Text { text: String::new() }
                };
                None
            },
            (StreamingResponseState::ContentBlockStarted { current_content }, "content_block_delta") => {
                if let Ok(delta_event) = serde_json::from_str::<ContentBlockDeltaEvent>(data) {
                    self.state = StreamingResponseState::ContentBlockStarted {
                        current_content: match current_content {
                            AssistantMessageContent::Text { text } => AssistantMessageContent::Text { text: text.to_owned() + &delta_event.delta.text },
                        }
                    };
                    Some(Ok(self.current_response()))
                } else {
                    Some(Err(LlmError::UnexpectedResponse))
                }
            },
            (StreamingResponseState::ContentBlockStarted { current_content }, "content_block_stop") => {
                self.response_parts.push(current_content.clone());
                self.state = StreamingResponseState::MessageTransferring;
                Some(Ok(self.current_response()))
            },
            (StreamingResponseState::MessageTransferring, "message_stop") => {
                self.state = StreamingResponseState::MessageCompleted;
                None
            },
            (StreamingResponseState::MessageTransferring, "message_delta") => {
                self.state = StreamingResponseState::ResponseCompleted;
                None
            },
            (_, "ping") => None,
            _ => Some(Err(LlmError::UnexpectedResponse))
        }
    }

    fn current_response(&self) -> Vec<super::Message> {
        let mut parts = Vec::new();
        for content in &self.response_parts {
            parts.push(AssistantMessagePart {
                complete: true,
                content: content.clone()
            });
        }
        if let StreamingResponseState::ContentBlockStarted { current_content } = &self.state {
            parts.push(AssistantMessagePart {
                complete: false,
                content: current_content.clone()
            });
        }
        vec![ super::Message::Assistant { parts } ]
    }

    fn is_completed(&self) -> bool {
        matches!(self.state, StreamingResponseState::ResponseCompleted)
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    system: Vec<SystemPrompt>,
    messages: Vec<Message>,
    stream: bool
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
    fn from_user_message_parts(parts: &[UserMessagePart]) -> Self {
        Message {
            role: MessageRole::User,
            content: parts.iter().cloned().map(|p| match p.content {
                super::UserMessageContent::Text { text } => MessageContent::Text { text }
            }).collect()
        }
    }

    fn from_assistant_message_parts(parts: &[AssistantMessagePart]) -> Self {
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
  ],
  "stream": true
}
            "#.to_string()))
            .with_status(200)
            .with_body(r#"
event: message_start
data: {"type": "message_start", "message": {"id": "msg_1nZdL29xx5MUA1yADyHTEsnR8uuvGzszyY", "type": "message", "role": "assistant", "content": [], "model": "claude-opus-4-1-20250805", "stop_reason": null, "stop_sequence": null, "usage": {"input_tokens": 25, "output_tokens": 1}}}

event: content_block_start
data: {"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}

event: ping
data: {"type": "ping"}

event: content_block_delta
data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "My"}}

event: content_block_delta
data: {"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": " response"}}

event: content_block_stop
data: {"type": "content_block_stop", "index": 0}

event: message_delta
data: {"type": "message_delta", "delta": {"stop_reason": "end_turn", "stop_sequence":null}, "usage": {"output_tokens": 15}}

event: message_stop
data: {"type": "message_stop"}
            "#)
            .create();

        let mut result = anthropic_client.complete(&request).await;

        mock.assert();

        assert_eq!(*result.next().await.unwrap().unwrap().first().unwrap(), Message::Assistant { parts: vec![
            AssistantMessagePart { complete: false, content: AssistantMessageContent::Text { text: "My".to_string() } }
        ] });

        assert_eq!(*result.next().await.unwrap().unwrap().first().unwrap(), Message::Assistant { parts: vec![
            AssistantMessagePart { complete: false, content: AssistantMessageContent::Text { text: "My response".to_string() } }
        ] });

        assert_eq!(*result.next().await.unwrap().unwrap().first().unwrap(), Message::Assistant { parts: vec![
            AssistantMessagePart { complete: true, content: AssistantMessageContent::Text { text: "My response".to_string() } }
        ] });
    }
}
