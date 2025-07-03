use std::env;

use crate::{
    api::{
        ApiError,
        jwt::{AuthError, Claims, KEYS},
    },
    db::User,
};
use axum::{Json, extract::State};
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use tracing::info;

#[derive(Deserialize)]
pub struct LoginPayload {
    code: String,
}
#[derive(Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}
impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}
pub async fn login(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthBody>, ApiError> {
    let wx_user = wx_login(payload.code).await?;
    let user = sqlx::query_as::<_, User>("select * from users where openid = ?")
        .bind(&wx_user.openid)
        .fetch_one(&pool)
        .await;
    let user = match user {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            sqlx::query("insert into users (openid, session_key) values (?, ?)")
                .bind(&wx_user.openid)
                .bind(&wx_user.session_key)
                .execute(&pool)
                .await?;
            sqlx::query_as::<_, User>("select * from users where openid = ?")
                .bind(wx_user.openid)
                .fetch_one(&pool)
                .await?
        }
        Err(e) => return Err(ApiError::from(e)),
    };
    let claims = Claims::new(user.id.to_string());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(Json(AuthBody::new(token)))
}

#[derive(Deserialize, Default)]
struct WxUser {
    pub openid: String,
    pub session_key: String,
}
async fn wx_login(code: String) -> Result<WxUser, AuthError> {
    println!("login code: {}", code);
    if code.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    let app_id = env::var("APP_ID").unwrap();
    let app_secret = env::var("APP_SECRET").unwrap();
    let endpoint = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        app_id, app_secret, code
    );
    let resp = reqwest::get(endpoint)
        .await
        .map_err(|_| AuthError::WrongCredentials)?
        .json::<Value>()
        .await
        .map_err(|_| AuthError::WrongCredentials)?;
    info!("login code: {},resp: {:?}", code, resp);

    let wx_user =
        serde_json::from_value::<WxUser>(resp).map_err(|_| AuthError::WrongCredentials)?;
    if wx_user.openid.is_empty() {
        return Err(AuthError::WrongCredentials);
    } else {
        Ok(wx_user)
    }
}
