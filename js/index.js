function placesInitialized() {
    console.log("map initialized")
    import("../pkg/index.js").catch(console.error).then(wasm => console.log("wasm successfully loaded"))
}

if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/service-worker.js').then(registration => {
            console.log('SW registered: ', registration);
        }).catch(registrationError => {
            console.log('SW registration failed: ', registrationError);
        });
    });
} else {
    console.error("ServiceWorker not available!");
}