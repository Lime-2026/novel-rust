use axum::extract::{OriginalUri, State};
use axum::Form;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;
use cookie::Cookie;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use crate::{routes, services};
use crate::services::json::ApiResponse;
use crate::utils::conf::CONFIG;
use crate::utils::cookie::set_cookie_value;
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

pub(crate) static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9]{6,32}$").unwrap());
pub(crate) static PASSWORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^.{6,32}$").unwrap());
static EMAIL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(
r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
).unwrap());

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterForm {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) email: String,
}

pub(crate) async fn get_register(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/user/register.html", CONFIG.theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let template_path = format!("{}/user/register.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}

pub(crate) async fn post_register(
    mut jar: CookieJar,
    Form(params) : Form<RegisterForm>
) -> impl IntoResponse {
    if !file_exists(format!("templates/{}/user/register.html", CONFIG.theme_dir)) {
        return Err(TeraRenderError::InvalidId);
    }
    let mut error_msg = Vec::new();
    // 校验用户名
    if params.username.is_empty() {
        error_msg.push("用户名不能为空".to_string());
    } else if !USERNAME_RE.is_match(&params.username) {
        error_msg.push("用户名必须是6-32位，仅包含英文和数字".to_string());
    }
    // 校验密码
    if params.password.is_empty() {
        error_msg.push("密码不能为空".to_string());
    } else if !PASSWORD_RE.is_match(&params.password) {
        error_msg.push("密码必须是6-32位".to_string());
    }
    // 校验邮箱
    if params.email.is_empty() {
        error_msg.push("邮箱不能为空".to_string());
    } else if !EMAIL_RE.is_match(&params.email) {
        error_msg.push("邮箱格式错误".to_string());
    }
    if !error_msg.is_empty() {
        return Ok((jar,ApiResponse::fail("参数校验失败", Some(error_msg))));
    }
    match services::user::create_user(&params.username, &params.password, &params.email).await {
        Ok(user) => {
            jar = set_cookie_value(jar, "ss_userid", user.uid.to_string().as_str(), 365 * 86400,false,false);
            jar = set_cookie_value(jar, "ss_username", user.uname.to_owned().as_str(), 365 * 86400,false,false);
            jar = set_cookie_value(jar, "ss_password", user.pass.to_owned().as_str(), 365 * 86400,true,false);
            Ok((jar,ApiResponse::success("注册成功", Some(""))))
        },
        Err(e) => {
            Ok((jar,ApiResponse::fail("注册失败", Some(vec![e.to_string()]))))
        }
    }
}

pub(crate) async fn get_logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar
        .remove(Cookie::from("ss_userid"))
        .remove(Cookie::from("ss_username"))
        .remove(Cookie::from("ss_password"));
    (jar, Redirect::to("/login"))
}