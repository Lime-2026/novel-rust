use std::collections::HashMap;
use axum::extract::{OriginalUri, State};
use axum::http::header::HOST;
use axum::http::{HeaderMap};
use axum::response::IntoResponse;
use crate::{routes, utils, models, services};
use utils::templates::render;
use utils::redis::conn::{get_cache_rows};
use models::novel::Novel;
use crate::utils::conf::{get_config};
pub async fn get_index(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, render::TeraRenderError> {
    let template_path = format!("{}/index.html", get_config().theme_dir);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())  // 安全转换为字符串
        .unwrap_or("unknown.host");     // 兜底值，避免 panic
    let commend = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} AND articleid IN ({val}) ORDER BY FIELD (articleid,{val})",filed=get_config().get_field(),table=get_config().prefix,val=get_config().commend_ids,where=get_config().get_where()),
        url,
        get_config().cache.home as u64,
        None,
    ).await;
    let popular = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY monthvisit DESC LIMIT 25",filed=get_config().get_field(),table=get_config().prefix),
        url,
        get_config().cache.home as u64,
        None,
    ).await;
    let mut sortarr: HashMap<u32, Vec<Novel>> = HashMap::new();
    for (k,_) in get_config().sort_arr.iter().enumerate() {
        let i = k as u32 + 1;
        let sql = format!(
            "SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = {} ORDER BY monthvisit DESC LIMIT 20",
            i,filed=get_config().get_field(),table=get_config().prefix,where=get_config().get_where()
        );
        let rows = get_cache_rows(
            sql,
            url,
            get_config().cache.home as u64,
            None,
        ).await;
        sortarr.insert(i, rows);
    }
    let lastupdate = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY lastupdate DESC LIMIT 30",filed=get_config().get_field(),table=get_config().prefix),
        url,
        get_config().cache.home as u64,
        None,
    ).await;
    let postdate = get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article ORDER BY postdate DESC LIMIT 30",filed=get_config().get_field(),table=get_config().prefix),
        url,
        get_config().cache.home as u64,
        None,
    ).await;
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
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

