mod message;

use futures::Stream;
pub use message::*;

pub type Result<T> = std::result::Result<T, LlmError>;

/// Type for error conditions on completing a request
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LlmError {
    ConnectionError
}

/// Generic trait to interact with an LLM
pub trait LlmClient {
    type Response : Stream<Item = Result<Vec<Message>>>;

    /// Send a prompt to the LLM and get a response
    fn complete(&mut self, request: &CompletionRequest) -> impl Future<Output = Self::Response>;
}

/// The request which contains all information to generate a text completion
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionRequest {
    /// The system prompts
    pub system: Vec<SystemPrompt>,

    /// The previous messages
    pub messages: Vec<Message>,
}

#[cfg(test)]
mod test {
    use std::vec::IntoIter;

    use futures::stream::{self, Iter};
    use tokio_stream::StreamExt;

    use super::{message::{AssistantMessageContent, AssistantMessagePart}, *};

    pub struct TestLlmClient {
        pub requests: Vec<CompletionRequest>,
        default_response: Result<Vec<Message>>,
        response_factory: Option<fn(CompletionRequest) -> Vec<Result<Vec<Message>>>>
    }

    #[cfg(test)]
    impl LlmClient for TestLlmClient {
        type Response = Iter<IntoIter<Result<Vec<Message>>>>;

        async fn complete(&mut self, request: &CompletionRequest) -> Self::Response {
            self.requests.push(request.clone());
            match self.response_factory {
                Some(factory) => stream::iter(factory(request.clone())),
                None => stream::iter(vec![self.default_response.clone()])
            }
        }
    }

    #[cfg(test)]
    impl Default for TestLlmClient {
        fn default() -> Self {
            Self {
                requests: Vec::new(),
                default_response: TestLlmClient::response_for_message("".to_string()),
                response_factory: None
            }
        }
    }

    #[cfg(test)]
    impl TestLlmClient {
        fn set_response_message(&mut self, message: String) {
            self.default_response = TestLlmClient::response_for_message(message);
        }
    
        fn set_response_error(&mut self, error: LlmError) {
            self.default_response = Err(error);
        }

        fn response_for_message(message: String) -> Result<Vec<Message>> {
            Ok(vec![Message::Assistant {
                parts: vec![AssistantMessagePart {
                    complete: true, content: AssistantMessageContent::Text { text: message }
                }]
            }])
        }
    }

    #[tokio::test]
    async fn use_test_client() {
        let mut client = TestLlmClient::default();
        let completion_request = CompletionRequest {
            system: vec![],
            messages: vec![],
        };

        let mut response = client.complete(&completion_request).await;
        assert_response(response.next().await, "");
        assert_eq!(response.next().await, None);

        let new_message = "New message";
        client.set_response_message(new_message.to_string());
        response = client.complete(&completion_request).await;
        assert_response(response.next().await, new_message);
        assert_eq!(response.next().await, None);

        let error = LlmError::ConnectionError;
        client.set_response_error(error.clone());
        response = client.complete(&completion_request).await;
        assert_eq!(response.next().await, Some(Err(error)));
        assert_eq!(response.next().await, None);
    }

    fn assert_response(response: Option<Result<Vec<Message>>>, expected_text: &str) {
        assert!(response.is_some());
        assert!(response.as_ref().unwrap().is_ok());
        assert_eq!(response.as_ref().unwrap().as_ref().unwrap().len(), 1);
        let message = response.as_ref().unwrap().as_ref().unwrap().first().unwrap();
        match message {
            Message::Assistant { parts } => {
                assert_eq!(parts.len(), 1);
                assert_eq!(
                    *parts.first().unwrap(),
                    AssistantMessagePart { complete: true, content: AssistantMessageContent::Text { text: expected_text.to_string() } }
                );
            },
            _ => panic!("Not an assistant message")
        }
    }
}

