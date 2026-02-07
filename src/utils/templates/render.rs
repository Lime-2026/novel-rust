use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use tera::{Context, Tera};
use crate::utils::conf::{get_config, multi_replace};

#[derive(Debug)]
pub enum TeraRenderError {
    InvalidId,
    Render(String),
}

impl IntoResponse for TeraRenderError {
    fn into_response(self) -> Response {
        match self {
            TeraRenderError::InvalidId => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            TeraRenderError::Render(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("模板渲染失败：{msg}"),
            ).into_response(),
        }
    }
}


fn format_tera_error(e: &tera::Error) -> String {
    let mut out = format!("{:#?}\n", e);
    let mut cur: &dyn std::error::Error = e;
    while let Some(src) = cur.source() {
        out.push_str(&format!("Caused by: {:#}\n", src));
        cur = src;
    }
    out
}

// 渲染模板的工具函数
pub(crate) async fn render_template(
    tera: Arc<Tera>,
    template_name: impl Into<String>,
    ctx: Context,
) -> Result<Html<String>, TeraRenderError> {
    let template_name = template_name.into();
    let html = tera.render(&template_name, &ctx)
        .map_err(|e| {
            let detail = format_tera_error(&e);
            eprintln!("Tera render error detail:\n{}", detail);
            TeraRenderError::Render(e.to_string())
        })?;
    if get_config().is_filter {
        let filter_html = multi_replace(&html);
        return Ok(Html(filter_html))
    }
    Ok(Html(html))
}
