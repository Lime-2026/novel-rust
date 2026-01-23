use axum::http::StatusCode;
use sea_orm::DbErr;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

// 自定义错误类型
#[derive(Debug)]
pub enum AppError {
    DbError(DbErr),
    NotFound(String),
    ValidationError(String),
    AuthError(String),
}

// 实现IntoResponse，让Axum自动转换为HTTP响应
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            AppError::DbError(e) => (StatusCode::INTERNAL_SERVER_ERROR, 50000, e.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, 40400, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, 40000, msg),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, 40100, msg),
        };

        let body = axum::Json(ApiError { code, message });
        (status, body).into_response()
    }
}

// 实现From转换，简化错误处理
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::DbError(err)
    }
}