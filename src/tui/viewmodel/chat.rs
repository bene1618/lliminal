use lliminal::llm::{Message, UserMessageContent, UserMessagePart};

#[derive(Clone, Debug)]
pub struct Chat {
    pub messages: Vec<Message>,
    pub user_input: bool,
    pub scroll: usize
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            messages: vec![],
            user_input: true,
            scroll: 0
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

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_add(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll = self.scroll.saturating_sub(amount);
    }
}
