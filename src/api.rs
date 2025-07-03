use axum::{Json, http::StatusCode, response::IntoResponse};
use jwt::AuthError;
use serde_json::json;

pub mod counter;
pub mod counter_record;
mod jwt;
pub mod user;

pub enum ApiError {
    Internal(anyhow::Error),
    Auth(AuthError),
    NotFound,
}
impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}
impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        Self::Auth(err)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::NotFound => {
                let body = Json(json!({
                    "error": "Not Found",
                }));
                (StatusCode::NOT_FOUND, body).into_response()
            }
            Self::Internal(err) => {
                let body = Json(json!({
                    "error": err.to_string(),
                }));
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            }
            Self::Auth(err) => err.into_response(),
        }
    }
}
