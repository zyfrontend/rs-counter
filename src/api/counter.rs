use super::{ApiError, jwt::Uid};
use crate::db::Counter;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};

pub async fn get_user_counter(
    id: i32,
    user_id: i32,
    pool: &Pool<Sqlite>,
) -> Result<Counter, ApiError> {
    let counter =
        sqlx::query_as::<_, Counter>("select * from counters where id = ? and user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(counter)
}

pub async fn list(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<Vec<Counter>>, ApiError> {
    let counters = sqlx::query_as::<_, Counter>(
        "select * from counters where user_id = ? order by sequence desc",
    )
    .bind(uid)
    .fetch_all(&pool)
    .await?;
    Ok(Json(counters))
}

#[derive(Deserialize, Debug)]
pub struct AddPayload {
    pub name: String,
    pub value: i32,
    pub step: i32,
    pub input_step: bool,
}

pub async fn add(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<AddPayload>,
) -> Result<Json<Counter>, ApiError> {
    let sequence = sqlx::query_as::<_, (i32,)>(
        "select sequence from counters where user_id = ? order by sequence desc limit 1",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await;
    let sequence = match sequence {
        Ok((sequence,)) => sequence + 1,
        Err(sqlx::Error::RowNotFound) => 1,
        Err(e) => return Err(ApiError::Internal(e.into())),
    };
    let counter = sqlx::query_as::<_, Counter>(
        "insert into counters (user_id, name, value, step, input_step, sequence) values (?, ?, ?, ?, ?, ?) returning *",
    )
    .bind(uid)
    .bind(payload.name)
    .bind(payload.value)
    .bind(payload.step)
    .bind(payload.input_step)
    .bind(sequence)
    .fetch_one(&pool)
    .await?;
    Ok(Json(counter))
}

pub async fn show(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<i32>,
) -> Result<Json<Counter>, ApiError> {
    let counter = get_user_counter(id, uid, &pool).await?;
    Ok(Json(counter))
}

#[derive(Deserialize, Debug)]
pub struct UpdatePayload {
    pub name: String,
    pub step: i32,
    pub input_step: bool,
}

pub async fn update(
    Path(id): Path<i32>,
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdatePayload>,
) -> Result<Json<Value>, ApiError> {
    get_user_counter(id, uid, &pool).await?;
    sqlx::query(
        "update counters set name = ?, step = ?, input_step = ?, updated_at = CURRENT_TIMESTAMP where id = ?",
    )
    .bind(payload.name)
    .bind(payload.step)
    .bind(payload.input_step)
    .bind(id)
    .execute(&pool)
    .await?;
    Ok(Json(json!({})))
}

pub async fn destroy(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, ApiError> {
    get_user_counter(id, uid, &pool).await?;
    sqlx::query(
        r#"delete from counters where id = ?;delete from counter_records where counter_id = ?"#,
    )
    .bind(id)
    .bind(id)
    .execute(&pool)
    .await?;
    Ok(Json(json!({})))
}

pub async fn top(
    Path(id): Path<i32>,
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
) -> Result<Json<Value>, ApiError> {
    get_user_counter(id, uid, &pool).await?;
    let sequence = sqlx::query_as::<_, (i32,)>(
        "select sequence from counters where user_id = ? order by sequence desc limit 1",
    )
    .bind(uid)
    .fetch_one(&pool)
    .await;
    let sequence = match sequence {
        Ok((sequence,)) => sequence + 1,
        Err(e) => return Err(ApiError::from(e)),
    };
    sqlx::query("update counters set sequence = ?, updated_at = CURRENT_TIMESTAMP where id = ?")
        .bind(sequence)
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(Json(json!({})))
}
