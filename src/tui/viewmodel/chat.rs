use lliminal::llm::{Message, UserMessageContent, UserMessagePart};

pub struct Chat {
    pub messages: Vec<Message>,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            messages: vec![],
        }
    }
}

impl Chat {
    pub fn submit_user_input(&mut self, input: &str) {
        self.messages.push(
            Message::User { parts: vec![
                UserMessagePart {
                    content: UserMessageContent::Text { text: input.to_string() }
                }
            ]}
        );
    }
}
