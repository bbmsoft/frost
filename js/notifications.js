function evalPermission(permission, onGranted, onDenied, onError) {

    if (!permission || permission == "default") {
        requestPermission(onGranted, onDenied, onError);
    } else if (permission == "granted") {
        onGranted();
    } else if (permission == "denied") {
        onDenied();
    } else {
        onError();
    }
}

async function requestPermission(onGranted, onDenied, onError) {
    if (checkNotificationPromise()) {
        const result = await Notification.requestPermission();
        permissionResult(result, onGranted, onDenied, onError);
    } else {
        Notification.requestPermission(result => permissionResult(result, onGranted, onDenied, onError));
    }
}

function permissionResult(permission, onGranted, onDenied, onError) {
    if (!Notification.permission) {
        Notification.permission = permission;
    }
    evalPermission(permission, onGranted, onDenied, onError);
}

function checkNotificationPromise() {
    try {
        Notification.requestPermission().then()
    } catch (e) {
        return false;
    }
    return true;
}

function cleanUp(notification) {
    document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'visible') {
            // The tab has become visible so clear the now-stale Notification.
            notification.close();
        }
    });
    window.addEventListener('onload', notification.close);
    window.addEventListener('onload', notification.close);
}

export function are_notifications_supported_js() {
    const notification = window.Notification
    return notification != null
}

export function request_notification_permission_js(onGranted, onDenied, onError) {

    if (!window.Notification) {
        onError();
    } else {
        evalPermission(Notification.permission, onGranted, onDenied, onError);
    }
}

export function show_notification_js(title, text, icon, tag) {
    show_notification_with_callbacks_js(title, text, icon, tag);
}

export function show_notification_with_callbacks_js(title, text, icon, tag, onClick, onError) {
    const notification = new Notification(title, { body: text, icon: icon, tag: tag });
    if (onClick) {
        notification.addEventListener("click", () => {
            notification.close();
            onClick();
        });
    }
    if (onError) {
        notification.addEventListener("error", onError);
    }
    cleanUp(notification);
}