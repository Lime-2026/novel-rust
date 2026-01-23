// --------------- 章节列表侧边栏逻辑 ---------------
const sidebar = document.getElementById('chaptersSidebar');
const mask = document.getElementById('mask');
const openBtn = document.getElementById('openSidebarBtn');
const closeBtn = document.getElementById('closeSidebarBtn');
// 阅读设置弹窗元素
const settingsModal = document.getElementById('settingsModal');
const openSettingsBtn = document.getElementById('openReadSettingBtn');
const closeSettingsBtn = document.getElementById('closeSettingsBtn');
const resetSettingsBtn = document.getElementById('resetSettingsBtn');
const fontSizeSlider = document.getElementById('fontSizeSlider');
const fontBtns = document.querySelectorAll('.font-btn');
const themeBtns = document.querySelectorAll('.theme-btn');
const chapterContent = document.querySelector('body');

const openSidebar = () => {
    sidebar.classList.add('show');
    mask.classList.add('show');
    document.body.style.overflow = 'hidden';
    settingsModal.classList.remove('show');
};

const closeSidebar = () => {
    sidebar.classList.remove('show');
    mask.classList.remove('show');
    document.body.style.overflow = '';
};

const openSettings = () => {
    settingsModal.classList.add('show');
    mask.classList.add('show');
    document.body.style.overflow = 'hidden';
    sidebar.classList.remove('show');
};

const closeSettings = () => {
    settingsModal.classList.remove('show');
    mask.classList.remove('show');
    document.body.style.overflow = '';
};

openBtn.addEventListener('click', openSidebar);
closeBtn.addEventListener('click', closeSidebar);
openSettingsBtn.addEventListener('click', openSettings);
closeSettingsBtn.addEventListener('click', closeSettings);
mask.addEventListener('click', (e) => {
    if (sidebar.classList.contains('show')) closeSidebar();
    if (settingsModal.classList.contains('show')) closeSettings();
});

const DEFAULT_SETTINGS = {
    fontSize: '18px',
    bgColor: '#ffffff',
    textColor: '#374151'
};

const saveSettings = (settings) => {
    localStorage.setItem('read_settings', JSON.stringify(settings));
};

const getSettings = () => {
    const saved = localStorage.getItem('read_settings');
    return saved ? JSON.parse(saved) : DEFAULT_SETTINGS;
};

// 应用设置到页面
const applySettings = (settings) => {
    console.log(settings)
    // 文本大小
    chapterContent.querySelector('.chapter-content').style.fontSize = settings.fontSize;
    fontSizeSlider.value = parseInt(settings.fontSize);
    fontBtns.forEach(btn => {
        btn.classList.toggle('active', btn.dataset.size === settings.fontSize);
    });
    // 背景和文本颜色
    setBG(chapterContent, settings)
    setBG(chapterContent.querySelector(`header`), settings)
    setBG(chapterContent.querySelector(`.mobile-nav`), settings)
    setBG(chapterContent.querySelector(`.chapter-content`), settings)
    setBG(chapterContent.querySelector(`.read-nav`), settings)
    setBG(chapterContent.querySelector(`.chapters-sidebar`), settings)
    setBG(chapterContent.querySelector(`.settings-modal`), settings)
    chapterContent.querySelectorAll(`.chapter-item`).forEach(item => {
            if (!item.classList.contains('active')) {
                item.style.setProperty("color", settings.textColor, 'important')
            }
        }
    )
    themeBtns.forEach(btn => {
        btn.classList.toggle('active', btn.dataset.bg === settings.bgColor);
    });
};

function setBG(current, settings) {
    current.style.setProperty("background-color", settings.bgColor, 'important');
    current.style.setProperty("color", settings.textColor, 'important');
}

// 文本大小控制
fontSizeSlider.addEventListener('input', () => {
    const fontSize = `${fontSizeSlider.value}px`;
    const current = getSettings();
    current.fontSize = fontSize;
    saveSettings(current);
    applySettings(current);
});

fontBtns.forEach(btn => {
    btn.addEventListener('click', () => {
        const fontSize = btn.dataset.size;
        const current = getSettings();
        current.fontSize = fontSize;
        saveSettings(current);
        applySettings(current);
    });
});

// 背景主题控制
themeBtns.forEach(btn => {
    btn.addEventListener('click', () => {
        const bgColor = btn.dataset.bg;
        const textColor = btn.dataset.text;
        const current = getSettings();
        current.bgColor = bgColor;
        current.textColor = textColor;
        saveSettings(current);
        applySettings(current);
    });
});

// 重置设置
resetSettingsBtn.addEventListener('click', () => {
    saveSettings(DEFAULT_SETTINGS);
    applySettings(DEFAULT_SETTINGS);
});

// 禁用按钮拦截
document.querySelectorAll('.nav-btn--disabled, .page-btn--disabled').forEach(btn => {
    btn.addEventListener('click', (e) => e.preventDefault());
});

// 移动端触摸关闭
document.addEventListener('touchstart', (e) => {
    if (sidebar.classList.contains('show') && !sidebar.contains(e.target)) closeSidebar();
    if (settingsModal.classList.contains('show') && !settingsModal.contains(e.target)) closeSettings();
});

document.addEventListener('DOMContentLoaded', () => {
    const saved = getSettings();
    applySettings(saved);
    // 强制隐藏弹窗和侧边栏
    sidebar.classList.remove('show');
    settingsModal.classList.remove('show');
    mask.classList.remove('show');
});