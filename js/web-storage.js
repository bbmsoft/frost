export const store_js = (key, value) => {
    const storage = window.localStorage;
    localStorage.setItem(key, value);
}

export const get_stored_js = (key) => {
    const storage = window.localStorage;
    return localStorage.getItem(key);
}