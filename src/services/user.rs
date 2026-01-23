use sea_orm::{DbErr, Value, Values};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::models::user::{BookShelf, BookShelfOnNovel, User};
use crate::services::novel::query_novel_process;
use crate::utils::conf::CONFIG;
use crate::utils::db::db::{exec_sql, get_one_as, query_all_as, query_count};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CreateUserError {
    #[error("用户名已经存在")]
    UsernameAlreadyExist,
    #[error("邮箱已经存在")]
    EmailAlreadyExist,
    #[error("创建用户失败,插入0条记录")]
    InsertFailed,
    #[error("未知错误")]
    Unknown,
    #[error("密码错误")]
    PasswordError,
    #[error("数据库错误：{0}")]
    DbErr(#[from] sea_orm::error::DbErr),
    #[error("用户不存在")]
    UserNotExist,
    #[error("登录认证失败")]
    LoginAuthFailed,
}

pub(crate) async fn create_user(
    username: &str,
    password: &str,
    email: &str,
) -> Result<User, CreateUserError> {
    let count = query_count(
        format!("SELECT COUNT(*) AS cnt FROM {table}system_users WHERE uname = ? ", table=CONFIG.prefix).as_str()
        ,Some(Values(vec![Value::String(Some(username.parse().unwrap()))]))
    ).await?;
    if count > 0 {
        return Err(CreateUserError::UsernameAlreadyExist);
    }
    let count = query_count(
        format!("SELECT COUNT(*) AS cnt FROM {table}system_users WHERE email = ? ", table=CONFIG.prefix).as_str()
        ,Some(Values(vec![Value::String(Some(email.parse().unwrap()))]))
    ).await?;
    if count > 0 {
        return Err(CreateUserError::EmailAlreadyExist);
    }
    let regdate = timestamp_10();
    let pass = format!("{:x}", md5::compute(password.as_bytes()));
    let (sql, values) = if CONFIG.sys_ver < 2.0 {
        (
            format!(
                "INSERT INTO {table}system_users (uname, name, pass, email, regdate) VALUES (?, ?, ?, ?, ?)",
                table = CONFIG.prefix
            ),
            Values(vec![
                Value::String(Some(username.to_owned())),
                Value::String(Some(username.to_owned())),
                Value::String(Some(pass)),
                Value::String(Some(email.to_owned())),
                Value::BigInt(Some(regdate)),
            ]),
        )
    } else {
        let u = Uuid::new_v4().to_string();
        let md5_hex = format!("{:x}", md5::compute(u.as_bytes()));
        let salt = md5_hex[md5_hex.len() - 16..].to_string();
        let raw = format!("{}{}", pass, salt);
        let pass_1 = format!("{:x}", md5::compute(raw.as_bytes()));
        (
            format!(
                "INSERT INTO {table}system_users (uname, name, pass, email, regdate, salt) VALUES (?, ?, ?, ?, ?, ?)",
                table = CONFIG.prefix
            ),
            Values(vec![
                Value::String(Some(username.to_owned())),
                Value::String(Some(username.to_owned())),
                Value::String(Some(pass_1)),
                Value::String(Some(email.to_owned())),
                Value::BigInt(Some(regdate)),
                Value::String(Some(salt)),
            ]),
        )
    };
    let num = exec_sql(
        &*sql,
        Option::from(values)
    ).await?;
    if num == 0 {
        return Err(CreateUserError::InsertFailed);
    }
    get_user(username, password).await
}

pub async fn is_user_login(
    user_id: &str,
    password: &str,
) -> Result<bool, CreateUserError> {
    let sql = if CONFIG.sys_ver < 2.0 {
        format!(
            "SELECT *,'' AS salt FROM {table}system_users WHERE uid = ? LIMIT 1",
            table = CONFIG.prefix
        )
    } else {
        format!(
            "SELECT * FROM {table}system_users WHERE uid = ? LIMIT 1",
            table = CONFIG.prefix
        )
    };
    let user = get_one_as::<User>(
        &sql,
        Some(Values(vec![Value::String(Some(user_id.to_owned()))])),
    )
        .await?
        .ok_or(CreateUserError::LoginAuthFailed)?;
    if user.pass != password {
        return Err(CreateUserError::LoginAuthFailed);
    }
    Ok(true)
}

pub async fn get_user(
    username: &str,
    password: &str,
) -> Result<User, CreateUserError> {
    let sql = if CONFIG.sys_ver < 2.0 {
        format!(
            "SELECT *,'' AS salt FROM {table}system_users WHERE uname = ? LIMIT 1",
            table = CONFIG.prefix
        )
    } else {
        format!(
            "SELECT * FROM {table}system_users WHERE uname = ? LIMIT 1",
            table = CONFIG.prefix
        )
    };
    let user = get_one_as::<User>(
        &sql,
        Some(Values(vec![Value::String(Some(username.to_owned()))])),
    )
        .await?
        .ok_or(CreateUserError::UserNotExist)?;
    let pass0 = format!("{:x}", md5::compute(password.as_bytes()));
    let expected = if CONFIG.sys_ver < 2.0 {
        pass0
    } else {
        let salt = user.salt.clone();
        let raw = format!("{}{}", pass0, salt);
        format!("{:x}", md5::compute(raw.as_bytes()))
    };

    if user.pass != expected {
        return Err(CreateUserError::PasswordError);
    }
    Ok(user)
}


/// 生成10位时间戳
pub fn timestamp_10() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

pub async fn get_bookcase_list(
    user_id: &str,
) -> Result<Vec<BookShelfOnNovel>, DbErr> {
    let sql = format!(
        "SELECT * FROM {table}article_bookcase WHERE userid = ?",
        table = CONFIG.prefix
    );
    let mut bs = query_all_as::<BookShelf>(
        &sql,
        Some(Values(vec![Value::String(Some(user_id.to_owned()))])),
    ).await?;
    if bs.is_empty() {
        return Ok(vec![]);
    }
    bookshelf_mapping(&mut bs);
    let mut vals = Vec::with_capacity(bs.len());
    for b in &bs {
        vals.push(Value::BigUnsigned(Some(b.articleid))); // 按你的实际类型换成 Value::Int / Value::String 等
    }
    let placeholders = std::iter::repeat("?")
        .take(bs.len())
        .collect::<Vec<_>>()
        .join(",");
    let novel_sql = format!("SELECT * FROM {table}article_article WHERE articleid IN ({}) ORDER BY lastupdate DESC",placeholders,table = CONFIG.prefix);
    let novel_list = query_novel_process(
        &novel_sql,
        Some(Values(vals)),
    ).await?;
    let ret: Vec<BookShelfOnNovel> = bs
        .into_iter()
        .filter_map(|b| {
            novel_list
                .iter()
                .find(|n| n.source_id == b.articleid)
                .cloned()
                .map(|novel| BookShelfOnNovel { case: b, novel })
        })
        .collect();
    Ok(ret)
}


pub(crate) fn bookshelf_mapping(bs: &mut [BookShelf]) {
    for b in bs {
        b.case_url = if b.chapterid == 0 {
            "".to_string()
        } else {
            let articleid = CONFIG.new_id(b.articleid);
            let chapterid = CONFIG.new_id(b.chapterid);
            CONFIG.read_url(articleid, chapterid, 1)
        };
    }
}

