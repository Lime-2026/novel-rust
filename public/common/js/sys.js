/**
 * 通用 Cookie 操作工具
 * 包含：设置、获取、删除、批量获取所有 Cookie 功能
 * 支持过期时间、路径、域名、Secure、SameSite 等配置
 */
const CookieUtil = {
    /**
     * 设置 Cookie
     * @param {string} key - Cookie 键名
     * @param {string} value - Cookie 值（自动编码）
     * @param {Object} [options] - 可选配置
     * @param {number} [options.expires] - 过期时间（秒），不传则为会话 Cookie（关闭浏览器失效）
     * @param {string} [options.path="/"] - Cookie 生效路径，默认根路径（/）
     * @param {string} [options.domain] - Cookie 生效域名，默认当前域名
     * @param {boolean} [options.secure=false] - 是否仅 HTTPS 下生效
     * @param {string} [options.sameSite="Lax"] - SameSite 属性（Strict/Lax/None），防止CSRF
     * @returns {void}
     */
    set: function (key, value, options = {}) {
        if (!key || typeof value === 'undefined') {
            console.warn('Cookie 设置失败：键名不能为空，值不能为 undefined');
            return;
        }
        const defaults = {
            path: '/',
            secure: false,
            sameSite: 'Lax' // 主流浏览器默认值，兼容大部分场景
        };
        const config = { ...defaults, ...options };
        let cookieStr = `${encodeURIComponent(key)}=${encodeURIComponent(value)}`;
        if (config.expires) {
            const expireDate = new Date();
            expireDate.setTime(expireDate.getTime() + config.expires * 1000);
            cookieStr += `; expires=${expireDate.toGMTString()}`;
        }
        if (config.path) {
            cookieStr += `; path=${config.path}`;
        }
        if (config.domain) {
            cookieStr += `; domain=${config.domain}`;
        }
        if (config.secure) {
            cookieStr += `; secure`;
        }
        if (config.sameSite) {
            cookieStr += `; SameSite=${config.sameSite}`;
        }
        document.cookie = cookieStr;
    },

    /**
     * 获取指定 Cookie 值
     * @param {string} key - Cookie 键名
     * @returns {string|null} 解码后的 Cookie 值，不存在则返回 null
     */
    get: function (key) {
        if (!key) return null;
        // 匹配规则：(^| )key=([^;]*)(;|$)
        const reg = new RegExp(`(^| )${encodeURIComponent(key)}=([^;]*)(;|$)`);
        const match = document.cookie.match(reg);
        return match ? decodeURIComponent(match[2]) : null;
    },

    /**
     * 删除指定 Cookie（通过设置过期时间为过去）
     * @param {string} key - Cookie 键名
     * @param {Object} [options] - 需和设置时的 path/domain 一致才能删除
     * @param {string} [options.path="/"] - 生效路径
     * @param {string} [options.domain] - 生效域名
     * @returns {void}
     */
    remove: function (key, options = {}) {
        // 复用 set 方法，设置过期时间为 -1 天（立即过期）
        this.set(key, '', {
            expires: -1,
            path: options.path || '/',
            domain: options.domain
        });
    },

    /**
     * 获取所有 Cookie，返回键值对对象
     * @returns {Object} 所有 Cookie 的键值对（已解码）
     */
    getAll: function () {
        const cookies = {};
        const cookieList = document.cookie.split('; ');

        for (const cookie of cookieList) {
            if (!cookie) continue;

            const [key, value] = cookie.split('=');
            if (key && value) {
                cookies[decodeURIComponent(key)] = decodeURIComponent(value);
            }
        }

        return cookies;
    },

    /**
     * 清空当前域名下所有 Cookie（仅能清空当前路径/域名可访问的 Cookie）
     * @returns {void}
     */
    clearAll: function () {
        const allCookies = this.getAll();
        Object.keys(allCookies).forEach(key => {
            this.remove(key);
        });
    }
};

