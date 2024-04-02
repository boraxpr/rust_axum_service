mod handlers;
use axum::http::header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION};
use axum::http::{HeaderValue, Method};
use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use handlers::{add, bulk_retreive, retrieve};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::{env, time::Duration};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            // Try grabbing RUST_LOG from environment variable
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax
                // target_module::module=level
                "boraxpr=trace,tower_http=trace,axum::rejection=trace,sqlx::query=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // governor default uses PeerIPKeyExtractor - behind reverse proxy, it will take all requests as if it's from the same IP
    // Manually config READ https://github.com/benwis/tower-governor/
    let governor_config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );
    // TODO: on a nice day, try secure preset
    // let governor_config = Box::new(GovernorConfig::secure());
    let governor_limiter = governor_config.limiter().clone();
    let interval = Duration::from_secs(60);
    // a separate background task to clean up
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        tracing::info!("rate limiting storage size: {}", governor_limiter.len());
        governor_limiter.retain_recent();
    });

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    if let Ok(environment) = env::var("ENV") {
        if environment == "DEV" {
            sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        }
    }

    let router = Router::new()
        .route("/create_todo", post(add))
        .route("/todos/:id", get(retrieve))
        .route("/todos", get(bulk_retreive))
        .layer(GovernorLayer {
            config: Box::leak(governor_config),
        })
        .layer(
            CorsLayer::new()
                .allow_origin(
                    env::var("ALLOW_ORIGIN_URL")
                        .expect("ALLOW_ORIGIN_URL must be set")
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .allow_methods([Method::GET, Method::POST]) // Specify the allowed HTTP methods
                .allow_headers([AUTHORIZATION, ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN]), // Specify the allowed request headers
        )
        .with_state(pool);

    // Port management
    let mut port: u16 = 8080;
    match env::var("PORT") {
        Ok(p) => {
            match p.parse::<u16>() {
                Ok(n) => {
                    port = n;
                }
                Err(_e) => {}
            };
        }
        Err(_e) => {}
    };

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
