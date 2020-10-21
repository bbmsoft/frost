export const set_cookie_js = (cookie) => {
    document.cookie = cookie;
}

export const get_cookies_js = () => {
    return decodeURIComponent(document.cookie);
}
