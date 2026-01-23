use std::fs::File;
use std::io::Read;
use once_cell::sync::Lazy;
use crate::models::config::Config;

pub(crate) static CONFIG: Lazy<Config> = Lazy::new(|| {
    load_config_sync().unwrap()
});

/// 加载 JSON 配置文件到 Config 结构体
pub fn load_config_sync() -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open("conf.json")?;
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)?;
    let mut config: Config = serde_json::from_str(&json_str)?;
    for i in 0..config.sort_arr.len() {
        let code = config.sort_arr[i].code.clone();
        let url = config.sort_url(code.as_str(), i + 1, 1);
        config.sort_arr[i].url = url;
    }
    Ok(config)
}