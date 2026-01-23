use std::fs::File;
use std::io::Read;
use once_cell::sync::Lazy;
use crate::models::config::Config;

pub(crate) static CONFIG: Lazy<Config> = Lazy::new(|| {
    load_config_sync().unwrap()
});

/// 加载 JSON 配置文件到 Config 结构体
pub fn load_config_sync() -> Result<Config, Box<dyn std::error::Error>> {
    // 1. 打开 JSON 文件
    let mut file = File::open("conf.json")?;
    // 2. 读取文件内容到字符串
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)?;
    // 3. 反序列化 JSON 到 Config 结构体
    let mut config: Config = serde_json::from_str(&json_str)?;
    for i in 0..config.sort_arr.len() {
        let code = config.sort_arr[i].code.clone(); // 或者 String/Copy 看你的字段类型
        let url = config.sort_url(code.as_str(), i + 1, 1);
        config.sort_arr[i].url = url;
    }
    Ok(config)
}