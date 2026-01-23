use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;
use crate::utils::text::{read_page_split, str_to_p};
use crate::{routes, services};
use axum::extract::{OriginalUri, Path, State};
use axum::http::header::HOST;
use axum::http::{HeaderMap};
use axum::response::IntoResponse;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;
use crate::utils::conf::CONFIG;
use crate::utils::file::file_exists;

static BR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)<br\s+[^>]*?/?>").expect("正则编译失败"));

#[derive(Deserialize)]
#[allow(dead_code)]
pub(crate) struct ChapterPath {
    sid: Option<u64>,
    id: u64,
    cid: Option<String>,
    s_cid: Option<String>,
    page: Option<String>,
}
pub(crate) async fn get_chapter(
    Path(p): Path<ChapterPath>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/chapter.html", CONFIG.theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let id = p.id;
    #[allow(unused)] let mut page = 1;
    #[allow(unused)] let mut cid = 0;
    let mut max_pid = 1;
    let mut next_page_url = String::new();
    let mut prev_page_url = String::new();
    // 判断是否船说兼容格式 _1
    if CONFIG.rewrite.chapter_url.contains("{s_cid}") {
        // 包含船说兼容 那么取s_cid值后判断是否包含_
        if let Some(s_cid) = p.s_cid {
            if s_cid.contains("_") {
                let s_cid_parts: Vec<&str> = s_cid.split('_').collect();
                cid = services::novel::extract_id(s_cid_parts[0]).ok_or(TeraRenderError::InvalidId)?;
                page = services::novel::extract_id(s_cid_parts[1]).ok_or(TeraRenderError::InvalidId)?;
            } else {
                page = 1;
                cid = services::novel::extract_id(&s_cid).ok_or(TeraRenderError::InvalidId)?;
            }
        }else {
            return Err(TeraRenderError::InvalidId);
        }
    } else {
        // 非兼容模式直接提取cid和page
        if let Some(sid) = p.cid {
            cid = services::novel::extract_id(&sid).ok_or(TeraRenderError::InvalidId)?;
        } else {    // 取不到值直接404
            return Err(TeraRenderError::InvalidId);
        }
        if let Some(r_page) = p.page {
            page = services::novel::extract_id(&r_page).ok_or(TeraRenderError::InvalidId)?;
        } else {
            page = 1;
        }
    }
    if page == 0 {
        // page允许为0，默认第一页
        page = 1;
    }
    if cid == 0 {
        // cid必须存在
        return Err(TeraRenderError::InvalidId);
    }
    let source_id =CONFIG.source_id(id);
    let source_chapter_id = CONFIG.source_id(cid);
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let row =
        services::novel::get_novel_info(url, CONFIG.cache.info, source_id)
            .await
            .into_iter()
            .next()
            .ok_or(TeraRenderError::InvalidId)?;
    let chapter_rows =
        services::novel::get_chapter_rows(url, CONFIG.cache.info, source_id)
            .await;
    let chapter = chapter_rows
        .iter()
        .find(|c| c.chapterid == cid)
        .ok_or(TeraRenderError::InvalidId)?;
    let chapter_index = chapter_rows
        .iter()
        .position(|c| c.chapterid == cid)
        .ok_or(TeraRenderError::InvalidId)?;
    let info_url = if CONFIG.is_3in1 {
        row.index_url.clone()
    } else {
        row.info_url.clone()
    };
    let next_url = if chapter_index == chapter_rows.len() - 1 {
        info_url.clone()
    } else {
        CONFIG.read_url(row.articleid, chapter_rows[chapter_index + 1].chapterid, 1)
    };
    let prev_url = if chapter_index == 0 {
        info_url.clone()
    } else {
        CONFIG
            .read_url(row.articleid, chapter_rows[chapter_index - 1].chapterid, 1)
    };
    // 拼接小说章节内容
    let txt_url = format!(
        "{}/{}/{}/{}.txt",
        CONFIG.txt_url,
        source_id / 1000,
        source_id,
        source_chapter_id
    );
    let mut chapter_content = services::novel::read_file(&txt_url).await;
    if chapter_content.is_empty() {
        chapter_content = "章节正在手打中，请稍后重新访问！".to_string();
    } else {
        chapter_content = BR_REGEX.replace_all(&chapter_content, "\n").to_string();
        match CONFIG.read_page_split_mode {
            1 => {  // 按行数分
                if CONFIG.read_page_split_lines
                    < chapter_content.split("\n").count() as u32
                {
                    (chapter_content, max_pid) =
                        read_page_split(&chapter_content, Some(CONFIG.read_page_split_lines as usize), Some(page as usize));
                    if page > max_pid {
                        return Err(TeraRenderError::InvalidId);
                    }
                    if page > 1 {
                        prev_page_url = CONFIG.read_url(
                            row.articleid,
                            cid,
                            page - 1,
                        )
                    }
                    if page < max_pid {
                        next_page_url = CONFIG.read_url(
                            row.articleid,
                            cid,
                            page + 1,
                        )
                    }
                }
            }
            2 => {  // 按字数分
                let per_page = CONFIG.read_page_split_lines as usize;
                let all_words = chapter_content.graphemes(true).count();
                let max_pid = all_words.div_ceil(per_page) as u64;
                if page > max_pid {
                    return Err(TeraRenderError::InvalidId);
                }
                let start = ((page - 1) as usize).saturating_mul(per_page);
                let page_text: String = chapter_content
                    .graphemes(true)
                    .skip(start)
                    .take(per_page)
                    .collect();
                chapter_content = str_to_p(&page_text);
                if page > 1 {
                    prev_page_url = CONFIG.read_url(
                        row.articleid,
                        cid,
                        page - 1,
                    )
                }
                if page < max_pid {
                    next_page_url = CONFIG.read_url(
                        row.articleid,
                        cid,
                        page + 1,
                    )
                }
            }
            _ => {
                chapter_content = str_to_p(chapter_content.as_str());
            }
        }
    }
    ctx.insert("chapter", chapter);
    ctx.insert("detail", &row);
    ctx.insert("chapters", &chapter_rows);
    ctx.insert("next_url", &next_url);
    ctx.insert("prev_url", &prev_url);
    ctx.insert("next_page_url", &next_page_url);
    ctx.insert("prev_page_url", &prev_page_url);
    ctx.insert("chapter_content", &chapter_content);
    ctx.insert("max_page", &max_pid);
    ctx.insert("page", &page);
    ctx.insert("info_url", &info_url);
    let template_path = format!("{}/chapter.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}
