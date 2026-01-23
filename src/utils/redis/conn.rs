use std::sync::Arc;
use crate::models::novel::{Novel, NovelChapter};
use crate::utils;
use redis::{Client, AsyncCommands, RedisResult, ToRedisArgs};
use redis::aio::MultiplexedConnection;
use sea_orm::{Values};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Mutex, OnceCell};
use crate::services::novel::{query_novel_chapter_process, query_novel_process};

pub(crate) static REDIS_CONN: OnceCell<Option<Arc<Mutex<MultiplexedConnection>>>> = OnceCell::const_new();

pub async fn get_redis_conn() -> Option<Arc<Mutex<MultiplexedConnection>>> {
    let opt = REDIS_CONN
        .get_or_init(|| async { init_redis().await })
        .await;

    opt.clone()
}
/// --------------------------
/// 初始化Redis连接池
/// --------------------------
pub(crate) async fn init_redis() -> Option<Arc<Mutex<MultiplexedConnection>>> {
    let redis_url =
        std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:123456@localhost:6379/0".to_string());
    let client = match Client::open(redis_url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Redis Client 创建失败（非致命）：{}", e);
            return None;
        }
    };
    match client.get_multiplexed_async_connection().await {
        Ok(conn) => Some(Arc::new(Mutex::new(conn))),
        Err(e) => {
            eprintln!("Redis 连接失败（非致命）：{}", e);
            None
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn cache_set<V: ToRedisArgs + Send + Sync>(
    conn: &mut MultiplexedConnection,
    key: &str,
    value: V,
    ttl_secs: u64,
) -> RedisResult<()> {
    redis::cmd("SETEX")
        .arg(key)
        .arg(ttl_secs)
        .arg(value)
        .query_async(conn)
        .await
}

#[allow(dead_code)]
pub(crate) async fn get_cache<T: redis::FromRedisValue>(
    conn: &mut MultiplexedConnection,
    key: &str,
) -> RedisResult<Option<T>> {
    conn.get(key).await
}

pub(crate) async fn cache_set_json<T: Serialize>(
    redis: Option<&Arc<Mutex<MultiplexedConnection>>>,
    key: &str,
    value: &T,
    ttl_secs: u64,
) -> RedisResult<()> {
    let Some(redis) = redis else {
        return Ok(());
    };

    let s = match serde_json::to_string(value) {
        Ok(v) => v,
        Err(_) => return Ok(()), // 序列化失败不影响主流程
    };

    let mut conn = redis.lock().await;
    let _: () = conn.set_ex(key, s, ttl_secs).await?;
    Ok(())
}

pub(crate) async fn cache_get_json<T: DeserializeOwned>(
    redis: Option<&Arc<Mutex<MultiplexedConnection>>>,
    key: &str,
) -> RedisResult<Option<T>> {
    let Some(redis) = redis else {
        return Ok(None);
    };

    let mut conn = redis.lock().await;
    let s: Option<String> = conn.get(key).await?;
    Ok(s.and_then(|s| serde_json::from_str(&s).ok()))
}

pub(crate) async fn get_cache_count(
    sql: String,
    url: &str,
    cache_time: u64,
    value: Option<Values>,
) -> u64 {
    let key_seed = format!("{}|{}|{:?}", url, sql, value);
    let key = format!("novel:count:{:x}", md5::compute(key_seed));
    let redis = get_redis_conn().await;
    if let Some(ref redis_arc) = redis {
        if let Ok(Some(cnt)) = cache_get_json::<u64>(Some(redis_arc), &key).await {
            return cnt;
        }
    }
    let cnt: u64 = utils::db::db::query_count(sql.as_str(), value)
        .await
        .unwrap_or(0);
    if let Some(ref redis_arc) = redis {
        let _ = cache_set_json(Some(redis_arc), &key, &cnt, cache_time).await;
    }
    cnt
}


pub(crate) async fn get_cache_rows(
    sql: String,
    url: &str,
    cache_time: u64,
    value: Option<Values>,
) -> Vec<Novel> {
    let key_seed = format!("{}|{}|{:?}", url, sql, value);
    let key = format!("novel:rows:{:x}", md5::compute(key_seed));
    let redis = get_redis_conn().await;
    if let Some(ref redis_arc) = redis {
        if let Ok(Some(rows)) = cache_get_json::<Vec<Novel>>(Some(redis_arc), &key).await {
            return rows;
        }
    }
    let rs: Vec<Novel> = query_novel_process(sql.as_ref(), value)
        .await
        .unwrap_or_else(|_| Vec::new());
    if !rs.is_empty() {
        if let Some(ref redis_arc) = redis {
            let _ = cache_set_json(Some(redis_arc), &key, &rs, cache_time).await;
        }
    }
    rs
}

pub(crate) async fn get_chapter_rows(
    sql: String,
    url: &str,
    cache_time: u64,
    value: Option<Values>,
) -> Vec<NovelChapter> {
    let key_seed = format!("{}|{}|{:?}", url, sql, value);
    let key = format!("novel:chapters:{:x}", md5::compute(key_seed));
    let redis = get_redis_conn().await;
    if let Some(ref redis_arc) = redis {
        if let Ok(Some(rows)) = cache_get_json::<Vec<NovelChapter>>(Some(redis_arc), &key).await {
            return rows;
        }
    }
    let rs: Vec<NovelChapter> = query_novel_chapter_process(sql.as_ref(), value)
        .await
        .unwrap_or_else(|_| Vec::new());
    if !rs.is_empty() {
        if let Some(ref redis_arc) = redis {
            let _ = cache_set_json(Some(redis_arc), &key, &rs, cache_time).await;
        }
    }
    rs
}