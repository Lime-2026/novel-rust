// Copyright (c) 2026 [Lime-2026]
// Licensed under Non-Profit Open Software License 3.0 (NPOSL-3.0)
// See LICENSE file in the project root for full license text.
mod app;
mod routes;
mod handlers;
mod models;
mod utils;
mod services;

use std::env;
use dotenv::dotenv;
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let app = routes::app::router().await;
    let port = env::var("PORT")
        .expect("请在.env文件中配置PORT");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",port)).await?;
    axum::serve(listener,app).await.expect("启动服务失败");
    Ok(())
}