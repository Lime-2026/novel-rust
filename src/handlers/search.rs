use crate::models::novel::Novel;
use crate::utils::conf::CONFIG;
use crate::utils::cookie::is_cookie_exist;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;
use crate::{routes, services, utils};
use axum::extract::{Form, OriginalUri, Query, State};
use axum::http::header::{CONTENT_TYPE, HOST};
use axum::http::{HeaderMap, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::CookieJar;
use sea_orm::{Value, Values};
use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    pub keyword: String,
    pub page: Option<u64>, // 可以不存在 预留吧
}

pub(crate) async fn get_search(
    Query(params): Query<SearchQuery>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
    jar: CookieJar,
) -> Result<impl IntoResponse, TeraRenderError> {
    search(params.keyword, params.page, app_state, headers, uri, jar).await
}

#[allow(dead_code)]
pub(crate) async fn post_search(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
    jar: CookieJar,
    Form(params): Form<SearchQuery>,
) -> Result<impl IntoResponse, TeraRenderError> {
    search(params.keyword, params.page, app_state, headers, uri, jar).await
}

pub(crate) fn html_resp(s: String) -> Response {
    ([(CONTENT_TYPE, "text/html; charset=utf-8")], Html(s)).into_response()
}

/// 通用处理
async fn search(
    keyword: String,
    _page: Option<u64>,
    app_state: routes::app::AppState,
    headers: HeaderMap,
    uri: Uri,
    mut jar: CookieJar,
) -> Result<Response, TeraRenderError> {
    if CONFIG.search.delay == -1 {
        return Ok(html_resp(r#"<script>alert("对不起,管理员已关闭此功能.");window.history.go(-1);</script>"#.to_owned()));
    }
    if is_cookie_exist(&jar, "search_last_time") {
        let r = format!(r#"<script>alert("搜索间隔: {} 秒");window.history.go(-1);</script>"#, CONFIG.search.delay);
        return Ok(html_resp(r));
    }
    let limit = CONFIG.search.limit.min(100);   // 搜索结果最多显示100条
    let mut search_rows: Vec<Novel> = Vec::new();
    let mut search_no_rows : Vec<Novel> = Vec::new();
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    if !keyword.is_empty() {
        if keyword.graphemes(true).count() < CONFIG.search.min as usize {
            let r = format!(r#"<script>alert("关键字最少 {} 个字符");window.history.go(-1);</script>"#, CONFIG.search.min);
            return Ok(html_resp(r));
        }
        let search_key = Value::String(Some(keyword.clone()));
        search_rows = if CONFIG.sys_ver > 6.0f32 {    // 多选搜索兼容
            utils::redis::conn::get_cache_rows(
                format!("SELECT {filed} FROM {table}article_article WHERE {where} AND MATCH(articlename, author) AGAINST(CONCAT('+',?) IN BOOLEAN MODE) ORDER BY lastupdate DESC LIMIT {limit};", filed = CONFIG.get_field(), table = CONFIG.prefix, where = CONFIG.get_where(), limit = limit),
                url,
                CONFIG.search.time as u64,
                Some(Values(vec![search_key.clone()])),
            ).await
        } else {
             utils::redis::conn::get_cache_rows(
                format!("SELECT {filed} FROM {table}article_article WHERE {where} AND (articlename LIKE CONCAT('%',?, '%') OR author LIKE CONCAT('%',?, '%')) ORDER BY lastupdate DESC LIMIT {limit};", filed = CONFIG.get_field(), table = CONFIG.prefix, where = CONFIG.get_where(), limit = limit),
                url,
                CONFIG.search.time as u64,
                Some(Values(vec![search_key.clone(),search_key.clone()])),
            ).await
        };
        if CONFIG.search.delay > 0 {
            jar = utils::cookie::set_cookie_value(jar, "search_last_time", "1", CONFIG.search.delay as usize, true, true);
        }
    }
    if search_rows.is_empty() {
        search_no_rows = services::novel::common_novel_random(url, limit, CONFIG.search.time as u64).await;
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("keyword", &keyword);
    ctx.insert("rows", &search_rows);
    ctx.insert("search_no_rows", &search_no_rows);
    let template_path = format!("{}/search.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    let resp = ([(CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response();
    Ok((jar, resp).into_response())
}
