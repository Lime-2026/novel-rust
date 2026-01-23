use serde::{Deserialize, Serialize};
use crate::handlers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub(crate) site_name: String,
    pub(crate) site_url: String,
    pub(crate) txt_url: String,
    pub(crate) sys_ver: f32,
    pub(crate) root_dir: String,
    pub(crate) remote_img_url: String,
    pub(crate) enable_down: bool,
    pub(crate) theme_dir: String,
    pub(crate) commend_ids: String,
    pub(crate) is_3in1: bool,
    pub(crate) category_per_page: i8,
    pub(crate) read_page_split_lines: u32,
    pub(crate) vote_perday: i8,
    pub(crate) index_list_num: i16,
    pub(crate) rewrite: Rewrite,
    pub(crate) sort_arr: Vec<Sort>,
    pub(crate) is_multiple: bool,
    pub(crate) confusion_value: u64,
    pub(crate) confusion_algorithm: String,
    pub(crate) filter: String,
    pub(crate) link: String,
    pub(crate) is_report: bool,
    pub(crate) report_time: u32,
    pub(crate) prefix: String,
    pub(crate) search: Search,
    pub(crate) cache: Cache,
    pub(crate) read_page_split_mode: u8,
}

impl Config {

    pub(crate) fn rank_url(&self,code : &str) -> String {
        self.rewrite.rank_url.replace("{code}",code)
    }

    pub fn sort_url(&self,pinyin : &str,id :usize,page: usize) -> String {
        self.rewrite.sort_url.replace("{code}",pinyin).replace("{id}",&id.to_string()).replace("{page}",&page.to_string())
    }
    pub fn get_chapter_table(&self,id: u64) -> String {
        if self.sys_ver > 5.0 { // 如果大于等于5.0，则表示是分表章节
            return format!("{}article_chapter_{}",self.prefix,id / 10000);
        }
        format!("{}article_chapter",self.prefix)
    }
    pub fn get_field(&self) -> String {
        if self.sys_ver < 2.0 {
            return String::from(handlers::define::NOVEL_FIELD);
        }
        String::from(handlers::define::NOVEL_FIELD_2)
    }

    pub fn get_where(&self) -> String {
        if self.sys_ver < 2.0 {
            return String::from(handlers::define::NOVEL_WHERE);
        }
        String::from(handlers::define::NOVEL_WHERE_2)
    }
    pub fn new_id(&self, id: u64) -> u64 {
        if !self.is_multiple {
            return id;
        }
        match self.confusion_algorithm.as_str() {
            "+" => id + self.confusion_value,
            "*" => id * self.confusion_value,
            _ => id ^ self.confusion_value,
        }
    }

    pub fn source_id(&self, id: u64) -> u64 {
        if !self.is_multiple {
            return id;
        }
        match self.confusion_algorithm.as_str() {
            "+" => id - self.confusion_value,
            "*" => id / self.confusion_value,
            _ => id ^ self.confusion_value,
        }
    }

    pub fn info_url(&self, id: u64) -> String {
        self.rewrite
            .info_url
            .replace("{id}", &id.to_string())
            .replace("{sid}", &self.short_id(id).to_string())
    }

    pub fn read_url(&self, id: u64,cid: u64,page: u64) -> String {
        let s_cid = if page == 1 { format!("{}",cid) } else { format!("{}_{}",cid,page) };
        self.rewrite
            .chapter_url
            .replace("{id}", &id.to_string())
            .replace("{sid}", &self.short_id(id).to_string())
            .replace("{cid}", &cid.to_string())
            .replace("{s_cid}", s_cid.as_str())
            .replace("{page}", &page.to_string())
    }

    pub fn short_id(&self, id: u64) -> u64 {
        id / 1000
    }

    pub fn index_url(&self, id: u64,page: u64) -> String {
        self.rewrite
            .index_list_url
            .replace("{id}", &id.to_string())
            .replace("{page}", &page.to_string())
            .replace("{sid}", &self.short_id(id).to_string())
    }

    pub fn author_url(&self, name: &str) -> String {
        self.rewrite
            .author_url
            .replace("{name}", &*urlencoding::encode(name))
    }

    pub fn get_sort_name(&self, sort_id: u8) -> Option<&str> {
        let index = sort_id.saturating_sub(1) as usize;
        self.sort_arr.get(index).map(|s| s.caption.as_str())
    }

    pub fn get_img_url(&self, id: u64, img_flag: bool) -> String {
        if img_flag {
            return format!(
                "{url}/{short_id}/{id}/{id}s.jpg",
                url = self.remote_img_url,
                short_id = id / 1000,
                id = id
            );
        }
        format!("/static/{}/nocover.jpg",self.theme_dir)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rewrite {
    pub(crate) info_url: String,
    pub(crate) chapter_url: String,
    pub(crate) sort_url: String,
    pub(crate) top_url: String,
    pub(crate) rank_url: String,
    pub(crate) complete_url: String,
    pub(crate) history_url: String,
    pub(crate) index_list_url: String,
    pub(crate) author_url: String,
    pub(crate) search_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sort {
    pub(crate) code: String,
    pub(crate) caption: String,
    #[serde(default)]
    pub(crate) url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cache {
    pub(crate) home: u32,
    pub(crate) info: u32,
    pub(crate) chapter: u32,
    pub(crate) sort: u32,
    pub(crate) rank: u32,
    pub(crate) other: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Search {
    pub(crate) limit: u16,      // 搜索结果数
    pub(crate) min: u8,         // 最小搜索单位
    pub(crate) time: u32,       // 缓存时间
    pub(crate) is_record: bool, // 记录搜索词
    pub(crate) delay: i32,      // 间隔 -1 表示关闭搜索功能 0 表示无限制
}
