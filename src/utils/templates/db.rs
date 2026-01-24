use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement, Value as SeaValue,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

use sea_orm::sqlx::{Column, Row, TypeInfo};
use tera::{Error as TeraError, Function, Result as TeraResult, Value as TeraValue};

pub static TOKIO_RT: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("创建 Tokio 全局运行时失败"));

#[derive(Clone)]
pub struct DbQueryTag {
    db: Arc<DatabaseConnection>,
}

/// db 标签：查询数据库
///
/// # 参数
///
/// - `table`：表名（必选）
/// - `select`：查询字段（可选，默认 `*`）
/// - `where`：查询条件（可选）
/// - `and`：AND 条件（可选）
/// - `or`：OR 条件（可选）
/// - `order`：排序字段（可选）
/// - `limit`：限制返回行数（可选）
/// - `offset`：偏移量（可选）
impl DbQueryTag {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn parse_args(&self, args: &HashMap<String, TeraValue>) -> TeraResult<DbQueryConfig> {
        let table = args
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeraError::msg("db 标签缺少必选参数 table（表名）"))?
            .to_string();

        validate_ident(&table, "table")?;

        let select = args
            .get("select")
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| TeraError::msg("select 参数必须是字符串（如 'id,title' 或 '*'）"))
                    .map(|s| parse_select_list(s))
            })
            .transpose()?
            .unwrap_or_else(|| vec!["*".to_string()]);

        if !(select.len() == 1 && select[0] == "*") {
            for f in &select {
                validate_ident(f, "select 字段")?;
            }
        }

        let where_cond = self.parse_conditions(args, "where")?;
        let and_conds = self.parse_conditions(args, "and")?;
        let or_conds = self.parse_conditions(args, "or")?;

        // 校验条件 key
        validate_cond_keys(&where_cond, "where")?;
        validate_cond_keys(&and_conds, "and")?;
        validate_cond_keys(&or_conds, "or")?;

        let limit = args
            .get("limit")
            .map(|v| {
                v.as_u64()
                    .ok_or_else(|| TeraError::msg("limit 参数必须是非负整数"))
                    .map(|n| n as u32)
            })
            .transpose()?;

        let offset = args
            .get("offset")
            .map(|v| v.as_u64().ok_or_else(|| TeraError::msg("offset 参数必须是非负整数")))
            .transpose()?;

        let order = args
            .get("order")
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| TeraError::msg("order 参数必须是字符串（如 'id desc'）"))
                    .and_then(|s| parse_order(s))
            })
            .transpose()?;

        Ok(DbQueryConfig {
            table,
            select,
            where_cond,
            and_conds,
            or_conds,
            limit,
            offset,
            order,
        })
    }

    fn parse_conditions(
        &self,
        args: &HashMap<String, TeraValue>,
        key: &str,
    ) -> Result<Option<Map<String, Value>>, TeraError> {
        args.get(key)
            .map(|v| {
                if v.is_object() {
                    v.as_object()
                        .ok_or_else(|| TeraError::msg(format!("{key} 参数必须是 JSON 对象")))
                        .map(|m| m.clone())
                } else {
                    Err(TeraError::msg(format!("{key} 参数必须是 JSON 对象")))
                }
            })
            .transpose()
    }

    fn build_sql(&self, config: &DbQueryConfig) -> TeraResult<(String, Vec<SeaValue>)> {
        let mut sql_parts = Vec::new();
        let mut params = Vec::new();

        let select_fields = if config.select.len() == 1 && config.select[0] == "*" {
            "*".to_string()
        } else {
            config
                .select
                .iter()
                .map(|f| format!("`{}`", f))
                .collect::<Vec<_>>()
                .join(", ")
        };

        sql_parts.push(format!("SELECT {select_fields} FROM `{}`", config.table));

        let mut where_clauses = Vec::new();

        if let Some(m) = &config.where_cond {
            for (k, v) in m {
                where_clauses.push(format!("`{}` = ?", k));
                params.push(self.val_to_sea(v));
            }
        }

        if let Some(m) = &config.and_conds {
            for (k, v) in m {
                where_clauses.push(format!("`{}` = ?", k));
                params.push(self.val_to_sea(v));
            }
        }

        if let Some(m) = &config.or_conds {
            let mut or_parts = Vec::new();
            for (k, v) in m {
                or_parts.push(format!("`{}` = ?", k));
                params.push(self.val_to_sea(v));
            }
            if !or_parts.is_empty() {
                where_clauses.push(format!("({})", or_parts.join(" OR ")));
            }
        }

        if !where_clauses.is_empty() {
            sql_parts.push(format!("WHERE {}", where_clauses.join(" AND ")));
        }

        if let Some((field, order_type)) = &config.order {
            sql_parts.push(format!("ORDER BY `{}` {}", field, order_type.to_uppercase()));
        }

        if let Some(limit) = config.limit {
            sql_parts.push(format!("LIMIT {}", limit));
            if let Some(offset) = config.offset {
                sql_parts.push(format!("OFFSET {}", offset));
            }
        }
        Ok((sql_parts.join(" "), params))
    }

    async fn execute_sql(
        &self,
        sql: &str,
        params: &[SeaValue],
    ) -> Result<Vec<Map<String, TeraValue>>, DbErr> {
        let stmt = Statement::from_sql_and_values(DbBackend::MySql, sql, params.to_vec());
        let results = self.db.query_all_raw(stmt).await?;
        let mut tera_results = Vec::new();
        for res in results {
            let mysql_row = res
                .try_as_mysql_row()
                .ok_or_else(|| DbErr::Custom("不是 MySQL 后端，无法用 db 标签".into()))?;
            let columns = mysql_row.columns();
            let mut row_map = Map::<String, TeraValue>::new();
            for (idx, col) in columns.iter().enumerate() {
                let col_name = col.name().to_string();
                let type_name = normalize_mysql_type(col.type_info().name());

                let value = match type_name.as_str() {
                    "INT" | "SMALLINT" | "TINYINT" | "MEDIUMINT" | "BIGINT" => {
                        mysql_row.try_get::<Option<i64>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    "FLOAT" | "DOUBLE" | "DECIMAL" => {
                        mysql_row.try_get::<Option<f64>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    "BOOLEAN" | "BOOL" => {
                        mysql_row.try_get::<Option<bool>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    "VARCHAR" | "TEXT" | "CHAR" | "LONGTEXT" | "MEDIUMTEXT" | "TINYTEXT" => {
                        mysql_row.try_get::<Option<String>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    "JSON" => {
                        mysql_row.try_get::<Option<Value>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    "DATETIME" | "TIMESTAMP" | "DATE" | "TIME" => {
                        mysql_row.try_get::<Option<String>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                    _ => {
                        mysql_row.try_get::<Option<String>, _>(idx).ok().flatten()
                            .map(TeraValue::from).unwrap_or(TeraValue::Null)
                    }
                };

                row_map.insert(col_name, value);
            }

            tera_results.push(row_map);
        }

        Ok(tera_results)
    }

    fn val_to_sea(&self, val: &Value) -> SeaValue {
        if let Some(v) = val.as_i64() {
            return SeaValue::BigInt(Some(v));
        }
        if let Some(v) = val.as_u64() {
            return SeaValue::BigUnsigned(Some(v));
        }
        if let Some(v) = val.as_f64() {
            return SeaValue::Double(Some(v));
        }
        if let Some(v) = val.as_bool() {
            return SeaValue::Bool(Some(v));
        }
        if let Some(v) = val.as_str() {
            return SeaValue::String(Some(v.to_string()));
        }
        if val.is_null() {
            return SeaValue::Json(None);
        }
        SeaValue::Json(Some(Box::new(val.clone())))
    }
}

#[derive(Debug)]
struct DbQueryConfig {
    table: String,
    select: Vec<String>,
    where_cond: Option<Map<String, Value>>,
    and_conds: Option<Map<String, Value>>,
    or_conds: Option<Map<String, Value>>,
    limit: Option<u32>,
    offset: Option<u64>,
    order: Option<(String, String)>, // (field, "asc"/"desc")
}

impl Function for DbQueryTag {
    fn call(&self, args: &HashMap<String, TeraValue>) -> TeraResult<TeraValue> {
        let config = self.parse_args(args)?;
        let (sql, params) = self.build_sql(&config)?;

        let results = TOKIO_RT
            .block_on(async { self.execute_sql(&sql, &params).await })
            .map_err(|e| TeraError::msg(e.to_string()))?;

        Ok(TeraValue::Array(
            results.into_iter().map(TeraValue::Object).collect(),
        ))
    }
}

/* ---------------- 安全辅助函数 ---------------- */

fn is_safe_ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn validate_ident(s: &str, what: &str) -> TeraResult<()> {
    if is_safe_ident(s) {
        Ok(())
    } else {
        Err(TeraError::msg(format!(
            "{what} 不合法：只允许字母/数字/下划线，例如 novel、id、title"
        )))
    }
}

fn validate_cond_keys(m: &Option<Map<String, Value>>, where_name: &str) -> TeraResult<()> {
    if let Some(map) = m {
        for k in map.keys() {
            validate_ident(k, &format!("{where_name} 条件字段"))?;
        }
    }
    Ok(())
}

fn parse_select_list(s: &str) -> Vec<String> {
    let s = s.trim();
    if s == "*" {
        return vec!["*".to_string()];
    }
    s.split(',')
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .collect()
}

fn parse_order(s: &str) -> TeraResult<(String, String)> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return Err(TeraError::msg("order 参数格式错误：字段 排序（如 'id desc'）"));
    }
    let field = parts[0].to_string();
    let dir = parts[1].to_lowercase();

    validate_ident(&field, "order 字段")?;
    if dir != "asc" && dir != "desc" {
        return Err(TeraError::msg("order 排序只允许 asc 或 desc（如 'id desc'）"));
    }

    Ok((field, dir))
}

fn normalize_mysql_type(t: &str) -> String {
    t.split(|c| c == '(' || c == ' ')
        .next()
        .unwrap_or(t)
        .to_uppercase()
}
