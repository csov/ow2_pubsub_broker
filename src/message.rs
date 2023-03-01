use std::collections::HashSet;

use actix::prelude::*;

#[derive(Message, Eq, Hash, PartialEq, Clone, Debug)]
#[rtype(result = "()")]
pub struct ChatID(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Notification(pub String);

impl From<Notification> for ChatID {
    fn from(notification: Notification) -> Self {
        ChatID(notification.0)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddSub {
    pub chat_id: ChatID,
    pub addr: Recipient<ChatID>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub chat_ids: HashSet<ChatID>,
    pub addr: Recipient<ChatID>,
}
