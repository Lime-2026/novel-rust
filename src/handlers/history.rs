use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap};
use axum::response::IntoResponse;
use crate::{routes, services};
use crate::utils::conf::CONFIG;
use crate::utils::file::file_exists;
use crate::utils::templates::render;
use crate::utils::templates::render::TeraRenderError;

pub(crate) async fn get_history(
    State(app_state): State<routes::app::AppState>,
    headers: HeaderMap,
    OriginalUri(uri): OriginalUri,
) -> Result<impl IntoResponse, TeraRenderError> {
    if !file_exists(format!("templates/{}/history.html", CONFIG.theme_dir)) {
        println!("No such file or directory");
        return Err(TeraRenderError::InvalidId);
    }
    let mut ctx = tera::Context::new();
    services::novel::process_tera_tag(&headers, &uri, &mut ctx);
    let template_path = format!("{}/history.html", CONFIG.theme_dir);
    let html = render::render_template(app_state.tera.clone(), &template_path, ctx).await?;
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    ))
}
