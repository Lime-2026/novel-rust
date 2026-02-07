use std::fs::File;
use std::io::Read;
use std::sync::{Arc};
use aho_corasick::AhoCorasick;
use arc_swap::{ArcSwap, ArcSwapOption};
use once_cell::sync::{Lazy};
use crate::models::config::Config;

pub(crate) static CONFIG: Lazy<ArcSwap<Config>> =
    Lazy::new(|| ArcSwap::from_pointee(load_config_sync().unwrap()));

static FILTER_ENGINE: Lazy<ArcSwapOption<FilterEngine>> =
    Lazy::new(|| ArcSwapOption::from(None));

pub struct FilterEngine {
    replacements: Vec<String>,
    ac: AhoCorasick,
}

impl FilterEngine {
    pub fn from_rules(rules: &[(String, String)]) -> Option<Self> {
        if rules.is_empty() {
            return None;
        }
        let patterns: Vec<String> = rules.iter().map(|(t, _)| t.clone()).collect();
        let replacements: Vec<String> = rules.iter().map(|(_, r)| r.clone()).collect();
        let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
        let ac = AhoCorasick::new(&pattern_refs).ok()?; // 构建失败就 None
        Some(Self { replacements, ac })
    }

    pub fn apply(&self, text: &str) -> String {
        let mut out = String::new();
        self.ac.replace_all_with(text, &mut out, |mat, _m, dst| {
            let idx = mat.pattern().as_usize();
            dst.push_str(&self.replacements[idx]);
            true
        });
        out
    }
}

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
    if config.is_filter {
        set_filter(parse_replace_rules(&*config.filter.clone()));
    }
    Ok(config)
}

pub fn get_config() -> Arc<Config> {
    CONFIG.load_full()
}

pub fn set_config(config: Config) {
    let arc = Arc::new(config);
    CONFIG.store(arc.clone());
    if arc.is_filter {
        set_filter(parse_replace_rules(&*arc.filter.clone()));
    } else {
        clear_filter()
    }
}

pub fn set_filter(new_rules: Vec<(String, String)>) {
    let engine = FilterEngine::from_rules(&new_rules).map(Arc::new);
    FILTER_ENGINE.store(engine);
}

pub fn multi_replace(text: &str) -> String {
    if let Some(engine) = FILTER_ENGINE.load_full() {
        engine.apply(text)
    } else {
        text.to_string()
    }
}

pub fn clear_filter() {
    FILTER_ENGINE.store(None);
}

fn parse_replace_rules(rule_str: &str) -> Vec<(String, String)> {
    let mut rules = Vec::new();
    for line in rule_str.lines().filter(|line| !line.is_empty()) {
        let mut parts = line.splitn(2,"$$$");
        let target = parts.next().unwrap_or("");
        let replacement = parts.next().unwrap_or("");
        if target.is_empty() {
            continue;
        }
        rules.push((target.to_string(), replacement.to_string()));
    }
    rules
}
