use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};

use super::{ApiError, jwt::Uid};
use crate::{api::counter::get_user_counter, db::CounterRecord};

#[derive(Deserialize, Debug)]
pub struct AddPayload {
    pub counter_id: i32,
    pub step: i32,
}

pub async fn add(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<AddPayload>,
) -> Result<Json<Value>, ApiError> {
    let counter = get_user_counter(payload.counter_id, uid, &pool).await?;
    let next_value = counter.value + payload.step;
    sqlx::query(
        r#"
    insert into counter_records (counter_id, step, begin, end) values (?, ?, ?, ?);
    update counters set value = ?, updated_at = CURRENT_TIMESTAMP where id = ?;
    "#,
    )
    .bind(payload.counter_id)
    .bind(payload.step)
    .bind(counter.value)
    .bind(next_value)
    .bind(next_value)
    .bind(payload.counter_id)
    .execute(&pool)
    .await?;
    Ok(Json(json!({})))
}

pub async fn list(
    Uid(uid): Uid,
    State(pool): State<Pool<Sqlite>>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<CounterRecord>>, ApiError> {
    get_user_counter(id, uid, &pool).await?;
    let records = sqlx::query_as::<_, CounterRecord>(
        "select * from counter_records where counter_id = ? order by id desc",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(records))
}
