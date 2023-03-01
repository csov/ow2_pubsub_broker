mod broker;
mod message;
mod session;
use std::{collections::HashSet, time::Instant};

use actix::*;
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

async fn broker_connection(
    req: HttpRequest,
    stream: web::Payload,
    broker: web::Data<Addr<broker::Broker>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            chat_ids: HashSet::new(),
            hb: Instant::now(),
            broker_addr: broker.get_ref().clone(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let broker = broker::Broker::new().await.start();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting WebSocket server at ws://0.0.0.0:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(broker.clone()))
            .route("/broker/ws", web::get().to(broker_connection))
            .wrap(Logger::default())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
