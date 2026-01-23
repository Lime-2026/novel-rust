
/// 通用用户登录请求校验 判断用户名和密码是否有一个为空
function validateLoginForm(form) {
    const username = form.username.value.trim();
    const password = form.password.value.trim();
    if (!username || !password) {
        alert("用户名和密码不能为空");
        return false;
    }
    return true;
}

function post_login(username, password) {
    const formData = new URLSearchParams();
    formData.append('username', username);
    formData.append('password', password);
    // 发起注册请求
    return fetch(
        "/login",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded",
            },
            body: formData,
        }
    ).then((response) => {
        if (!response.ok) {
            return `HTTP 错误：${response.status}`;
        }
        return response.json();
    }).then((data) => {
        if (data.success) return "success";
        else return data.errors.join("\n");
    }).catch((error) => {
        return "登录请求失败";
    });
}

function post_register(username, password, email) {
    const formData = new URLSearchParams();
    formData.append('username', username);
    formData.append('password', password);
    formData.append('email', email);
    return fetch(
        "/register",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded",
            },
            body: formData,
        }
    ).then((response) => {
        if (!response.ok) {
            return `HTTP 错误：${response.status}`
        }
        return response.json();
    }).then((data) => {
        if (data.success) return "success";
        else return data.errors.join("\n");
    }).catch((error) => {
        return "注册请求失败";
    });
}

/// 通用用户注册请求校验 判断用户账号密码邮箱是否合规
function validateRegisterForm(form) {
    const username = form.username.value.trim();
    const password = form.password.value.trim();
    if (!username || !password) {
        alert("用户名和密码不能为空");
        return false;
    }
    const email = form.email.value.trim();
    if (!email) {
        alert("邮箱不能为空");
        return false;
    }
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(email)) {
        alert("邮箱格式错误");
        return false;
    }
    // 校验用户名 用户名仅允许字母数字下划线还有@ 连接线
    const usernameRegex = /^[a-zA-Z0-9_@-]{6,32}$/;
    if (!usernameRegex.test(username)) {
        alert("用户名仅允许字母数字_@-  6-32位");
        return false;
    }
    // 密码长度至少6位
    if (password.length < 6) {
        alert("密码长度至少6位");
        return false;
    }
    return true;
}

/// 通用请求添加书架 根据返回内容是否success判断是否添加成功
// 如不需要默认alert提示 可以设置is_alert为false
// 请求不支持json格式 只能使用x-www-form-urlencoded格式
function add_bookshelf(
    articleid,
    articlename,
    chapterid,
    chaptername,
    is_alert = true
) {
    if (!chapterid) {
        chapterid = 0;
        chaptername = "";
    }
    const formData = new URLSearchParams();
    formData.append('articleid', articleid);
    formData.append('articlename', articlename);
    formData.append('chapterid', chapterid);
    formData.append('chaptername', chaptername);
    // 此处发起请求
    fetch(
        "/addbookcase",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded",
            },
            body: formData,
        }
    ).then((response) => {
        return response.json();
    }).then((data) => {
        if (data.success) {
            if(is_alert)alert("添加书架成功");
            return true;
        } else {
            if(is_alert)alert("添加书架失败");
            return false;
        }
    }).catch((error) => {
        if(is_alert)alert("添加书架失败");
        return false;
    })
}

/// 通用请求删除书架 根据返回内容是否success判断是否删除成功
// 如不需要默认alert提示 可以设置is_alert为false
// 请求不支持json格式 只能使用x-www-form-urlencoded格式
function del_bookshelf(
    caseid,
    is_alert = true
) {
    const formData = new URLSearchParams();
    formData.append('caseid', caseid);
    // 此处发起请求
    fetch(
        "/delbookcase",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/x-www-form-urlencoded",
            },
            body: formData,
        }
    ).then((response) => {
        return response.json();
    }).then((data) => {
        if (data.success) {
            if(is_alert)alert("删除书架成功");
            // 刷新页面
            window.location.reload();
            return true;
        } else {
            if(is_alert)alert("删除书架失败");
            return false;
        }
    }).catch((error) => {
        if(is_alert)alert("删除书架失败");
        return false;
    })
}
