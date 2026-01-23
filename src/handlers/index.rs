use std::collections::HashMap;
use axum::extract::{OriginalUri, State};
use axum::http::header::HOST;
use axum::http::{HeaderMap};
use axum::response::IntoResponse;
use crate::{routes, utils, models, services};
use utils::templates::render;
use utils::redis::conn::{get_cache_rows};
use models::novel::Novel;
use crate::utils::conf::CONFIG;

pub async fn get_index(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, render::TeraRenderError> {
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let template_path = format!("{}/index.html", CONFIG.theme_dir);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())  // 安全转换为字符串
        .unwrap_or("unknown.host");     // 兜底值，避免 panic
    let commend = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} AND articleid IN ({val}) ORDER BY FIELD (articleid,{val})",filed=CONFIG.get_field(),table=CONFIG.prefix,val=CONFIG.commend_ids,where=CONFIG.get_where()),
        url,
        CONFIG.cache.home as u64,
        None,
    ).await;
    let popular = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY monthvisit DESC LIMIT 25",filed=CONFIG.get_field(),table=CONFIG.prefix),
        url,
        CONFIG.cache.home as u64,
        None,
    ).await;
    let mut sortarr: HashMap<u32, Vec<Novel>> = HashMap::new();
    for (k,_) in CONFIG.sort_arr.iter().enumerate() {
        let i = k as u32 + 1;
        let sql = format!(
            "SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = {} ORDER BY monthvisit DESC LIMIT 20",
            i,filed=CONFIG.get_field(),table=CONFIG.prefix,where=CONFIG.get_where()
        );
        let rows = get_cache_rows(
            sql,
            url,
            CONFIG.cache.home as u64,
            None,
        ).await;
        sortarr.insert(i, rows);
    }
    let lastupdate = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY lastupdate DESC LIMIT 30",filed=CONFIG.get_field(),table=CONFIG.prefix),
        url,
        CONFIG.cache.home as u64,
        None,
    ).await;
    let postdate = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY postdate DESC LIMIT 30",filed=CONFIG.get_field(),table=CONFIG.prefix),
        url,
        CONFIG.cache.home as u64,
        None,
    ).await;
    ctx.insert("commend", &commend);
    ctx.insert("postdate", &postdate);
    ctx.insert("sortarr", &sortarr);
    ctx.insert("lastupdate", &lastupdate);
    ctx.insert("popular", &popular);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}

