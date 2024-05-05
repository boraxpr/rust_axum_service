mod handlers;

use axum::{
    body::Body,
    extract::Request,
    http::{
        header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION},
        HeaderValue, Method, Response, StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use dotenv::dotenv;
use governor::middleware::StateInformationMiddleware;
use handlers::{create_todo, get_all_todo, get_todo_by_id};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tower_governor::{governor::GovernorConfig, key_extractor::PeerIpKeyExtractor, GovernorLayer};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultOnFailure, DefaultOnRequest, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};
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
    // let governor_config = Box::new(
    //     GovernorConfigBuilder::default()
    //         .per_second(2)
    //         .burst_size(5)
    //         .finish()
    //         .unwrap(),
    // );
    // Example of using preset : secure
    // It's needed to specify key extractor and middleware
    // However, the default middlewares are not available in the same crate (tower_governor) but it's must be imported from governor::middleware
    // https://docs.rs/governor/0.6.3/governor/middleware/
    let governor_config = Arc::new(GovernorConfig::<
        PeerIpKeyExtractor,
        StateInformationMiddleware,
    >::secure());
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

    // TODO: Pass DAO to all handler manually or deal with shared state
    let router = Router::new()
        .route("/todos", get(get_all_todo).post(create_todo))
        .route("/todos/:id", get(get_todo_by_id))
        .layer(GovernorLayer {
            config: governor_config,
        })
        .layer(
            // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
            // Tracing layer is used for tracing the flow of the request
            // However, Body can't be logged because the body is an async stream, in order to log it, the whole thing need to be buffered
            // Which does not work for infinite incoming stream
            // https://github.com/hyperium/tonic/discussions/1133
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<Body>| {
                    tracing::debug_span!("http-request", status_code = tracing::field::Empty,)
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    |response: &Response<Body>, _latency: Duration, span: &Span| {
                        span.record("status_code", &tracing::field::display(response.status()));
                        tracing::debug!("response generated")
                    },
                )
                .on_failure(
                    DefaultOnFailure::new()
                        .level(Level::ERROR)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
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

    let test_router = Router::new().route("/test", get(test_handler));

    let app: Router<()> = Router::new().merge(router).merge(test_router);
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
        // into_make_service_with_connect_info is required for tower_governor/governor PeerIpKeyExtractor; in order to ratelimit by IP address, it must be used to parse the peer ip address for tower to find.
        // READ https://github.com/benwis/tower-governor; Common pitfalls
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn test_handler() -> Result<impl IntoResponse, impl IntoResponse> {
    Ok::<(axum::http::StatusCode, &str), ()>((StatusCode::OK, "OK"))
}
