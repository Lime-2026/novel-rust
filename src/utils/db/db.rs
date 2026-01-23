use sea_orm::{ConnectionTrait, DbErr, FromQueryResult, Statement, Values};
use crate::utils::db::conn::get_db_conn_ref;

/// 根据语句取出计数结果 必须包含 cnt 别名
///
/// 例如：SELECT COUNT(*) AS cnt FROM table WHERE ...
pub async fn query_count(sql: &str, value: Option<Values>) -> Result<u64, DbErr> {
    let db = get_db_conn_ref().await;
    let stmt = match value {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let row_opt = db.query_one_raw(stmt).await?;
    let row = match row_opt {
        Some(r) => r,
        None => return Ok(0),
    };
    let cnt_i64: i64 = row.try_get("", "cnt")?;
    Ok(cnt_i64.max(0) as u64)
}


pub async fn exec_sql(sql: &str, value: Option<Values>) -> Result<u64, DbErr> {
    let db = get_db_conn_ref().await;
    let stmt = match value {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let res = db.execute_raw(stmt).await?;
    Ok(res.rows_affected())
}

/// 根据语句取出单条记录
///
/// 需要结构体实现 FromQueryResult
pub async fn get_one_as<T>(sql: &str, value: Option<Values>) -> Result<Option<T>, DbErr>
where
    T: FromQueryResult,
{
    let db = get_db_conn_ref().await;
    let stmt = match value {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let row_opt = db.query_one_raw(stmt).await?;
    Ok(row_opt.map(|row| T::from_query_result(&row, "")).transpose()?)
}

pub async fn query_all_as<T>(sql: &str, value: Option<Values>) -> Result<Vec<T>, DbErr>
where
    T: FromQueryResult,
{
    let db = get_db_conn_ref().await;
    let stmt = match value {
        Some(values) => Statement::from_sql_and_values(db.get_database_backend(), sql, values),
        None => Statement::from_string(db.get_database_backend(), sql),
    };
    let rows = db.query_all_raw(stmt).await?;
    let out: Vec<T> = rows
        .into_iter()
        .map(|row| T::from_query_result(&row, ""))
        .collect::<Result<Vec<T>, DbErr>>()?;
    Ok(out)
}
