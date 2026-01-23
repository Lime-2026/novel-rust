use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap};
use axum::http::header::HOST;
use axum::response::IntoResponse;
use sea_orm::{Value, Values};
use serde::{Deserialize};
use crate::{routes, services, utils};
use crate::handlers::index_list::IndexListPageUrl;
use crate::services::novel::generate_pagination_numbers;
use crate::utils::conf::CONFIG;
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

#[derive(Deserialize)]
pub(crate) struct SortPath {
    code: Option<String>,
    page: Option<String>,
}

pub(crate) async fn get_sort(
    Path(p): Path<SortPath>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/sort.html", CONFIG.theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut code = p.code.ok_or(TeraRenderError::InvalidId)?;
    #[allow(unused)] let mut sort_id = CONFIG.sort_arr.len().saturating_sub(1);
    let mut page = 1;
    if let Some(sid) = p.page {
        page = services::novel::extract_id(&sid).ok_or(TeraRenderError::InvalidId)?;
    }
    if CONFIG.rewrite.sort_url.contains("{id}") {
        sort_id = services::novel::extract_id(&code).ok_or(TeraRenderError::InvalidId)? as usize;
    } else {
        code = services::novel::extract_str(&code).ok_or(TeraRenderError::InvalidId)?.parse().unwrap();
        sort_id = CONFIG.sort_arr.iter()
            .position(|s| s.code == code)
            .ok_or(TeraRenderError::InvalidId)?;
    }
    if sort_id >= CONFIG.sort_arr.len() {
        return Err(TeraRenderError::InvalidId);
    }
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let sort = &CONFIG.sort_arr[sort_id];
    let offset = (page - 1).saturating_mul(CONFIG.category_per_page as u64);
    let count = utils::redis::conn::get_cache_count(
        format!("SELECT COUNT(*) AS cnt FROM {table}article_article WHERE {where} AND sortid = ?;", table = CONFIG.prefix, where = CONFIG.get_where()),
        url,
        CONFIG.cache.sort as u64,
        Some(Values(vec![Value::TinyInt(Some((sort_id + 1) as i8))])),
    ).await;
    let max_page = count.div_ceil(CONFIG.category_per_page as u64).max(1);
    if page > max_page {    // 如不需要超出边界404 可注释
        return Err(TeraRenderError::InvalidId);
    }
    let rows = utils::redis::conn::get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} AND sortid = ? ORDER BY lastupdate DESC LIMIT {limit} OFFSET ?;", filed = CONFIG.get_field(), table = CONFIG.prefix, where = CONFIG.get_where(), limit = CONFIG.category_per_page),
        url,
        CONFIG.cache.sort as u64,
        Some(Values(vec![((sort_id + 1) as i8).into(), offset.into()])),
    ).await;
    let prev_url = if page > 1 {
        CONFIG.sort_url(sort.code.as_str(), sort_id + 1, (page - 1) as usize)
    } else {
        String::new()
    };
    let next_url = if page < max_page {
        CONFIG.sort_url(sort.code.as_str(), sort_id + 1, (page + 1) as usize)
    } else {
        String::new()
    };
    let mut jump_pages:Vec<IndexListPageUrl> = Vec::new();
    let j = generate_pagination_numbers(page as usize, max_page);
    for p in j {
        jump_pages.push(IndexListPageUrl {
            page: p as u64,
            url: CONFIG.sort_url(sort.code.as_str(), sort_id + 1, p),
            select: p == page as usize,
        });
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("sort", sort);
    ctx.insert("sort_id", &sort_id);
    ctx.insert("rows", &rows);
    ctx.insert("prev_url", &prev_url);
    ctx.insert("next_url", &next_url);
    ctx.insert("page", &page);
    ctx.insert("max_page", &max_page);
    ctx.insert("jump_pages", &jump_pages);
    let template_path = format!("{}/sort.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}