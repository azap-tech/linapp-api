use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::{middleware, App, HttpServer};
use deadpool_postgres::Config;
use dotenv::dotenv;
use listenfd::ListenFd;
use std::env;
use tokio_postgres::NoTls;

mod doctor;
mod error;
mod location;
mod login;
mod sync_service;
mod tickets;
mod users_notification;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut listenfd = ListenFd::from_env();
    dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let cfg = Config::from_env("PG").expect("Deadpool_postgres config invalid");
    let db_pool = cfg.create_pool(NoTls).expect("Can not connect to database");

    let store_broadcaster = sync_service::Broadcaster::create();

    let mut server = HttpServer::new(move || {
        let session_secret = env::var("SESSION_SECRET").expect("env var SESSION_SECRET undefined");
        let session_key = session_secret.as_bytes();
        assert!(session_key.len() >= 32);
        App::new()
            .data(db_pool.clone())
            .app_data(store_broadcaster.clone())
            .wrap(Cors::new().supports_credentials().finish())
            .wrap(
                CookieSession::signed(session_key) // <- create cookie based session middleware
                    .secure(false),
            )
            // enable logger always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            .configure(tickets::config)
            .configure(login::config)
            .configure(doctor::config)
            .configure(location::config)
            .configure(sync_service::config)
            .configure(users_notification::config)
    });

    // allow auto reload
    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind(format!("0.0.0.0:{}", port))?
    };

    server.run().await
}
