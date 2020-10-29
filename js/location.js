export const is_geolocation_supported_js = () => {
    const navigator = window.navigator
    return navigator.geolocation != null
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