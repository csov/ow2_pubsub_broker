use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::pubsub::{AddSub, Broker, ChatID, Disconnect};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(25);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    pub chat_ids: HashSet<ChatID>,
    pub hb: Instant,
    pub broker_addr: Addr<Broker>,
}

impl WsChatSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                act.broker_addr.do_send(Disconnect {
                    chat_ids: act.chat_ids.clone(),
                    addr: ctx.address().recipient(),
                });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.broker_addr.do_send(Disconnect {
            chat_ids: self.chat_ids.clone(),
            addr: ctx.address().recipient(),
        });
        Running::Stop
    }
}

/// Handle messages from the broker, we simply send it to peer websocket
impl Handler<ChatID> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: ChatID, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                if let Ok(chat_id) = Uuid::parse_str(&text) {
                    self.broker_addr.do_send(AddSub {
                        chat_id: ChatID(chat_id.to_string()),
                        addr: ctx.address().recipient(),
                    });
                }
            }
            ws::Message::Binary(_) => {},
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
