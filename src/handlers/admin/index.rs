use std::env;
use axum::extract::{Query, State};
use axum::{Form, Json};
use axum::response::IntoResponse;
use serde::{Deserialize};
use crate::models::config::Config;
use crate::routes;
use crate::services::json::ApiResponse;
use crate::services::user::timestamp_10;
use crate::utils::conf::{get_config, set_config};
use crate::utils::file::get_folders;
use crate::utils::templates::render::{TeraRenderError, render_template};

#[derive(Debug, Deserialize)]
pub(crate) struct AdminFrom {
    pub(crate) token: String,
    pub(crate) time: Option<u64>
}

pub(crate) async fn admin_conf_edit(
    Query(params): Query<AdminFrom>,
    Json(body): Json<Config>
)-> impl IntoResponse {
    let token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::new());
    #[cfg(not(debug_assertions))]   // 非调试模式下
    {
        if token.is_empty() || token == "admin_token" {
            return ApiResponse::fail("token error", Some(vec!["默认密钥不可用".to_string()]));
        }
    }
    let time = params.time.unwrap_or_else(|| 0);
    let new_token = format!("{:x}",md5::compute(format!("{token}{time}",token=token,time=time)));
    if new_token != params.token || time + 300 < timestamp_10() as u64 {
        return ApiResponse::fail("token error", Some(vec!["密钥不正确或过期".to_string()]));
    }
    let conf = match serde_json::to_string_pretty(&body) {
        Ok(s) => s,
        Err(e) => {
            return  ApiResponse::fail("serialize error", Some(vec![e.to_string()]));
        }
    };
    if let Err(e) = tokio::fs::write("conf.json", conf).await {
        return ApiResponse::fail("save error", Some(vec![e.to_string()]));
    }
    set_config(body);
    ApiResponse::success("success", Some(String::new()))
}

pub(crate) async fn admin_conf_get(
    Form(params) : Form<AdminFrom>
) -> impl IntoResponse{
    let token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::new());
    #[cfg(not(debug_assertions))]   // 非调试模式下
    {
        if token.is_empty() || token == "admin_token" {
            return ApiResponse::fail("token error", Some(vec!["默认密钥不可用".to_string()]));
        }
    }
    let time = params.time.unwrap_or_else(|| 0);
    let new_token = format!("{:x}",md5::compute(format!("{token}{time}",token=token,time=time)));
    if new_token != params.token || time + 300 < timestamp_10() as u64 {
        return ApiResponse::fail("token error", Some(vec!["密钥不正确或过期".to_string()]));
    }
    // 密钥验证成功把CONFIG序列化后返回
    let conf = match serde_json::to_string_pretty(&*get_config()) {
        Ok(s) => s,
        Err(e) => {
            return  ApiResponse::fail("serialize error", Some(vec![e.to_string()]));
        }
    };
    ApiResponse::success("success",Some(conf))
}

pub(crate) async fn index(
    State(app_state): State<routes::app::AppState>,
    Form(params) : Form<AdminFrom>
) -> Result<impl IntoResponse, TeraRenderError> {
    let mut ctx = tera::Context::new();
    let token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::new());
    let time = timestamp_10();
    #[cfg(not(debug_assertions))]   // 非调试模式下
    {
        if token.is_empty() || token == "admin_token" || params.token != token{
            return Err(TeraRenderError::InvalidId);
        }
    }
    #[cfg(debug_assertions)]    // 调试模式下 无需验证密钥
    {
        if token.is_empty() || params.token != token {
            return Err(TeraRenderError::InvalidId);
        }
    }
    let new_token = format!("{:x}",md5::compute(format!("{token}{time}",token=token,time=time)));
    let folders = get_folders("templates");
    ctx.insert("themes", &folders);
    ctx.insert("token", &new_token);
    ctx.insert("time", &time);
    let html = render_template(app_state.tera.clone(), "admin.html", ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}
