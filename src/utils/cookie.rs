use axum_extra::extract::cookie::CookieJar;
use cookie::{Cookie, SameSite, time::Duration};

#[allow(dead_code)]
pub fn get_cookie_value(jar: &CookieJar, key: &str) -> Option<String> {
    jar.get(key).map(|c| c.value().to_owned())
}

pub fn set_cookie_value(jar: CookieJar, key: &str, value: &str,age: usize,only: bool,secure: bool) -> CookieJar {
    let cookie = Cookie::build((key.to_owned(), value.to_owned()))
        .path("/")
        .max_age(Duration::seconds(age as i64))
        .http_only(only)
        .secure(secure)
        .same_site(SameSite::Lax)
        .build();
    jar.add(cookie)
}

#[allow(dead_code)]
pub fn remove_cookie(jar: CookieJar, key: &str) -> CookieJar {
    jar.remove(Cookie::from(key.to_owned()))
}

/// 判断cookie是否存在 存在返回true
pub fn is_cookie_exist(jar: &CookieJar, key: &str) -> bool {
    jar.get(key).is_some()
}
