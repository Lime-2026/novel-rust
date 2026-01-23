use crate::utils::conf::CONFIG;
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;
use crate::{routes, services};
use axum::extract::{OriginalUri, State, Form};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use crate::services::json::ApiResponse;
use crate::utils::cookie::set_cookie_value;

#[derive(Debug, Deserialize)]
pub(crate) struct LoginForm {
    pub(crate) username: String,
    pub(crate) password: String,
}

pub(crate) async fn get_login(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/user/login.html", CONFIG.theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let template_path = format!("{}/user/login.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}

pub(crate) async fn post_login(
    mut jar: CookieJar,
    Form(params) : Form<LoginForm>
) -> impl IntoResponse {
    // 不存在这个模板 你请求你妈呢
    if !file_exists(format!("templates/{}/user/login.html", CONFIG.theme_dir)) {
        return Err(TeraRenderError::InvalidId);
    }
    let mut error_msg = Vec::new();
    // 校验用户名
    if params.username.is_empty() {
        error_msg.push("用户名不能为空".to_string());
    } else if !crate::handlers::user::register::USERNAME_RE.is_match(&params.username) {
        error_msg.push("用户名必须是6-32位，仅包含英文和数字".to_string());
    }
    // 校验密码
    if params.password.is_empty() {
        error_msg.push("密码不能为空".to_string());
    } else if !crate::handlers::user::register::PASSWORD_RE.is_match(&params.password) {
        error_msg.push("密码必须是6-32位".to_string());
    }
    if !error_msg.is_empty() {
        return Ok((jar,ApiResponse::fail("参数校验失败", Some(error_msg))));
    }
    match services::user::get_user(&params.username, &params.password).await {
        Ok(user) => {
            jar = set_cookie_value(jar, "ss_userid", user.uid.to_string().as_str(), 365 * 86400,false,false);
            jar = set_cookie_value(jar, "ss_username", user.uname.to_owned().as_str(), 365 * 86400,false,false);
            jar = set_cookie_value(jar, "ss_password", user.pass.to_owned().as_str(), 365 * 86400,true,false);
            Ok((jar,ApiResponse::success("登录成功", Some(""))))
        },
        Err(e) => {
            eprintln!("登录失败: {:?}", e);
            Ok((jar,ApiResponse::fail("登录失败", Some(vec![e.to_string()]))))
        }
    }
}

