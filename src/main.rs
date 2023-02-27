use actix_web::{middleware::Logger, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting WebSocket server at ws://0.0.0.0:8080");

    HttpServer::new(move || App::new().wrap(Logger::default()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
