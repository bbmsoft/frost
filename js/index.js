if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/service-worker.js').then(registration => {
            console.log('SW registered: ', registration);
            loadWasm();
        }).catch(registrationError => {
            console.log('SW registration failed: ', registrationError);
            loadWasm();
        });
    });
} else {
    console.error("ServiceWorker not available!");
    loadWasm();
}

function loadWasm() {
    import("../pkg/index.js").catch(console.error);
}