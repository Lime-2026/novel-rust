use std::{io::Read, sync::Arc, time::Duration};

use encoding_rs::{Encoding, GB18030, GBK, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;
use mime::Mime;
use once_cell::sync::Lazy;
use rand::Rng;
use reqwest::StatusCode;
use thiserror::Error;
use tokio::sync::Semaphore;
use url::Url;

pub(crate) static FETCHER: Lazy<HttpFetcher> = Lazy::new(|| {
    // timeout=30s, global_in_flight=800（你可以按机器内存调）
    HttpFetcher::new(30, 800).expect("init HttpFetcher failed")
});

#[derive(Error, Debug)]
pub enum HttpRequestError {
    #[error("无效的URL：{0}")]
    InvalidUrl(String),

    #[error("不支持的URL协议：{0}（仅支持 http/https）")]
    UnsupportedScheme(String),

    #[error("Reqwest错误：{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("请求返回非成功状态码：{0}")]
    NonSuccessStatusCode(StatusCode),

    #[error("读取响应内容失败：{0}")]
    ReadBodyError(String),

    #[error("编码解析失败：{0}")]
    EncodingError(String),
}

/// 解析并校验 URL（只允许 http/https）
fn parse_http_url(url: &str) -> Result<Url, HttpRequestError> {
    let url = url.trim();
    let parsed = Url::parse(url).map_err(|e| HttpRequestError::InvalidUrl(e.to_string()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(HttpRequestError::UnsupportedScheme(other.to_string())),
    }
}

/// charset label -> Encoding
fn encoding_from_charset_label(label: &str) -> &'static Encoding {
    let lower = label.trim().trim_matches('"').to_ascii_lowercase();
    if lower == "gb2312" || lower == "gbk" {
        return GBK;
    }
    if lower == "gb18030" {
        return GB18030;
    }
    Encoding::for_label(lower.as_bytes()).unwrap_or(UTF_8)
}

/// 从响应头 Content-Type 的 charset 获取编码；取不到则默认 UTF-8（宽容）
fn get_response_encoding(response: &reqwest::Response) -> &'static Encoding {
    let Some(ct) = response.headers().get(reqwest::header::CONTENT_TYPE) else {
        return UTF_8;
    };
    let Ok(cts) = ct.to_str() else { return UTF_8; };
    let Ok(mime) = cts.parse::<Mime>() else { return UTF_8; };

    if let Some(cs) = mime.get_param("charset") {
        return encoding_from_charset_label(cs.as_str());
    }
    UTF_8
}

/// bytes -> String（按指定 encoding 解码）
fn decode_bytes(bytes: &[u8], encoding: &'static Encoding) -> Result<String, HttpRequestError> {
    let mut reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .build(bytes);

    let mut s = String::new();
    reader
        .read_to_string(&mut s)
        .map_err(|e| HttpRequestError::EncodingError(format!("解码失败：{}", e)))?;
    Ok(s)
}

async fn read_response_text(resp: reqwest::Response) -> Result<String, HttpRequestError> {
    // 先取 encoding（借用 resp）
    let enc = get_response_encoding(&resp);
    // 再取 bytes（move resp）
    let bytes = resp.bytes().await?;
    decode_bytes(bytes.as_ref(), enc)
}

/// 只做全局并发限制的 Fetcher（不限制单域名）
pub struct HttpFetcher {
    client: reqwest::Client,
    global_sem: Arc<Semaphore>,
    // 重试参数
    retry_times: usize,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl HttpFetcher {
    /// timeout_secs：单请求超时
    /// global_in_flight：全局在途上限（建议按内存算：200KB+ 响应，先从 300~1000 试）
    pub fn new(timeout_secs: u64, global_in_flight: usize) -> Result<Self, HttpRequestError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(200)
            .build()?;

        Ok(Self {
            client,
            global_sem: Arc::new(Semaphore::new(global_in_flight)),
            retry_times: 2,      // 默认重试 2 次（总尝试 3 次）
            base_backoff_ms: 80,
            max_backoff_ms: 1500,
        })
    }

    fn should_retry(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    async fn backoff_sleep(&self, attempt: usize) {
        let exp = 1u64.checked_shl(attempt.min(10) as u32)
            .unwrap_or(u64::MAX);
        let mut ms = self.base_backoff_ms.saturating_mul(exp).min(self.max_backoff_ms);
        let jitter: u64 = rand::rng().random_range(0..=ms / 2 + 1);
        ms += jitter;
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }

    pub async fn get_text(&self, url: &str) -> Result<String, HttpRequestError> {
        let parsed = parse_http_url(url)?;
        let _permit = self.global_sem.clone().acquire_owned().await.unwrap();
        for attempt in 0..=self.retry_times {
            let resp = self.client.get(parsed.clone()).send().await?;
            if resp.status().is_success() {
                return read_response_text(resp).await;
            }
            let st = resp.status();
            if attempt < self.retry_times && Self::should_retry(st) {
                self.backoff_sleep(attempt).await;
                continue;
            }
            return Err(HttpRequestError::NonSuccessStatusCode(st));
        }
        Err(HttpRequestError::ReadBodyError("unexpected retry loop exit".into()))
    }


    #[allow(dead_code)]
    pub async fn post_json_text<T: serde::Serialize>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<String, HttpRequestError> {
        let parsed = parse_http_url(url)?;
        let _permit = self.global_sem.clone().acquire_owned().await.unwrap();
        for attempt in 0..=self.retry_times {
            let resp = self.client.post(parsed.clone()).json(body).send().await?;
            if resp.status().is_success() {
                return read_response_text(resp).await;
            }
            let st = resp.status();
            if attempt < self.retry_times && Self::should_retry(st) {
                self.backoff_sleep(attempt).await;
                continue;
            }
            return Err(HttpRequestError::NonSuccessStatusCode(st));
        }
        Err(HttpRequestError::ReadBodyError("unexpected retry loop exit".into()))
    }
}
