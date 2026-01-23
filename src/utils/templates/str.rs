use std::collections::HashMap;
use chrono::{Local, TimeZone};
use tera::{Function, Result as TeraResult, Value};
use crate::utils::conf::CONFIG;

#[derive(Clone)]
pub struct RewriterFunction;
impl Function for RewriterFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let type_str = args
            .get("type")
            .ok_or_else(|| tera::Error::msg("获取哪种类型的伪静态是必须的(search | rank | top)"))?
            .as_str()
            .ok_or_else(|| tera::Error::msg("type 参数必须是字符串类型"))?;
        match type_str {
            "search" => Ok(Value::String(CONFIG.rewrite.search_url.clone())),
            "rank" => {
                let code = args
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("allvisit");
                Ok(Value::String(CONFIG.rank_url(code)))
            },
            "top" => Ok(Value::String(CONFIG.rewrite.top_url.clone())),
            "history" => Ok(Value::String(CONFIG.rewrite.history_url.clone())),
            _ => Err(tera::Error::msg(format!("未知的 type 参数值: {}", type_str))),
        }
    }
}

#[derive(Clone)]
pub struct SortArrayFunction;
impl Function for SortArrayFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        let arr = CONFIG
            .sort_arr
            .iter()
            .map(|s| serde_json::to_value(s).map_err(|e| tera::Error::msg(e.to_string())))
            .collect::<Result<Vec<Value>, _>>()?;
        Ok(Value::Array(arr))
    }
}

#[derive(Clone)]
pub struct SubstrFunction;
impl Function for SubstrFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // 1. 手动校验必选参数（替代 FunctionSignature 的作用）
        let title = args
            .get("name")
            .ok_or_else(|| tera::Error::msg("format_novel_title 缺少必选参数 name（小说标题）"))?
            .as_str()
            .ok_or_else(|| tera::Error::msg("name 参数必须是字符串类型"))?;

        // 2. 处理可选参数（手动设置默认值）
        let max_len = args
            .get("max_len")
            // 手动处理类型转换失败的情况
            .map(|v| v.as_u64().ok_or(tera::Error::msg("max_len 必须是数字")))
            // 转换成功则转为 usize，失败/无参数则用默认值 20
            .transpose()? // 把 Result<Option<u64>, Error> 转为 Option<Result<u64, Error>>
            .unwrap_or(20) as usize;

        // 3. 核心业务逻辑（和之前一致）
        let formatted_title = if title.len() > max_len {
            format!("{}...", &title[0..max_len])
        } else {
            format!("{}", title)
        };
        Ok(Value::String(formatted_title))
    }
}

#[derive(Clone)]
pub struct TimeFunction;

impl Function for TimeFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let default_timestamp = Local::now().timestamp();

        let timestamp = match args.get("time") {
            Some(Value::Number(num)) => {
                num.as_i64()
                    .ok_or_else(|| tera::Error::msg(format!(
                        "参数 `time` 必须是整数类型的时间戳，当前值：{}", num
                    )))?
            }
            Some(val) => return Err(tera::Error::msg(format!(
                "参数 `time` 必须是时间戳（数字），当前类型：{:?}", val
            ))),
            None => default_timestamp,
        };

        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d %H:%M:%S");

        let dt = Local.timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| tera::Error::msg(format!(
                "无效的时间戳：{}，无法转换为本地时间", timestamp
            )))?;
        let formatted_time = dt.format(format).to_string();
        Ok(Value::String(formatted_time))
    }
}