use std::collections::HashMap;
use chrono::{Local, TimeZone};
use tera::{Function, Result as TeraResult, Value};
use crate::utils::conf::get_config;
use rand::prelude::*;

const RANDOM_CHARS: [char; 36] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'
];
const RANDOM_LETTER_CHARS: [char; 62] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// 随机数
///
/// # 用法
/// - `length` 随机长度 (默认长度5)
/// - `letter` 大写字符是否参与随机 (默认true)
pub struct RandomStringFunction;
impl Function for RandomStringFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let letter = args
            .get("letter")
            .map(|v| v.as_bool().ok_or(tera::Error::msg("letter 必须是bool类型")))
            .transpose()?
            .unwrap_or(true);
        let length = args
            .get("length")
            .map(|v| v.as_u64().ok_or(tera::Error::msg("length 必须是number类型")))
            .transpose()?
            .unwrap_or(5) as usize;
        let mut rng = rand::rng();
        let mut result = String::with_capacity(length);
        if letter {
            for _ in 0..length {
                let c = RANDOM_LETTER_CHARS.choose(&mut rng).unwrap();
                result.push(*c);
            }
        } else {
            for _ in 0..length {
                let c = RANDOM_CHARS.choose(&mut rng).unwrap();
                result.push(*c);
            }
        }
        Ok(Value::String(result))
    }
}

pub struct LinkFunction;
impl Function for LinkFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        Ok(Value::String(get_config().link.clone()))
    }
}

pub struct StatCodeFunction;
impl Function for StatCodeFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        Ok(Value::String(get_config().stat_code.clone()))
    }
}
pub struct AdsFunction;
impl Function for AdsFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let key = args
            .get("key")
            .ok_or_else(|| tera::Error::msg("获取配置项的键是必须的"))?
            .as_str()
            .ok_or_else(|| tera::Error::msg("key 参数必须是字符串类型"))?;
        let value = get_config().ads
            .iter()
            .find(|v| v.pos == key)
            .map(|v| v.code.clone())
            .unwrap_or_else(|| "".to_string());
        Ok(Value::String(value))
    }
}
#[derive(Clone)]
pub struct GETConfigFunction;
impl Function for GETConfigFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let key = args
            .get("key")
            .ok_or_else(|| tera::Error::msg("获取配置项的键是必须的"))?
            .as_str()
            .ok_or_else(|| tera::Error::msg("key 参数必须是字符串类型"))?;
        // 出于性能考虑 此处不做任何序列化取值 默认返回空字符串
        match key {
            "is_lang" => Ok(Value::Bool(get_config().is_lang)),
            _ => Ok(Value::String("".to_string())),
        }
    }
}

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
            "search" => Ok(Value::String(get_config().rewrite.search_url.clone())),
            "rank" => {
                let code = args
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("allvisit");
                Ok(Value::String(get_config().rank_url(code)))
            },
            "top" => Ok(Value::String(get_config().rewrite.top_url.clone())),
            "history" => Ok(Value::String(get_config().rewrite.history_url.clone())),
            _ => Err(tera::Error::msg(format!("未知的 type 参数值: {}", type_str))),
        }
    }
}

#[derive(Clone)]
pub struct SortArrayFunction;
impl Function for SortArrayFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        let arr = get_config()
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
        let title = args
            .get("name")
            .ok_or_else(|| tera::Error::msg("format_novel_title 缺少必选参数 name（小说标题）"))?
            .as_str()
            .ok_or_else(|| tera::Error::msg("name 参数必须是字符串类型"))?;
        let max_len = args
            .get("max_len")
            .map(|v| v.as_u64().ok_or(tera::Error::msg("max_len 必须是数字")))
            .transpose()? 
            .unwrap_or(20) as usize;
        
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