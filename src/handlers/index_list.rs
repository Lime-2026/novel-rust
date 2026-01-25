use crate::models::novel::NovelChapter;
use crate::routes::app::AppState;
use crate::services::lang_tail::{gen_lang_tail, get_lang_tail_array};
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;
use crate::{services};
use axum::extract::{OriginalUri, Path, State};
use axum::http::HeaderMap;
use axum::http::header::HOST;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use crate::services::novel::{extract_id, get_novel_info};
use crate::utils::conf::get_config;

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct IndexListPath {
    sid: Option<u64>,
    id: u64,
    page: String,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct IndexListPageUrl {
    pub(crate) page: u64,
    pub(crate) url: String,
    pub(crate) select: bool,
}

pub(crate) async fn get_index_list(
    Path(p): Path<IndexListPath>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/index_list.html", get_config().theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let id = p.id;
    let page =extract_id(&p.page).ok_or(TeraRenderError::InvalidId)?;
    if page == 0 {
        return Err(TeraRenderError::InvalidId);
    }
    let source_id = get_config().source_id(id);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let row = services::novel::get_novel_info(url, get_config().cache.info, source_id)
        .await
        .into_iter()
        .next()
        .ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows = services::novel::get_chapter_rows(url, get_config().cache.info, source_id).await;
    let last_12 = &chapter_rows[chapter_rows.len().saturating_sub(12)..];
    let last_chapter: NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.last().unwrap().clone()
    };
    let first_chapter: NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.first().unwrap().clone()
    };
    // 获取总页码 要向上取整
    let total_page =
        (chapter_rows.len() + get_config().index_list_num as usize - 1) / get_config().index_list_num as usize;
    if page as usize > total_page {
        return Err(TeraRenderError::InvalidId);
    }
    // 根据page计算起始章节索引
    let start_index = (page - 1) * get_config().index_list_num as u64;
    let end_index = start_index + get_config().index_list_num as u64;
    // 安全的得到剪切的章节列表
    let end_index = end_index.min(chapter_rows.len() as u64);
    let cut_chapters = &chapter_rows[start_index as usize..end_index as usize];
    // 生成上一页 下一页页码
    let prev_url = if page > 1 {
        get_config().index_url(row.articleid, page - 1)
    } else {
        row.info_url.to_owned()
    };
    let next_url = if page < total_page as u64 {
        get_config().index_url(row.articleid, page + 1)
    } else {
        row.info_url.to_owned()
    };
    let mut page_urls = Vec::with_capacity(total_page);
    for i in 1..=total_page as u64 {
        page_urls.push(IndexListPageUrl {
            page: i,
            select: i == page,
            url: get_config().index_url(row.articleid, i),
        });
    }
    let lang_arr = get_lang_tail_array(source_id, url).await;
    // 处理长尾词生成
    if get_config().is_lang {
        // 交给协程
        gen_lang_tail(source_id, row.articlename.clone())
    }
    ctx.insert("prev_url", &prev_url);
    ctx.insert("next_url", &next_url);
    ctx.insert("detail", &row);
    ctx.insert("cut_chapters", &cut_chapters);
    ctx.insert("last_chapters", &last_12);
    ctx.insert("source_id", &source_id);
    ctx.insert("last_chapter", &last_chapter);
    ctx.insert("first_chapter", &first_chapter);
    ctx.insert("total_page", &total_page);
    ctx.insert("page_urls", &page_urls);
    ctx.insert("page", &page);
    ctx.insert("lang_arr", &lang_arr);
    let template_path = format!("{}/index_list.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}

pub(crate) async fn get_lang_index(
    Path(p): Path<IndexListPath>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/index_list.html", get_config().theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let lang_id = &p.id;
    let source_lang_id = get_config().source_id(*lang_id);
    if source_lang_id == 0 {
        return Err(TeraRenderError::InvalidId);
    }
    let page = extract_id(&p.page).ok_or(TeraRenderError::InvalidId)?;
    if page == 0 {
        return Err(TeraRenderError::InvalidId);
    }
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown.host");
    let lang_row = services::lang_tail::get_lang_tail(source_lang_id,url).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let mut row = get_novel_info(url, get_config().cache.info, lang_row.sourceid).await.into_iter().next().ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows = services::novel::get_chapter_rows(url, get_config().cache.info, lang_row.sourceid).await;
    let last_12 = &chapter_rows[chapter_rows.len().saturating_sub(12)..];
    let last_chapter: NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.last().unwrap().clone()
    };
    let first_chapter: NovelChapter = if chapter_rows.is_empty() {
        NovelChapter::default(row.info_url.as_str())
    } else {
        chapter_rows.first().unwrap().clone()
    };
    // 获取总页码 要向上取整
    let total_page =
        (chapter_rows.len() + get_config().index_list_num as usize - 1) / get_config().index_list_num as usize;
    if page as usize > total_page {
        return Err(TeraRenderError::InvalidId);
    }
    // 根据page计算起始章节索引
    let start_index = (page - 1) * get_config().index_list_num as u64;
    let end_index = start_index + get_config().index_list_num as u64;
    // 安全的得到剪切的章节列表
    let end_index = end_index.min(chapter_rows.len() as u64);
    let cut_chapters = &chapter_rows[start_index as usize..end_index as usize];
    // 生成上一页 下一页页码
    let prev_url = if page > 1 {
        get_config().index_url(row.articleid, page - 1)
    } else {
        row.info_url.to_owned()
    };
    let next_url = if page < total_page as u64 {
        get_config().index_url(row.articleid, page + 1)
    } else {
        row.info_url.to_owned()
    };
    let mut page_urls = Vec::with_capacity(total_page);
    for i in 1..=total_page as u64 {
        page_urls.push(IndexListPageUrl {
            page: i,
            select: i == page,
            url: get_config().index_url(row.articleid, i),
        });
    }
    let lang_arr = get_lang_tail_array(lang_row.sourceid,url).await;
    row.articlename = lang_row.langname;
    row.info_url = lang_row.info_url;
    row.index_url = lang_row.index_url;
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("prev_url", &prev_url);
    ctx.insert("next_url", &next_url);
    ctx.insert("detail", &row);
    ctx.insert("cut_chapters", &cut_chapters);
    ctx.insert("last_chapters", &last_12);
    ctx.insert("source_id", &lang_row.sourceid);
    ctx.insert("last_chapter", &last_chapter);
    ctx.insert("first_chapter", &first_chapter);
    ctx.insert("total_page", &total_page);
    ctx.insert("page_urls", &page_urls);
    ctx.insert("page", &page);
    ctx.insert("lang_arr", &lang_arr);
    let template_path = format!("{}/index_list.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}
