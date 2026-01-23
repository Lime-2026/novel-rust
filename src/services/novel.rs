use std::io;
use std::io::Read;
use std::path::Path;
use axum::http::{HeaderMap, Uri};
use axum::http::header::HOST;
use encoding_rs::{Encoding, GBK};
use encoding_rs_io::DecodeReaderBytesBuilder;
use sea_orm::{DbErr, FromQueryResult, Statement, Value, Values};
use crate::handlers::define::NOVEL_CHAPTER_FILED;
use crate::models::novel::{Novel, NovelChapter};
use crate::utils;
use crate::utils::conf::CONFIG;
use crate::utils::db::conn::get_db_conn_ref;
use crate::utils::redis::conn::{get_cache_rows};
use crate::utils::text::time_to_cn;

const HTTP_PREFIX: &str = "http://";
const HTTPS_PREFIX: &str = "https://";

// 通用标签赋予
pub(crate) fn process_tera_tag(
    headers: &HeaderMap,
    uri: &Uri,
    ctx: &mut tera::Context,
) {
    let http_host = headers.get(HOST).map(|v| v.to_str().ok().unwrap_or("unknown.host")).unwrap_or("unknown.host");
    let request_uri = uri.to_string();
    ctx.insert("SITE_NAME", &CONFIG.site_name);
    ctx.insert("Uri", &request_uri);
    ctx.insert("SITE_URL", &http_host);
    ctx.insert("theme", &CONFIG.theme_dir);
}

pub(crate) fn extract_id(path: &str) -> Option<u64> {
    let last = path.split('/')
        .filter(|s| !s.is_empty())
        .last()?;
    let id_part = last.rsplit_once('.').map(|(a, _)| a).unwrap_or(last);
    id_part.parse::<u64>().ok()
}

pub(crate) fn extract_str(path: &str) -> Option<&str> {
    let last = path
        .split('/')
        .filter(|s| !s.is_empty())
        .last()?;

    Some(last.rsplit_once('.').map(|(a, _)| a).unwrap_or(last))
}

pub(crate) async fn get_novel_info(
    url: &str,
    cache: u32,
    source_id: u64,
) -> Vec<Novel> {
    get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} AND articleid = ? LIMIT 1;",filed=CONFIG.get_field(),table=CONFIG.prefix,where=CONFIG.get_where()),
        url,
        cache as u64,
        Some(Values(vec![Value::BigUnsigned(Some(source_id))])),
    ).await
}

pub(crate) async fn get_chapter_rows(
    url: &str,
    cache: u32,
    source_id: u64,
) -> Vec<NovelChapter> {
    crate::utils::redis::conn::get_chapter_rows(
        format!("SELECT {filed} FROM {table} WHERE articleid = ? ORDER BY chapterid ASC;",filed=NOVEL_CHAPTER_FILED,table=CONFIG.get_chapter_table(source_id)),
        url,
        cache as u64,
        Some(Values(vec![Value::BigUnsigned(Some(source_id))])),
    ).await
}

pub(crate) async fn read_file(path_or_url: &str) -> String {
    let s = path_or_url.trim();
    if s.starts_with(HTTP_PREFIX) || s.starts_with(HTTPS_PREFIX) {
        return utils::request::FETCHER.get_text(s).await.unwrap_or(String::new())
    }
    read_txt_to_utf8(s).unwrap_or(String::new())
}

pub fn read_txt_to_utf8<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    if let Ok(s) = std::str::from_utf8(&bytes) {
        let s = s.strip_prefix('\u{feff}').unwrap_or(s);
        return Ok(s.to_owned());
    }
    decode_bytes_with_encoding(&bytes, GBK)
}

fn decode_bytes_with_encoding(bytes: &[u8], encoding: &'static Encoding) -> io::Result<String> {
    let mut reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .build(bytes);
    let mut s = String::new();
    reader.read_to_string(&mut s)?;
    Ok(s)
}

/// 生成当前页前三、后三的页码列表（最多10个）
/// - page: 当前页码（≥1）
/// - max_page: 最大页码（≥1）
/// - 返回：有效页码的Vec<usize>
pub(crate) fn generate_pagination_numbers(page: usize, max_page: u64) -> Vec<usize> {
    let max_page = max_page as usize;
    if max_page == 0 {
        return vec![];
    }
    let mut start = page.saturating_sub(5);
    start = start.max(1);
    let mut end = start + 10;
    if end > max_page {
        end = max_page;
        start = end.saturating_sub(10);
        start = start.max(1);
    }
    (start..=end).collect()
}

pub(crate) async fn common_novel_random(
    url: &str,
    limit: u16,
    cache: u64
) -> Vec<Novel> {
    get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE articleid >= (SELECT FLOOR(RAND() * (SELECT MAX(articleid) FROM {table}article_article))) ORDER BY lastupdate DESC LIMIT {limit}", filed = CONFIG.get_field(), table = CONFIG.prefix, limit = limit),
        url,
        cache,
        None,
    ).await
}

#[allow(dead_code)]
static DEFAULT_NAME: &str = "其它类型";
#[allow(dead_code)]
static SERIALIZE: &str = "连载中";
#[allow(dead_code)]
static COMPLETE: &str = "已完结";
#[allow(dead_code)]
static DISCONTINUED: &str = "下架中";
#[allow(dead_code)]
static REVIEWED: &str = "已审核";
pub(crate) async fn query_novel_process(
    sql: &str,
    params: Option<Values>,
) -> Result<Vec<Novel>, DbErr> {
    let db = get_db_conn_ref().await;
    let stmt = match params {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let rows = Novel::find_by_statement(stmt)
        .all(db)
        .await
        .inspect_err(|e| eprintln!("query error = {:#?}", e))?;
    Ok(novel_mapping(rows))
}

pub(crate) async fn query_novel_chapter_process(
    sql: &str,
    params: Option<Values>,
) -> Result<Vec<NovelChapter>, DbErr> {
    let db = get_db_conn_ref().await;
    let stmt = match params {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let rows = NovelChapter::find_by_statement(stmt)
        .all(db)
        .await
        .inspect_err(|e| eprintln!("query error = {:#?}", e))?; // 这里直接打印错误
    Ok(novel_chapter_mapping(rows))
}

/// 常用 不考虑用map迭代
pub(crate) fn novel_chapter_mapping(mut rows: Vec<NovelChapter>) -> Vec<NovelChapter> {
    for row in &mut rows {
        let source_id = row.articleid;
        let chapter_id = row.chapterid;
        row.articleid = CONFIG.new_id(source_id);
        row.chapterid = CONFIG.new_id(chapter_id);
        row.read_url = CONFIG.read_url(row.articleid,row.chapterid,1);
        row.source_id = chapter_id;
    }
    rows
}

/// 常用 不考虑用map迭代
pub(crate) fn novel_mapping(mut rows: Vec<Novel>) -> Vec<Novel> {
    for row in &mut rows {
        let source_id = row.articleid;
        row.source_id = source_id;
        row.articleid = CONFIG.new_id(row.articleid);
        row.info_url = CONFIG.info_url(row.articleid);
        row.index_url = CONFIG.index_url(row.articleid,1);
        row.intro_des = utils::text::txt_200_des(&row.intro);
        row.author_url = CONFIG.author_url(&row.author);
        row.sortname = if let Some(sort_name) = CONFIG.get_sort_name(row.sortid) {
            String::from(sort_name)
        } else {
            String::from(DEFAULT_NAME)
        };
        row.sortname_2 = row.sortname.chars().skip(0).take(2).collect::<String>();
        row.isfull = if row.fullflag {
            String::from(COMPLETE)
        } else {
            String::from(SERIALIZE)
        };
        row.words_w = row.words / 10000;
        row.img_url = CONFIG.get_img_url(source_id,row.imgflag);
        row.lastupdate_cn = time_to_cn(row.lastupdate as i64);
        row.last_url = CONFIG.read_url(row.articleid,row.lastchapterid,1);
    }
    rows
}
