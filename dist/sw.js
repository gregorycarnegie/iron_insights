const CACHE_NAME = 'iron-insights-v1';
const STATIC_ASSETS = [
    '/',
    '/analytics',
    '/1rm',
    '/sharecard',
    '/static/js/dist/plotly.min.js',
    '/static/js/dist/arrow.min.js',
    '/static/wasm/iron_insights_wasm.js',
    '/static/wasm/iron_insights_wasm_bg.wasm',
    '/static/js/lazy-loader.js'
];

const API_CACHE_DURATION = 5 * 60 * 1000; // 5 minutes

self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then(cache => cache.addAll(STATIC_ASSETS))
            .then(() => self.skipWaiting())
    );
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then(cacheNames => {
            return Promise.all(
                cacheNames
                    .filter(name => name !== CACHE_NAME)
                    .map(name => caches.delete(name))
            );
        }).then(() => self.clients.claim())
    );
});

self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);

    // Network-first for API calls
    if (url.pathname.startsWith('/api/')) {
        event.respondWith(
            fetch(request)
                .then(response => {
                    const clone = response.clone();
                    caches.open(CACHE_NAME).then(cache => {
                        cache.put(request, clone);
                    });
                    return response;
                })
                .catch(() => caches.match(request))
        );
        return;
    }

    // Cache-first for static assets
    event.respondWith(
        caches.match(request)
            .then(response => {
                if (response) return response;

                return fetch(request).then(response => {
                    // Cache successful responses
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(cache => {
                            cache.put(request, clone);
                        });
                    }
                    return response;
                });
            })
    );
});
