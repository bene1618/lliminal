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
    /// The system prompt and previous messages
    pub messages: Vec<Message>
}

/// A message in a chat which should be completed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    System(SystemMessage),
    User(UserMessage),
    Assistant(AssistantMessage),
    Tool(ToolMessage)
}

/// A system message, which the model should follow regardless of the other messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemMessage {
    content: String
}

/// A user message, which might contain of different parts to support multimodality
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserMessage {
    parts: Vec<UserMessagePart>
}

/// A part of a user message
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserMessagePart {
    Text(String),
    Image(Bytes),
    Audio(Bytes),
    File(Bytes)
}

/// The message coming from the model
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMessage {
    content: Option<String>,
    tool_calls: Vec<ToolCall>
}

/// A tool call allows calling an external tool like an MCP server for retrieving more information
/// or for conducting actions on behalf of the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCall {
    id: String,
    function_name: String,
    function_args_json: String
}

/// The message of a tool in response to a tool use assistant message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolMessage {
    content: String,
    tool_call_id: String
}

/// The response from the LLM tool
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionResponse {
    message: AssistantMessage
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
