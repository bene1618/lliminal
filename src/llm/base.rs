use bytes::Bytes;

pub type Result<T> = std::result::Result<T, LlmError>;

/// Generic trait to interact with an LLM
pub trait LlmClient {
    /// Send a prompt to the LLM and get a response
    async fn complete(&mut self, request: CompletionRequest) -> Result<CompletionResponse>;
}

/// The request which contains all information to generate a text completion
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionRequest {
    /// The system prompts
    pub system: Vec<SystemPrompt>,

    /// The previous messages
    pub messages: Vec<Message>,
}

/// A system message, which the model should follow regardless of the other messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemPrompt {
    pub content: String
}

/// A message in a chat which should be completed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A user message, which might contain of different parts to support multimodality
    User { parts: Vec<UserMessagePart> },
    /// An assistant message, which contains a text response and/or requests for tool calls
    Assistant(AssistantMessage),
    /// The response from a tool call
    Tool {
        content: String,
        tool_call_id: String
    }
}

/// A message from the assistant
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>
}

/// A part of a user message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserMessagePart {
    Text(String),
    Image { data: Bytes, media_type: ImageMediaType },
    Audio { data: Bytes, media_type: AudioMediaType },
    File { data: Bytes, media_type: FileMediaType }
}

/// The media type for an image representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageMediaType {
    JPEG, PNG, GIF, WEBP
}

/// The media type for an audio representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioMediaType {
    WAV, MP3
}

/// The media type for a file representation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileMediaType {
    PlainText, PDF
}

/// A tool call allows calling an external tool like an MCP server for retrieving more information
/// or for conducting actions on behalf of the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCall {
    pub id: String,
    pub function_name: String,
    pub function_args_json: String
}

/// The response from the LLM tool
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionResponse {
    pub message: AssistantMessage
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LlmError {
}

#[cfg(test)]
pub struct TestLlmClient {
    pub requests: Vec<CompletionRequest>,
    pub default_response: Result<CompletionResponse>,
    pub response_factory: Option<fn(CompletionRequest) -> Result<CompletionResponse>>
}

#[cfg(test)]
impl LlmClient for TestLlmClient {
    async fn complete(&mut self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.requests.push(request.clone());
        match self.response_factory {
            Some(factory) => factory(request),
            None => self.default_response.clone()
        }
    }
}

#[cfg(test)]
impl Default for TestLlmClient {
    fn default() -> Self {
        Self {
            requests: Vec::new(),
            default_response: Ok(CompletionResponse { message: AssistantMessage { content: Some("".to_string()), tool_calls: Vec::new() } }),
            response_factory: None
        }
    }
}

#[cfg(test)]
impl TestLlmClient {
    fn set_response_message(&mut self, message: String) {
        self.default_response = Ok(CompletionResponse { message: AssistantMessage { content: Some(message), tool_calls: Vec::new() } });
    }

    fn set_response_error(&mut self, error: LlmError) {
        self.default_response = Err(error);
    }
}
