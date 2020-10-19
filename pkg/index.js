export const set_cookie_js = (key, value, daysValid) => {
    var d = new Date();
    d.setTime(d.getTime() + (daysValid * 24 * 60 * 60 * 1000));
    var expires = "expires=" + d.toUTCString();
    document.cookie = key + "=" + value + ";" + expires;
}

export const get_cookie_js = key => {
    var name = key + "=";
    var decodedCookie = decodeURIComponent(document.cookie);
    var ca = decodedCookie.split(';');
    for (var i = 0; i < ca.length; i++) {
        var c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(name) == 0) {
            return c.substring(name.length, c.length);
        }
    }
    return null;
}

export const get_location_js = (onSuccess, onError) => {

    const navigator = window.navigator
    if (navigator.geolocation) {
        const geolocation = navigator.geolocation
        const success = position => onSuccess(position.coords.latitude, position.coords.longitude)
        const error = e => onError(e.code, e.message)
        const options = { timeout: 3000, enableHighAccuracy: false, maximumAge: 0 }
        geolocation.getCurrentPosition(success, error, options)
    } else {
        onError(4, "Geolocation not supported by device")
    }
}