const menuBtn = document.getElementById('menuBtn');
const mobileMenu = document.getElementById('mobileMenu');
const iconBars = menuBtn.querySelectorAll('.icon-bar');
menuBtn.addEventListener('click', function (e) {
    e.stopPropagation();
    mobileMenu.classList.toggle('show');
    iconBars.forEach((bar, index) => {
        if (mobileMenu.classList.contains('show')) {
            if (index === 0) bar.style.transform = 'translateY(8px) rotate(45deg)';
            if (index === 1) bar.style.opacity = '0';
            if (index === 2) bar.style.transform = 'translateY(-8px) rotate(-45deg)';
        } else {
            bar.style.transform = '';
            bar.style.opacity = '1';
        }
    });
});

function bindSortDropdown(triggerId) {
    const trigger = document.getElementById(triggerId);
    const content = trigger.nextElementSibling;

    if (!trigger || !content) return;

    trigger.addEventListener('click', (e) => {
        e.stopPropagation();
        content.classList.toggle('show');
        trigger.textContent = content.classList.contains('show') ? '分类 ▴' : '分类 ▾';
    });

    document.addEventListener('click', () => {
        content.classList.remove('show');
        trigger.textContent = '分类 ▾';
    });
    content.addEventListener('click', (e) => {
        e.stopPropagation();
    });
}

bindSortDropdown('desktopSortBtn');
bindSortDropdown('mobileSortBtn');

document.querySelectorAll('input[name="keyword"]').forEach(input => {
    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            input.closest('form').submit();
        }
    });
});

let tipTimer = null;
function showTip(content, type = 'info', duration = 3000) {
    const tipEl = document.getElementById('globalTip');
    if (!tipEl) return;
    if (tipTimer) {
        clearTimeout(tipTimer);
        tipTimer = null;
    }
    tipEl.textContent = content;
    tipEl.className = '';
    tipEl.classList.add(type);
    tipEl.classList.add('show');
    tipTimer = setTimeout(() => {
        tipEl.classList.remove('show');
        setTimeout(() => {
            tipEl.textContent = '';
        }, 300);
    }, duration);
}

function showSuccessTip(content) {
    showTip(content, 'success');
}
function showErrorTip(content) {
    showTip(content, 'error');
}
function showInfoTip(content) {
    showTip(content, 'info');
}

/**
 * 抽离cookie获取
 */
function get_cookie(key) {
    if (!key) return null;
    const reg = new RegExp(`(^| )${encodeURIComponent(key)}=([^;]*)(;|$)`);
    const match = document.cookie.match(reg);
    return match ? decodeURIComponent(match[2]) : null;
}

window.onload = function() {
    const loginDiv = document.querySelectorAll('header .login');
    loginDiv.forEach(item => {
        const parent = item.parentElement;
        let linkHtml = '';
        if(!get_cookie("ss_userid")) {
            linkHtml = `<a href="/login">登录</a><a href="/register">注册</a>`;
        } else {
            linkHtml = `<a href="/bookcase">书架</a><a href="/logout">退出登录</a>`;
        }
        const tempContainer = document.createElement('div');
        tempContainer.innerHTML = linkHtml;
        const linkElements = tempContainer.childNodes;
        item.remove();
        Array.from(linkElements).forEach(link => {
            parent.appendChild(link);
        });
    });
}