#![allow(dead_code)]
use std::{
    collections::{HashMap, HashSet},
    env::var,
    error, process,
};

use actix::prelude::*;
use futures_util::StreamExt;
use google_cloud_default::WithAuthExt;
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    subscription::SubscriptionConfig,
};

use crate::message::{AddSub, ChatID, Disconnect, Notification};

lazy_static::lazy_static! {
    static ref GOOGLE_APPLICATION_CREDENTIALS: String = var("GOOGLE_APPLICATION_CREDENTIALS").expect("GOOGLE_APPLICATION_CREDENTIALS must be set");
    static ref TOPIC: String = var("OW2_TOPIC").expect("OW2_TOPIC must be set");
    // Subscription name must start with a character
    static ref SUBSCRIPTION: String = format!("a{}", uuid::Uuid::new_v4());

}

pub struct Broker {
    sessions: HashMap<ChatID, HashSet<Recipient<ChatID>>>,
}

impl Broker {
    pub async fn new() -> Self {
        Broker { sessions: HashMap::new() }
    }
}

impl Actor for Broker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = ctx.address();

        log::debug!("Starting GCP Subscriber");
        actix::spawn(async move {
            if let Err(e) = connect_to_gcp(addr).await {
                log::error!("Error connecting to GCP PUB/SUB: {:?}", e);
                process::exit(0x0100);
            }
        });
    }
}

async fn connect_to_gcp(addr: Addr<Broker>) -> Result<(), Box<dyn error::Error>> {
    let config = ClientConfig::default().with_auth().await?;
    let gcp_client = Client::new(config).await?;
    let subscription = gcp_client.subscription(&SUBSCRIPTION);
    if !subscription.exists(None, None).await? {
        let config = SubscriptionConfig::default();
        let topic = gcp_client.topic(&TOPIC);
        subscription.create(topic.fully_qualified_name(), config, None, None).await?;
    }
    let mut iter = subscription.subscribe(None).await?;
    log::debug!("Connected to GCP PUB/SUB");

    while let Some(received_message) = iter.next().await {
        received_message.ack().await?;
        let msg = received_message.message;
        log::debug!("Received message: {:?}", String::from_utf8(msg.data.clone()));
        match String::from_utf8(msg.data) {
            Ok(chat_id) => {
                addr.do_send(Notification(chat_id));
            }
            Err(e) => {
                log::error!("Error parsing message: {:?}", e);
                return Err(Box::new(e));
            }
        }
    }
    Ok(())
}

impl Handler<Notification> for Broker {
    type Result = ();

    fn handle(&mut self, msg: Notification, _ctx: &mut Self::Context) -> Self::Result {
        let chat_id: ChatID = msg.into();
        let Some(recipients) = self.sessions.get(&chat_id) else { return };
        for r in recipients.iter() {
            r.do_send(chat_id.clone());
        }
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
