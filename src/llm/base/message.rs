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
    Assistant { parts: Vec<AssistantMessagePart> },
}

/// A part of a user message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserMessagePart {
    /// The content of the message part
    pub content: UserMessageContent
}

/// The content of a user message part
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserMessageContent {
    Text { text: String }
}

/// A part of an assistant message
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistantMessagePart {
    /// A flag which indicates whether the part is complete or being generated
    pub complete: bool,

    /// The content of an assistant message
    pub content: AssistantMessageContent
}

/// The content of an assistant message part
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistantMessageContent {
    Text { text: String }
}

