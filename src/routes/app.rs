use std::sync::Arc;
use axum::{middleware, Router};
use axum::routing::{get, post};
use tera::Tera;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use crate::handlers::author::get_author;
use crate::handlers::chapter::get_chapter;
use crate::handlers::history::get_history;
use crate::handlers::index::{get_index};
use crate::handlers::index_list::{get_index_list, get_lang_index};
use crate::handlers::info::{get_info, get_lang};
use crate::handlers::rank::{get_rank, get_top};
use crate::handlers::search::{get_search, post_search};
use crate::handlers::sort::get_sort;
use crate::handlers::user::bookcase::{add_bookcase, del_bookcase, get_bookcase, login_auth};
use crate::handlers::user::login::{get_login, post_login};
use crate::handlers::user::register::{get_logout, get_register, post_register};
use crate::utils;
use crate::utils::conf::CONFIG;
use crate::utils::db::conn::{init_conn, DB_CONN};

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub(crate) tera: Arc<Tera>,
}

pub async fn router() -> Router {
    let db = init_conn().await.expect("初始化数据库连接失败");
    DB_CONN.set(db).expect("DB_CONN 已经初始化过了");
    let tera = utils::templates::init::init_tera().unwrap();
    let template_names: Vec<&str> = tera.get_template_names().collect();
    eprintln!("已加载模板：{:?}", template_names);
    let mut router = Router::new();
    if CONFIG.is_lang {
        router = router.route(trim_suffix(CONFIG.rewrite.lang_url.as_str()), get(get_lang))
            .route(trim_suffix(CONFIG.rewrite.lang_index_url.as_str()), get(get_lang_index));
    }
    router.route("/", get(get_index))
        .route(trim_suffix(CONFIG.rewrite.info_url.as_str()), get(get_info))
        .route(trim_suffix(CONFIG.rewrite.index_list_url.as_str()), get(get_index_list))
        .route(trim_suffix(CONFIG.rewrite.chapter_url.as_str()), get(get_chapter))
        .route(trim_suffix(CONFIG.rewrite.sort_url.as_str()).replace("{id}","{code}").as_str(), get(get_sort))    // 在注册的时候，将 {id} 替换为 {code}
        .route(trim_suffix(CONFIG.rewrite.author_url.as_str()), get(get_author))
        .route(trim_suffix(CONFIG.rewrite.rank_url.as_str()), get(get_rank))
        .route(trim_suffix(CONFIG.rewrite.top_url.as_str()), get(get_top))
        .route(CONFIG.rewrite.history_url.as_str(),get(get_history))
        .route(CONFIG.rewrite.search_url.as_str(), get(get_search).post(post_search))
        .route("/login", get(get_login).post(post_login))
        .route("/register", get(get_register).post(post_register))
        .route("/bookcase", get(get_bookcase).layer(middleware::from_fn(login_auth)))
        .route("/delbookcase", post(del_bookcase).layer(middleware::from_fn(login_auth)))
        .route("/addbookcase", post(add_bookcase).layer(middleware::from_fn(login_auth)))
        .route("/logout", get(get_logout))
        .nest_service("/static", ServeDir::new("public"))
        .layer(CompressionLayer::new())
        .with_state(AppState { tera})
}

fn trim_suffix(s: &str) -> &str {
    if !s.contains("}.") {
        return s;
    }
    match s.rfind('.') {
        Some(dot) => &s[..dot],
        None => s,
    }
}