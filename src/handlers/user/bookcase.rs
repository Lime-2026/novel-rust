use axum::body::Body;
use axum::extract::{OriginalUri, State, Form};
use axum::http::{HeaderMap, Request};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::{CookieJar };
use cookie::Cookie;
use sea_orm::Values;
use serde::Deserialize;
use crate::{routes, services};
use crate::services::json::ApiResponse;
use crate::services::user::get_bookcase_list;
use crate::utils::conf::CONFIG;
use crate::utils::db::db::{exec_sql, query_count};
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

pub(crate) async fn get_bookcase(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
    jar: CookieJar
) -> Result<Response, TeraRenderError> {
    // 有中间件 不存在文件自然重定向
    let user_id = jar.get("ss_userid").map(|c| c.value().to_string());
    let Some(user_id) = user_id else {    // 通常不会出问题,中间件校验过了 出问题直接重定向吧
        return Ok(Redirect::to("/login").into_response());
    };
    if user_id.is_empty() {
        return Ok(Redirect::to("/login").into_response());
    }
    let bs = get_bookcase_list(user_id.as_str()).await.unwrap_or_else(|e| {
        eprintln!("get_bookcase_list error: {e:?}");
        vec![]
    });
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("bookcase_list", &bs);
    let template_path = format!("{}/user/bookcase.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ).into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct DelBookcaseReq {
    caseid: u64
}

#[derive(Debug, Deserialize)]
pub(crate) struct AddBookcaseReq {
    articleid: u64,
    articlename: String,
    chapterid: Option<u64>,
    chaptername: Option<String>,
}

pub(crate) async fn add_bookcase(
    jar: CookieJar,
    Form(params): Form<AddBookcaseReq>,
) -> impl IntoResponse {
    if !file_exists(format!("templates/{}/user/bookcase.html", CONFIG.theme_dir)) {
        return Err(TeraRenderError::InvalidId);
    }
    let user_id = jar.get("ss_userid").map(|c| c.value().to_string());
    if params.articleid == 0 || params.articlename.is_empty() || user_id.is_none(){
        return Ok(ApiResponse::fail("添加失败", Some(vec!["传参错误".to_string()])))
    }
    let source_id = CONFIG.source_id(params.articleid);
    let source_cid = params.chapterid.unwrap_or(0);
    let count_sql = format!("SELECT COUNT(*) AS cnt FROM {}article_bookcase WHERE articleid = ? and userid = ?", CONFIG.prefix);
    let num = query_count(
        count_sql.as_str(),
        Some(Values(vec![source_id.into(), user_id.clone().into()])),
    ).await.unwrap_or(0);
    let num_2 = if num > 0 {
        let s = format!("UPDATE {}article_bookcase SET chapterid = ?, chaptername = ? WHERE articleid = ? and userid = ?", CONFIG.prefix).to_owned();
        exec_sql(
            s.as_str(),
            Some(Values(vec![
                source_cid.into(),
                params.chaptername.clone().unwrap_or_default().into(),
                source_id.into(),
                user_id.clone().into(),
            ])),
        ).await.unwrap_or(0)
    } else {
        let ss = format!("UPDATE {}article_article SET goodnum = goodnum + 1 WHERE articleid = ?", CONFIG.prefix).to_owned();
        _ = exec_sql(
            ss.as_str(),
            Some(Values(vec![
                source_id.into(),
            ])),
        ).await.unwrap_or(0);
        let s = format!("INSERT INTO {}article_bookcase (articleid, articlename, chapterid, chaptername, userid,username) VALUES (?, ?, ?, ?, ?, ?)", CONFIG.prefix).to_owned();
        exec_sql(
            s.as_str(),
            Some(Values(vec![
                source_id.into(),
                params.articlename.clone().into(),
                source_cid.into(),
                params.chaptername.clone().unwrap_or_default().into(),
                user_id.clone().into(),
                "".into()
            ])),
        ).await.unwrap_or(0)
    };
    if num_2 == 0 {
        return Ok(ApiResponse::fail("添加失败", Some(vec!["数据库操作失败".to_string()])))
    }
    Ok(ApiResponse::success("添加成功", Some("")))
}

pub(crate) async fn del_bookcase(
    jar: CookieJar,
    Form(params): Form<DelBookcaseReq>,
) -> impl IntoResponse {
    if !file_exists(format!("templates/{}/user/bookcase.html", CONFIG.theme_dir)) {
        return Err(TeraRenderError::InvalidId);
    }
    let user_id = jar.get("ss_userid").map(|c| c.value().to_string());
    if params.caseid == 0 || user_id.is_none() {
        return Ok(ApiResponse::fail("删除失败", Some(vec!["传参错误".to_string()])))
    }
    let sql = format!("DELETE FROM {}article_bookcase WHERE caseid = ? and userid = ?", CONFIG.prefix);
    match exec_sql(
        sql.as_str(),
        Some(Values(vec![params.caseid.into(),user_id.into()])),
    ).await {
        Ok(_num) => {
            Ok(ApiResponse::success("删除成功", Some("")))
        },
        Err(e) => {
            eprintln!("sql error: {e:?}");
            Ok(ApiResponse::fail("删除失败", Some(vec!["数据库操作失败".to_string()])))
        }
    }
}

pub(crate) async fn login_auth(
    jar: CookieJar,
    req: Request<Body>,
    next: Next,
) -> Result<Response, TeraRenderError> {
    if !file_exists(format!("templates/{}/user/bookcase.html", CONFIG.theme_dir)) {
        return Err(TeraRenderError::InvalidId);
    }
    let user_id = jar.get("ss_userid").map(|c| c.value().to_string());
    let pass = jar.get("ss_password").map(|c| c.value().to_string());
    let (Some(user_id), Some(pass)) = (user_id, pass) else {
        return Ok(Redirect::to("/login").into_response());
    };
    match services::user::is_user_login(&user_id, &pass).await {
        Ok(_user) => {
            Ok(next.run(req).await)
        }
        Err(_e) => {
            let jar = jar
                .remove(Cookie::build("ss_userid").path("/").build())
                .remove(Cookie::build("ss_username").path("/").build())
                .remove(Cookie::build("ss_password").path("/").build());
            Ok((jar, Redirect::to("/login")).into_response())
        }
    }
}
