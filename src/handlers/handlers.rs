use axum::{extract::Path, extract::State, http::StatusCode, response::IntoResponse, Json};

use sqlx::{FromRow, PgPool};

use crate::{Todo, TodoNew};

pub async fn retrieve(
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

pub async fn bulk_retreive(
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match sqlx::query_as::<_, Todo>("SELECT * FROM TODO")
        .fetch_all(&pool)
        .await
    {
        Ok(todos) => Ok((StatusCode::OK, Json(todos))),
        Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
    }
}

pub async fn add(
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
