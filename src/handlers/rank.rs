use std::collections::HashMap;
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap};
use axum::http::header::HOST;
use axum::response::IntoResponse;
use sea_orm::{Value, Values};
use serde::{Deserialize, Serialize};
use crate::{routes, services, utils};
use crate::models::novel::Novel;
use crate::services::novel::extract_str;
use crate::utils::conf::get_config;
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

const RANK_NAV_ITEMS: &[(&str, &str)] = &[
    ("allvisit", "总排行榜"),
    ("monthvisit", "月排行榜"),
    ("weekvisit", "周排行榜"),
    ("dayvisit", "日排行榜"),
    ("allvote", "总推荐榜"),
    ("monthvote", "月推荐榜"),
    ("weekvote", "周推荐榜"),
    ("dayvote", "日推荐榜"),
    ("goodnum", "收藏榜"),
];

fn get_rank_nav(key: &str) -> Option<&str> {
    RANK_NAV_ITEMS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

#[derive(Deserialize,Serialize)]
struct RankUrls {
    title: String,
    url: String,
    select: bool,
}

pub(crate) async fn get_rank(
    Path(code): Path<String>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/rank.html", get_config().theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let key = extract_str(&code).unwrap_or("");
    if key.is_empty() {
        return Err(TeraRenderError::InvalidId);
    }
    let title =  get_rank_nav(key).unwrap_or("");
    if title.is_empty() {
        return Err(TeraRenderError::InvalidId);
    }
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let rows = utils::redis::conn::get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} ORDER BY ? DESC LIMIT 100;", filed = get_config().get_field(), table = get_config().prefix, where = get_config().get_where()),
        url,
        get_config().cache.rank as u64,
        Some(Values(vec![Value::String(Some(code.to_owned()))])),
    ).await;
    // 开始生成nav
    let mut rank_urls = Vec::new();
    for (k, v) in RANK_NAV_ITEMS {
        rank_urls.push(RankUrls {
            title: v.to_string(),
            url:  get_config().rank_url(k),
            select: *k == key,
        });
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("title", &title);
    ctx.insert("rows", &rows);
    ctx.insert("rank_nav", &rank_urls);
    let template_path = format!("{}/rank.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}

pub(crate) async fn get_top(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError>{
    if !file_exists(format!("templates/{}/top.html", get_config().theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut rows:Vec<HashMap<&str,HashMap<&str,Vec<Novel>>>> = Vec::new();
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let conf = get_config();
    for (i, v) in conf.sort_arr.iter().enumerate() {
        let k = i + 1;
        let allvisit = format!("SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = {sort_id} ORDER BY allvisit DESC LIMIT 50;", filed = get_config().get_field(), table = get_config().prefix, where = get_config().get_where(), sort_id = k);
        let monthvisit = format!("SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = {sort_id} ORDER BY monthvisit DESC LIMIT 50;", filed = get_config().get_field(), table = get_config().prefix, where = get_config().get_where(), sort_id = k);
        let weekvisit = format!("SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = {sort_id} ORDER BY weekvisit DESC LIMIT 50;", filed = get_config().get_field(), table = get_config().prefix, where = get_config().get_where(), sort_id = k);
        let allvisit_rows = utils::redis::conn::get_cache_rows(
            allvisit,
            url,
            get_config().cache.rank as u64,
            None,
        ).await;
        let monthvisit_rows = utils::redis::conn::get_cache_rows(
            monthvisit,
            url,
            get_config().cache.rank as u64,
            None,
        ).await;
        let weekvisit_rows = utils::redis::conn::get_cache_rows(
            weekvisit,
            url,
            get_config().cache.rank as u64,
            None,
        ).await;
        let rank_map: HashMap<&str, Vec<Novel>> = HashMap::from([
            ("allvisit", allvisit_rows),
            ("monthvisit", monthvisit_rows),
            ("weekvisit", weekvisit_rows),
        ]);
        let mut sort_rank_map = HashMap::new();
        sort_rank_map.insert(v.caption.as_str(), rank_map);
        rows.push(sort_rank_map);
    }
    let mut rank_urls = Vec::new();
    for (k, v) in RANK_NAV_ITEMS {
        rank_urls.push(RankUrls {
            title: v.to_string(),
            url:  get_config().rank_url(k),
            select: false,
        });
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("rank_nav", &rank_urls);
    ctx.insert("rows", &rows);
    let template_path = format!("{}/top.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}