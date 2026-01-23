use std::env;
use std::sync::Arc;
use sea_orm::{Database, DatabaseConnection, DbErr};
use tokio::sync::OnceCell;

pub(crate) static DB_CONN: OnceCell<Arc<DatabaseConnection>> = OnceCell::const_new();

pub async fn init_conn() -> Result<Arc<DatabaseConnection>, DbErr> {
    let db_url = env::var("DATABASE_URL")
        .expect("请在.env文件中配置DATABASE_URL");
    let conn = Database::connect(db_url).await?;
    Ok(Arc::new(conn))
}

#[allow(dead_code)]
pub async fn get_db_conn() -> Result<Arc<DatabaseConnection>, DbErr> {
    // 只在第一次初始化时会执行 init_conn()
    let conn = DB_CONN
        .get_or_try_init(|| async { init_conn().await })
        .await?;

    Ok(conn.clone())
}

pub async fn get_db_conn_ref() -> &'static DatabaseConnection {
    let conn = DB_CONN
        .get_or_init(|| async {
            init_conn()
                .await
                .unwrap_or_else(|e| panic!("数据库连接失败: {e}"))
        })
        .await;

    &**conn
}