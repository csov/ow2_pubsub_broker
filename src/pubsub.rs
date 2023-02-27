#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use actix::prelude::*;
use rand::Rng;

#[derive(Message, Eq, Hash, PartialEq, Clone)]
#[rtype(result = "()")]
pub struct ChatID(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Trigger;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddRecipient {
    chat_id: ChatID,
    pub addr: Recipient<ChatID>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct DelRecipient {
    chat_ids: Vec<ChatID>,
    pub addr: Recipient<ChatID>,
}

pub struct Broker {
    // TODO: research why examples use HashMap<usize, Recipient<Message>>
    // The reason might be that a recipient does not produce a unique hash.
    // If this is the case, a unique ID must be passed back to the session actor.
    sessions: HashMap<ChatID, HashSet<Recipient<ChatID>>>,
}

impl Broker {
    pub fn new() -> Self {
        Broker { sessions: HashMap::new() }
    }

    fn send_id(&mut self) {
        let mut rng = rand::thread_rng();
        let chat_id = ChatID((rng.gen::<u8>() % 10).to_string());
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

impl Handler<AddRecipient> for Broker {
    type Result = ();

    fn handle(
        &mut self,
        msg: AddRecipient,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.sessions
            .entry(msg.chat_id)
            .and_modify(|recipients| {
                recipients.insert(msg.addr.clone());
            })
            .or_insert(vec![msg.addr].into_iter().collect());
    }
}

impl Handler<DelRecipient> for Broker {
    type Result = ();

    fn handle(
        &mut self,
        msg: DelRecipient,
        _: &mut Self::Context,
    ) -> Self::Result {
        for chat_id in msg.chat_ids {
            self.sessions.entry(chat_id).and_modify(|recipients| {
                recipients.remove(&msg.addr);
            });
        }
    }
}
