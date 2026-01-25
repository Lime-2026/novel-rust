use axum::extract::{OriginalUri, Path, State};
use axum::http::{HeaderMap};
use axum::http::header::HOST;
use axum::response::IntoResponse;
use sea_orm::{Value, Values};
use crate::{routes, services, utils};
use crate::utils::conf::{get_config};
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

pub(crate) async fn get_author(
    Path(mut author): Path<String>,
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/author.html", get_config().theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    author = urlencoding::decode(&author).unwrap().parse().unwrap();
    if author.is_empty() {
        return Err(TeraRenderError::InvalidId);
    }
    let url = headers
        .get(HOST)
        .and_then(|v| v.to_str().ok()) // 安全转换为字符串
        .unwrap_or("unknown.host");
    let rows = utils::redis::conn::get_cache_rows(
        format!("SELECT {filed} FROM {table}article_article WHERE {where} AND author = ? ORDER BY lastupdate DESC;", filed = get_config().get_field(), table = get_config().prefix, where = get_config().get_where()),
        url,
        get_config().cache.other as u64,
        Some(Values(vec![Value::String(Some(author.to_owned()))])),
    ).await;
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    ctx.insert("author", &author);
    ctx.insert("rows", &rows);
    let template_path = format!("{}/author.html", get_config().theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}