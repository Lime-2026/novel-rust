use crate::utils::templates::render;
use crate::{routes, services};
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap};
use axum::http::header::HOST;
use axum::response::IntoResponse;
use serde::Deserialize;
use crate::models::novel::NovelChapter;
use crate::utils::conf::CONFIG;
use crate::utils::templates::render::TeraRenderError;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct BookPath {
    sid: Option<u64>,
    id: String,
}

pub(crate) async fn get_info(
    Path(p): Path<BookPath>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let id = services::novel::extract_id(&p.id).ok_or(TeraRenderError::InvalidId)?;
    let source_id = CONFIG.source_id(id);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())  // 安全转换为字符串
        .unwrap_or("unknown.host");
    let row = services::novel::get_novel_info(url, CONFIG.cache.info, source_id).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows = services::novel::get_chapter_rows(url, CONFIG.cache.info, source_id).await;
    let last_12 = &chapter_rows[chapter_rows.len().saturating_sub(12)..];
    let last_chapter:NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.last().unwrap().clone()
    };
    let first_chapter:NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.first().unwrap().clone()
    };
    ctx.insert("detail", &row);
    ctx.insert("chapters", &chapter_rows);
    ctx.insert("last_chapters", &last_12);
    ctx.insert("source_id", &source_id);
    ctx.insert("last_chapter", &last_chapter);
    ctx.insert("first_chapter", &first_chapter);
    let template_path = format!("{}/info.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}
