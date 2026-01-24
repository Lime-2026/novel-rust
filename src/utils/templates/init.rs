use std::fs;
use std::sync::Arc;
use tera::{Result as TeraResult, Tera};
use crate::utils::db::conn::{DB_CONN};
use crate::utils::templates::db::DbQueryTag;
use crate::utils::templates::str::{GETConfigFunction, RewriterFunction, SortArrayFunction, SubstrFunction, TimeFunction};

/// --------------------------
/// 初始化Tera模板引擎（全局复用）
/// --------------------------
pub fn init_tera() -> TeraResult<Arc<Tera>> {
    let mut tera = Tera::default();
    add_templates(&mut tera, "templates").expect("添加模板失败");
    tera.autoescape_on(Vec::new());
    tera.register_function("substr",SubstrFunction);
    let db_tag = DbQueryTag::new(DB_CONN.get().expect("DB 尚未初始化").clone());
    tera.register_function("db",db_tag);
    tera.register_function("date",TimeFunction);
    tera.register_function("sort_arr",SortArrayFunction);
    tera.register_function("rewrite",RewriterFunction);
    tera.register_function("conf",GETConfigFunction);
    Ok(Arc::new(tera))
}

fn add_templates(tera: &mut Tera, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_templates(tera, path.to_str().unwrap())?;
        } else if path.extension().map_or(false, |ext| ext == "html") {
            let content = fs::read_to_string(&path)?;
            let template_name = path.strip_prefix("templates/")?.to_str().unwrap().replace("\\", "/");
            tera.add_raw_template(template_name.as_str(), &content)?;
        }
    }
    Ok(())
}