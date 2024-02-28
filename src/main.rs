use axum::http::header::{ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN, AUTHORIZATION};
use axum::http::{HeaderValue, Method};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, PgPool};
use tower_http::cors::CorsLayer;

async fn retrieve(
    Path(id): Path<i64>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("SELECT * FROM TODO WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
    {
        Ok(todo) => Ok((StatusCode::OK, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn bulk_retreive(State(pool): State<PgPool>) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("SELECT * FROM TODO")
        .fetch_all(&pool)
        .await
    {
        Ok(todos) => Ok((StatusCode::OK, Json(todos))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

async fn add(
    State(pool): State<PgPool>,
    Json(data): Json<TodoNew>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("INSERT INTO TODO (note) VALUES ($1) RETURNING id, note")
        .bind(&data.note)
        .fetch_one(&pool)
        .await
    {
        Ok(todo) => Ok((StatusCode::CREATED, Json(todo))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

// #[derive(Clone)]
// struct MyState {
//     pool: PgPool,
// }

#[tokio::main]
async fn main() {
    // sqlx::migrate!("./migrations")
    //     .run(&pool)
    //     .await
    //     .map_err(CustomError::new)?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect("postgresql://root:6501@host.docker.internal:5432/root")
        .await
        .unwrap();

    let router = Router::new()
        .route("/create_todo", post(add))
        .route("/todos/:id", get(retrieve))
        .route("/todos", get(bulk_retreive))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST]) // Specify the allowed HTTP methods
                .allow_headers([AUTHORIZATION, ACCEPT, ACCESS_CONTROL_ALLOW_ORIGIN]), // Specify the allowed request headers
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7070").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[derive(Deserialize)]
struct TodoNew {
    pub note: String,
}

#[derive(Serialize, FromRow)]
struct Todo {
    pub id: i32,
    pub note: String,
}
