#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use actix::prelude::*;

#[derive(Message, Eq, Hash, PartialEq, Clone, Debug)]
#[rtype(result = "()")]
pub struct ChatID(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Trigger;

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

pub struct Broker {
    sessions: HashMap<ChatID, HashSet<Recipient<ChatID>>>,
    test_counter: u8,
}

impl Broker {
    pub fn new() -> Self {
        Broker { sessions: HashMap::new(), test_counter: 0 }
    }

    fn send_id(&mut self) {
        self.test_counter += 1;
        if self.test_counter == 10 {
            self.test_counter = 0
        }
        let chat_id = ChatID((self.test_counter % 10).to_string());
        if let Some(recipients) = self.sessions.get(&chat_id) {
            for r in recipients.iter() {
                r.do_send(chat_id.clone());
            }
        }
    }
}

impl Actor for Broker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = ctx.address();
        // TODO: replace this with logic that connects to GCP PubSub
        actix::spawn(async move {
            loop {
                actix::clock::sleep(Duration::from_secs(10)).await;
                addr.do_send(Trigger {});
            }
        });
    }
}

impl Handler<Trigger> for Broker {
    type Result = ();

    fn handle(&mut self, _: Trigger, _ctx: &mut Self::Context) -> Self::Result {
        self.send_id();
    }
}

impl Handler<AddSub> for Broker {
    type Result = ();

    fn handle(&mut self, msg: AddSub, _: &mut Self::Context) -> Self::Result {
        self.sessions
            .entry(msg.chat_id)
            .and_modify(|recipients| {
                recipients.insert(msg.addr.clone());
            })
            .or_insert(vec![msg.addr].into_iter().collect());
    }
}

impl Handler<Disconnect> for Broker {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        for chat_id in msg.chat_ids {
            self.sessions.entry(chat_id).and_modify(|recipients| {
                recipients.remove(&msg.addr);
            });
        }
    }
}
