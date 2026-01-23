use std::borrow::Cow;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use html_escape::encode_text;

static RE_SPACE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"　{2,}|\s{2,}").expect("invalid regex")
});

pub(crate) fn str_to_p(txt: &str) -> String {
    let mut out = String::new();
    for line in txt.lines() {
        out.push_str("<p>");
        out.push_str(line); // 如需 HTML 转义在这里做
        out.push_str("</p>");
    }
    out
}

pub(crate) fn str_arr_to_p(lines: &[&str]) -> String {
    let mut result = String::new();
    result.reserve(lines.len() * 50);
    for line in lines {
        let cleaned: Cow<'_, str> = if line.contains("  ")
            || line.contains('\t')
            || line.contains("　　")
        {
            RE_SPACE.replace_all(line, " ")
        } else {
            Cow::Borrowed(*line)
        };

        let s = cleaned.as_ref();
        if s.trim().is_empty() {
            continue;
        }
        result.push_str("<p>");
        result.push_str(s);
        result.push_str("</p>");
    }

    result
}

/// 按行分割文本，每页多少行可配置
/// 每页多少行默认 20 行
/// page 表示当前用户访问的是第多少页 只渲染这个页
pub(crate) fn read_page_split(txt: &str, line: Option<usize>, page: Option<usize>) -> (String, u64) {
    let lines_per_page = line.unwrap_or(20);
    if lines_per_page == 0 {
        return (String::new(), 0);
    }
    let page = page.unwrap_or(1);
    if page == 0 {
        return (String::new(), 0);
    }
    let escaped_txt = html_special_chars_no_quotes(txt);
    let all_lines: Vec<&str> = escaped_txt.split('\n').collect();
    if all_lines.is_empty() {
        return (String::new(), 0);
    }
    let total_pages = all_lines.len().div_ceil(lines_per_page);
    if page > total_pages {
        return (String::new(), 0);
    }
    let start = (page - 1) * lines_per_page;
    let end = (start + lines_per_page).min(all_lines.len());
    (str_arr_to_p(&all_lines[start..end]), total_pages as u64)
}

fn html_special_chars_no_quotes(s: &'_ str) -> Cow<'_, str> {
    // 提前检查是否包含需要转义的字符，避免不必要的内存分配
    let needs_escape = s.contains('&') || s.contains('<') || s.contains('>');
    if !needs_escape {
        return Cow::Borrowed(s);
    }
    // 预分配足够的内存，减少扩容开销（原110%扩容比例保留，兼顾性能）
    let mut result = String::with_capacity((s.len() * 110) / 100);
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    Cow::Owned(result)
}

pub fn txt_200_des(txt: &str) -> String {
    let escaped = encode_text(txt); // Cow<'_, str>
    let collapsed = RE_SPACE.replace_all(&escaped, " ");
    collapsed.chars().take(200).collect()
}

pub fn time_to_cn(time: i64) -> String {
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let time = if time > now_ts { now_ts } else { time };
    let diff = now_ts.saturating_sub(time);
    if diff < 2 * 60 {
        "刚刚".to_string()
    } else if diff < 60 * 60 {
        format!("{}分钟前", diff / 60)
    } else if diff < 60 * 60 * 24 {
        format!("{}小时前", diff / (60 * 60))
    } else if diff < 60 * 60 * 24 * 30 {
        format!("{}天前", diff / (60 * 60 * 24))
    } else if diff < 60 * 60 * 24 * 365 {
        format!("{}个月前", diff / (60 * 60 * 24 * 30))
    } else {
        let dt = match Utc.timestamp_opt(time, 0) {
            chrono::LocalResult::Single(dt) => dt,
            _ => Utc::now(),
        };
        dt.format("%Y-%m-%d").to_string()
    }
}
