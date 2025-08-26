use lliminal::llm::{Message, UserMessageContent, UserMessagePart};

#[derive(Clone, Debug)]
pub struct Chat {
    pub messages: Vec<Message>,
    pub user_input: bool
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            messages: vec![],
            user_input: true
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
        self.user_input = false;
    }

    pub fn wait_for_user(&mut self) {
        self.user_input = true;
    }
}
