let bookmax = 200;  // 最大历史记录数量

function LastRead(){this.bookList="bookList"}
LastRead.prototype={
    set:function(bid,uri,bookname,chaptername,author,img_url){
        if(!(bid&&uri&&bookname&&chaptername&&author&&img_url))return;
        const v = bid + '#' + uri + '#' + bookname + '#' + chaptername + '#' + author + '#' + img_url;
        let aBooks = lastread.getBook();
        const aBid = [];
        for (i=0; i<aBooks.length;i++){aBid.push(aBooks[i][0]);}
        if(aBid.indexOf(bid) !== -1){
            lastread.remove(bid);
        }else{
            while (aBooks.length >= bookmax) {
                lastread.remove(aBooks[0][0]);
                aBooks = lastread.getBook();
            }
        }
        this.setItem(bid,v);
        this.setBook(bid)
    },
    get:function(k){
        return this.getItem(k)?this.getItem(k).split("#"):"";
    },
    remove:function(k){
        this.removeItem(k);
        this.removeBook(k)
    },
    setBook:function(v){
        var reg=new RegExp("(^|#)"+v);
        var books =	this.getItem(this.bookList);
        if(books===""){
            books=v;
        } else{
            if(books.search(reg)===-1){
                books+="#"+v;
            } else{
                // 修复原版bug：replace结果未赋值，修改无效（必须加这行，否则重复bid不更新）
                books = books.replace(reg,"#"+v);
            }
        }
        this.setItem(this.bookList,books)
    },
    getBook:function(){
        const v = this.getItem(this.bookList) ? this.getItem(this.bookList).split("#") : Array();
        const books = Array();
        if(v.length){
            for(let i=0; i<v.length; i++){
                const tem = this.getItem(v[i]).split('#');
                if (tem.length>3)books.push(tem);
            }
        }
        return books
    },
    removeBook:function(v){
        const reg = new RegExp("(^|#)" + v);
        let books = this.getItem(this.bookList);
        if(!books){
            books="";
        } else{
            if(books.search(reg)!==-1){
                books=books.replace(reg,"");
            }
        }
        this.setItem(this.bookList,books)
    },
    setItem:function(k,v){
        if(!!window.localStorage){
            localStorage.setItem(k,v);
        } else{
            const expireDate = new Date();
            const EXPIRE_MONTH = 30 * 24 * 3600 * 1000;
            expireDate.setTime(expireDate.getTime()+12*EXPIRE_MONTH);
            document.cookie=k+"="+encodeURIComponent(v)+";expires="+expireDate.toGMTString()+"; path=/";
        }
    },
    getItem:function(k){
        let value = "";
        let result = "";
        if(!!window.localStorage){
            result=window.localStorage.getItem(k);
            value=result||"";
        } else{
            const reg = new RegExp("(^| )" + k + "=([^;]*)(;|\x24)");
            result = reg.exec(document.cookie);
            if(result){
                value=decodeURIComponent(result[2])||"";
            }
        }
        return value
    },
    removeItem:function(k){
        if(!!window.localStorage){
            window.localStorage.removeItem(k);
        } else{
            const expireDate = new Date();
            expireDate.setTime(expireDate.getTime()-1000);
            document.cookie=k+"= "+";expires="+expireDate.toGMTString();
        }
    },
    removeAll:function(){
        if(!!window.localStorage){
            window.localStorage.clear();
        } else{
            const v = this.getItem(this.bookList) ? this.getItem(this.bookList).split("#") : Array();
            if(v.length){
                for(const i in v ){
                    this.removeItem(v[i]);
                }
            }
            this.removeItem(this.bookList);
        }
    }
}

window.lastread = new LastRead();

function removebook(k) {
    lastread.remove(k);
    window.location.reload()
}

function removeall() {
    lastread.removeAll();
    window.location.reload()
}