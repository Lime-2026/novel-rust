pub(crate) static NOVEL_FIELD: &str = "'' as articlecode,size as words,articleid,articlename,intro,author,sortid,fullflag,display,lastupdate,imgflag,allvisit,allvote,goodnum,keywords,lastchapter,lastchapterid";
pub(crate) static NOVEL_FIELD_2: &str = "articlecode,words,articleid,articlename,intro,author,sortid,fullflag,display,lastupdate,imgflag,allvisit,allvote,goodnum,keywords,lastchapter,lastchapterid";
pub(crate) static NOVEL_WHERE: &str = "display <> 1 AND size >= 0 ";
pub(crate) static NOVEL_WHERE_2: &str = "display <> 1 AND words >= 0 ";
pub(crate) static NOVEL_CHAPTER_FILED: &str = "articleid,chapterid,chaptername,lastupdate,chaptertype,chapterorder";
