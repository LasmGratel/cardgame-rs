pub struct ChatMessages {
    pub messages: Vec<ChatMessage>
}

impl ChatMessages {
    pub fn new() -> Self {
        ChatMessages {
            messages: vec![]
        }
    }
    pub fn add(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }
}

pub enum ChatMessage {
    SystemMessage(String),
    UserMessage(String),
    GameMessage(String)
}

impl ChatMessage {
    pub fn get_author(&self) -> String {
        match self {
            ChatMessage::SystemMessage(_) => "SYSTEM".to_string(),
            ChatMessage::UserMessage(_) => "USER".to_string(),
            _ => "OTHER".to_string()
        }
    }

    pub fn get_text(&self) -> String {
        match self {
            ChatMessage::SystemMessage(msg) => msg.clone(),
            ChatMessage::UserMessage(msg) => msg.clone(),
            ChatMessage::GameMessage(msg) => msg.clone(),
        }
    }
}