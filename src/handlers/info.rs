use crate::utils::templates::render;
use crate::{routes, services};
use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap, Uri};
use axum::http::header::HOST;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use crate::handlers::index_list::{get_index_list, get_lang_index, IndexListPath};
use crate::models::novel::NovelChapter;
use crate::services::lang_tail::{gen_lang_tail, get_lang_tail_array};
use crate::utils::templates::render::TeraRenderError;
use crate::services::novel::{extract_id,process_tera_tag,get_novel_info,get_chapter_rows};
use crate::utils::conf::get_config;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct BookPath {
    sid: Option<u64>,
    id: String,
}

pub(crate) async fn get_info_3in1(
    Path(p): Path<BookPath>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    let id = extract_id(&p.id).ok_or(TeraRenderError::InvalidId)?;
    // 处理三合一模板
    if get_config().is_3in1 {
        get_index_list(IndexListPath{id: id.clone(),sid: p.sid.clone(),page: 1.to_string()},app_state,headers,uri).await
    } else {
        get_info(id,app_state,headers,uri).await
    }
}

pub(crate) async fn get_lang_info_3in1(
    Path(p): Path<BookPath>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    let id = extract_id(&p.id).ok_or(TeraRenderError::InvalidId)?;
    // 处理三合一模板
    if get_config().is_3in1 {
        get_lang_index(IndexListPath{id: id.clone(),sid: p.sid.clone(),page: 1.to_string()},app_state,headers,uri).await
    } else {
        get_lang(id,app_state,headers,uri).await
    }
}

pub(crate) async fn get_info(
    id: u64,
    app_state: routes::app::AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, TeraRenderError> {
    let source_id = get_config().source_id(id);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown.host");
    let row = get_novel_info(url, get_config().cache.info, source_id).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows = get_chapter_rows(url, get_config().cache.info, source_id).await;
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
    let lang_arr = get_lang_tail_array(source_id,url).await;
    // 处理长尾词生成
    if get_config().is_lang {
        // 交给协程
        gen_lang_tail(source_id,row.articlename.clone())
    }
    let mut ctx = tera::Context::new();
    process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("detail", &row);
    ctx.insert("chapters", &chapter_rows);
    ctx.insert("last_chapters", &last_12);
    ctx.insert("source_id", &source_id);
    ctx.insert("last_chapter", &last_chapter);
    ctx.insert("first_chapter", &first_chapter);
    ctx.insert("lang_arr", &lang_arr);
    let template_path = format!("{}/info.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ).into_response())
}

pub(crate) async fn get_lang(
    lang_id: u64,
    app_state: routes::app::AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, TeraRenderError> {
    let source_lang_id = get_config().source_id(lang_id);
    if source_lang_id == 0 {
        return Err(TeraRenderError::InvalidId);
    }
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown.host");
    let lang_row = services::lang_tail::get_lang_tail(source_lang_id,url).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let mut row = get_novel_info(url, get_config().cache.info, lang_row.sourceid).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows = get_chapter_rows(url, get_config().cache.info, lang_row.sourceid).await;
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
    let lang_arr = get_lang_tail_array(lang_row.sourceid,url).await;
    row.articlename = lang_row.langname;
    row.info_url = lang_row.info_url;
    row.index_url = lang_row.index_url;
    let mut ctx = tera::Context::new();
    process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("detail", &row);
    ctx.insert("chapters", &chapter_rows);
    ctx.insert("last_chapters", &last_12);
    ctx.insert("source_id", &lang_row.sourceid);
    ctx.insert("last_chapter", &last_chapter);
    ctx.insert("first_chapter", &first_chapter);
    ctx.insert("lang_arr", &lang_arr);
    let template_path = format!("{}/info.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ).into_response())
}