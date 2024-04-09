use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde::{
    ser::{Serialize, SerializeStruct, Serializer},
    Deserialize,
};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Deserialize, Debug)]
pub struct Todo {
    pub id: i64,
    pub note: String,
}

// https://docs.rs/serde/latest/serde/trait.Serialize.html
// https://serde.rs/impl-serialize.html
// Implement Serializer for Todo
// Due to JavaScript Maximum Safe Integer Limitation : 2^53 - 1 (9007199254740990)
// Passing integer greater than 9007199254740990 will result to JavaScript rounding the value
// to the nearest representable integer which is not accurate.
impl Serialize for Todo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut todo_map = serializer.serialize_struct("Todo", 2)?;
        todo_map.serialize_field("id", &self.id.to_string())?;
        todo_map.serialize_field("note", &self.note)?;
        todo_map.end()
    }
}

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

pub async fn bulk_retrieve(
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
    Json(data): Json<Todo>,
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
