use once_cell::sync::Lazy;
use sea_orm::{ConnectionTrait, DbErr, FromQueryResult, Statement, Value};
use crate::models::novel::{LangTail};
use crate::utils::conf::get_config;
use crate::utils::db::conn::get_db_conn_ref;
use crate::utils::redis::conn::{cache_get_json, cache_set_json, get_redis_conn};
use crate::utils::templates::db::TOKIO_RT;
use std::collections::{HashSet};
use std::sync::{Mutex};
use regex::Regex;
use crate::services::user::timestamp_10;
use crate::utils::request::FETCHER;

static RUNNING: Lazy<Mutex<HashSet<u64>>> = Lazy::new(|| Mutex::new(HashSet::new()));
struct Guard(u64);
impl Drop for Guard {
    fn drop(&mut self) {
        if let Ok(mut set) = RUNNING.lock() {
            set.remove(&self.0);
        }
    }
}

#[allow(dead_code)]
struct EngineInfo {
    url:String,
    regex :Regex,
    name:String,
}

static ENGINE: Lazy<Vec<EngineInfo>> = Lazy::new(|| {
    vec![
        EngineInfo{
            url: "https://sug.so.360.cn/suggest?encodein=utf-8&encodeout=utf-8&format=json&word={key}&callback=window.so.sug".to_owned(),
            regex:  Regex::new(r#""word":"([^"]+)""#).unwrap(),
            name: "360".to_owned(),
        },
        EngineInfo{
            url: "https://api.bing.com/qsonhs.aspx?type=cb&q={key}".to_owned(),
            regex: Regex::new(r#""Txt":"([^"]+)""#).unwrap(),
            name: "bing".to_owned(),
        },
        EngineInfo{
            url: "https://so.toutiao.com/2/article/search_sug/?keyword={key}&in_tfs=&aid=&pd=synthesis&ps_type=sug".to_owned(),
            regex: Regex::new(r#""keyword":"([^"]+)""#).unwrap(),
            name: "toutiao".to_owned(),
        },
        EngineInfo{
            url: "https://suggestion.baidu.com/su?wd={key}".to_owned(),
            regex: Regex::new(r#"s:\[([^]]+)]"#).unwrap(),
            name: "baidu".to_owned(),
        },
    ]
});

pub(crate) async fn get_lang_tail(
    source_id: u64,
    url: &str,
) -> Result<LangTail, DbErr> {
    let sql = format!("SELECT langid,sourceid,langname,uptime FROM {}article_langtail WHERE langid = ?", get_config().prefix);
    let key_seed = format!("{}|{}|{:?}", url, sql, source_id);
    let key = format!("novel:langtail:{:x}", md5::compute(key_seed));
    let redis = get_redis_conn().await;
    if let Some(ref redis_arc) = redis {
        if let Ok(Some(rows)) = cache_get_json::<LangTail>(Some(redis_arc), &key).await {
            return Ok(rows);
        }
    }
    let db = get_db_conn_ref().await;
    let stmt =  Statement::from_sql_and_values(db.get_database_backend(), sql, Some(Value::BigUnsigned(Option::from(source_id.clone()))));
    let rows = LangTail::find_by_statement(stmt).one(db).await;
    if let Ok(Some(mut row)) = rows {
        if let Some(ref redis_arc) = redis {
            let _ = cache_set_json(Some(redis_arc), &key, &row, get_config().cache.info as u64).await;
        }
        Ok(mapping_langtail(&mut row))
    } else {
        Err(rows.err().unwrap())
    }
}

pub(crate) async fn get_lang_tail_array(
    article_id: u64,
    url: &str,
) -> Vec<LangTail> {
    if !get_config().is_lang {
        return Vec::new();
    }
    let sql = format!("SELECT langid,sourceid,langname,uptime FROM {}article_langtail WHERE sourceid = ?", get_config().prefix);
    let key_seed = format!("{}|{}|{:?}", url, sql, article_id);
    let key = format!("novel:langtail:rows:{:x}", md5::compute(key_seed));
    let redis = get_redis_conn().await;
    if let Some(ref redis_arc) = redis {
        if let Ok(Some(rows)) = cache_get_json::<Vec<LangTail>>(Some(redis_arc), &key).await {
            return rows;
        }
    }
    let db = get_db_conn_ref().await;
    let stmt =  Statement::from_sql_and_values(db.get_database_backend(), sql, Some(Value::BigUnsigned(Option::from(article_id.clone()))));
    let mut rows = LangTail::find_by_statement(stmt).all(db).await.unwrap_or_else(|_e| {
        eprintln!("get_lang_tail_array error={:?} article_id={:?}", _e, article_id);
        Vec::new()
    });
    if !rows.is_empty() {
        if let Some(ref redis_arc) = redis {
            let _ = cache_set_json(Some(redis_arc), &key, &rows, get_config().cache.info as u64).await;
        }
    }
     mapping_langtail_array(&mut rows)
}

/// 将长尾生成丢在后台任务，不阻塞主线程
pub(crate) fn gen_lang_tail(article_id: u64, article_name: String) {
    {
        let mut set = RUNNING.lock().unwrap();
        if !set.insert(article_id) {
            return;
        }
    }
    TOKIO_RT.spawn_blocking(move || {
        let _g = Guard(article_id);
        let r = TOKIO_RT.block_on(gen_lang_tail_impl(article_id, article_name));
        if let Err(e) = r {
            eprintln!("gen_lang_tail panic article_id={}: {:?}", article_id, e);
        }
    });
}

async fn gen_lang_tail_impl(article_id: u64, article_name: String) -> Result<(), DbErr> {
    let db = get_db_conn_ref().await;
    let sql = format!("SELECT uptime FROM {}article_langtail WHERE sourceid = ? ORDER BY uptime DESC LIMIT 1", get_config().prefix);
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        [Value::BigUnsigned(Some(article_id))],
    );
    if let Ok(Some(row)) = LangTail::find_by_statement(stmt).one(db).await {
        let last_uptime = row.uptime + (7 * 86400);
        let this_time = timestamp_10() as u64;
        if last_uptime >= this_time {
            return Ok(());
        }
    }
    let mut lang_name_arr: HashSet<String> = HashSet::new();
    for engine in ENGINE.iter() {
        let url = engine.url.replace("{key}", &*urlencoding::encode(&article_name));
        let res = FETCHER.get_text(&url).await.unwrap_or_default();
        for caps in engine.regex.captures_iter(&res) {
            if let Some(m) = caps.get(1) {
                if engine.name == "baidu" {
                    let lang_arr = m.as_str().to_owned().replace("\"","");
                    for lang in lang_arr.split(',') {
                        lang_name_arr.insert(lang.to_owned());
                    }
                    continue;
                }
                lang_name_arr.insert(m.as_str().to_owned());
            }
        }
    }
    if lang_name_arr.is_empty() {
        return Ok(());
    }
    batch_upsert_langtail(article_id,article_name, lang_name_arr).await?;
    Ok(())
}

async fn batch_upsert_langtail(article_id: u64,article_name :String, names: HashSet<String>) -> Result<(), DbErr> {
    let db = get_db_conn_ref().await;
    let now = timestamp_10() as u64;
    let mut values_sql = String::new();
    let mut vals: Vec<Value> = Vec::with_capacity(names.len() * 3);
    for (i, name) in names.into_iter().enumerate() {
        if name.is_empty() || name == article_name{
            continue;
        }
        if i > 0 { values_sql.push(','); }
        values_sql.push_str("(?, ?, ?)");
        vals.push(Value::BigUnsigned(Some(article_id)));
        vals.push(Value::String(Option::from(name.clone())));
        vals.push(Value::BigUnsigned(Some(now)));
    }
    let sql = format!(
        "INSERT INTO {}article_langtail (sourceid, langname, uptime) VALUES {} \
         ON DUPLICATE KEY UPDATE uptime = VALUES(uptime)",
        get_config().prefix,
        values_sql
    );
    let stmt = Statement::from_sql_and_values(db.get_database_backend(), sql, vals);
    db.execute_raw(stmt).await.map(|_| ()).map_err(|e| e.into())
}

fn mapping_langtail(row: &mut LangTail) -> LangTail {
    let new_id = get_config().new_id(row.langid);
    row.info_url =  get_config().lang_info_url(new_id);
    row.index_url = get_config().lang_index_url(new_id,1);
    row.clone()
}

fn mapping_langtail_array(rows: &mut Vec<LangTail>) -> Vec<LangTail> {
    rows.iter_mut().map(|row| mapping_langtail(row)).collect()
}
