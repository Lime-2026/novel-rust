use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use crate::models::novel::Novel;

#[derive(Debug, FromQueryResult,Serialize,Deserialize)]
#[allow(dead_code)]
pub(crate) struct User {
    pub(crate) uid: u64,
    pub(crate) uname: String,
    pub(crate) name: String,
    pub(crate) pass: String,
    pub(crate) email: String,
    pub(crate) salt: String,
}


#[derive(Debug, FromQueryResult,Serialize,Deserialize)]
#[allow(dead_code)]
pub(crate) struct BookShelf {
    pub(crate) caseid: u64,
    pub(crate) articleid: u64,
    pub(crate) articlename: String,
    pub(crate) chapterid: u64,
    pub(crate) chaptername: String,
    #[sea_orm(skip)]
    pub(crate) case_url: String,
}

#[derive(Debug, Serialize,Deserialize)]
#[allow(dead_code)]
pub(crate) struct BookShelfOnNovel{
    pub(crate) case: BookShelf,
    pub(crate) novel: Novel,
}