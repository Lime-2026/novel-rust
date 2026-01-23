use serde::Serialize;
use axum::Json;
use axum::http::StatusCode;

// 通用JSON响应结构体（泛型，支持不同数据类型）
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 接口是否成功
    pub success: bool,
    /// 提示信息（成功/失败描述）
    pub msg: String,
    /// 附加数据（可选，比如成功后返回用户信息）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 错误详情（可选，比如多个校验错误）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

// 快捷构造函数：成功响应
impl<T> ApiResponse<T> {
    pub fn success(msg: &str, data: Option<T>) -> Self {
        Self {
            success: true,
            msg: msg.to_string(),
            data,
            errors: None,
        }
    }

    // 失败响应
    pub fn fail(msg: &str, errors: Option<Vec<String>>) -> Self {
        Self {
            success: false,
            msg: msg.to_string(),
            data: None,
            errors,
        }
    }
}

// 实现IntoResponse，让ApiResponse可以直接作为axum的响应返回
impl<T: Serialize> axum::response::IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json; charset=utf-8")],
            Json(self),
        ).into_response()
    }
}