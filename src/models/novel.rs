use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, FromQueryResult,Serialize,Deserialize,Clone)]
#[allow(dead_code)]
pub(crate) struct NovelChapter {
    pub articleid: u64,
    pub chapterid: u64,
    pub chaptername: String,
    pub chaptertype: u8,
    pub chapterorder: u32,
    #[sea_orm(skip)]
    pub words: u32,
    pub lastupdate: u64,
    #[sea_orm(skip)]
    pub read_url: String,
    #[sea_orm(skip)]
    pub source_id: u64,
}

impl NovelChapter {
    pub(crate) fn default(info_url: &str) -> NovelChapter {
        NovelChapter {
            articleid: 0,
            chapterid: 0,
            chaptername: "暂无章节".to_string(),
            chaptertype: 0,
            chapterorder: 0,
            words: 0,
            lastupdate: 0,
            read_url: info_url.to_string(),
            source_id: 0,
        }
    }
}

#[derive(Debug, FromQueryResult,Serialize,Deserialize,Clone)]
#[allow(dead_code)]
pub(crate) struct Novel {
    pub articleid: u64,
    pub articlename: String,
    pub intro: String,
    pub author: String,
    pub sortid: u8,
    pub fullflag: bool,
    pub display: bool,
    pub lastupdate: u64,
    pub imgflag: bool,
    pub allvisit: u64,
    pub allvote: u64,
    pub goodnum: u64,
    pub keywords: String,
    pub lastchapter: String,
    pub lastchapterid: u64,
    #[sea_orm(ignore)]
    pub words:  u64,
    #[sea_orm(ignore)]
    pub articlecode: String,
    #[sea_orm(skip)]
    pub info_url: String,
    #[sea_orm(skip)]
    pub index_url: String,
    #[sea_orm(skip)]
    pub intro_des: String,
    #[sea_orm(skip)]
    pub author_url: String,
    #[sea_orm(skip)]
    pub sortname: String,
    #[sea_orm(skip)]
    pub sortname_2: String,
    #[sea_orm(skip)]
    pub sort_url: String,
    #[sea_orm(skip)]
    pub isfull: String,
    #[sea_orm(skip)]
    pub words_w: u64,
    #[sea_orm(skip)]
    pub lastupdate_cn: String,
    #[sea_orm(skip)]
    pub last_url: String,
    #[sea_orm(skip)]
    pub img_url: String,
    #[sea_orm(skip)]
    pub source_id: u64,
}


#[derive(Debug, FromQueryResult,Serialize,Deserialize,Clone)]
#[allow(dead_code)]
pub(crate) struct LangTail {
    pub langid: u64,
    pub langname: String,
    pub sourceid: u64,
    pub uptime: u64,
    #[sea_orm(skip)]
    pub info_url: String,
    #[sea_orm(skip)]
    pub index_url: String,
}
