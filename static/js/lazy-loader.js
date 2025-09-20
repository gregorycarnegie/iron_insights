/**
 * Lazy Loading Module for Iron Insights
 * Dynamically loads JavaScript libraries based on page requirements
 */

class LazyLoader {
    constructor() {
        this.loadedScripts = new Set();
        this.loadingPromises = new Map();
    }

    /**
     * Load a script dynamically and return a promise
     * @param {string} src - Script source URL
     * @param {Object} options - Loading options
     * @returns {Promise} Promise that resolves when script is loaded
     */
    loadScript(src, options = {}) {
        // Return existing promise if already loading
        if (this.loadingPromises.has(src)) {
            console.log(`📋 Script already loading: ${src}`);
            return this.loadingPromises.get(src);
        }

        // Return resolved promise if already loaded
        if (this.loadedScripts.has(src)) {
            console.log(`✅ Script already loaded: ${src}`);
            return Promise.resolve();
        }

        console.log(`🔄 Loading script: ${src}`);

        const promise = new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = src;
            script.async = true;

            if (options.defer) script.defer = true;
            if (options.charset) script.charset = options.charset;

            script.onload = () => {
                console.log(`✅ Script loaded successfully: ${src}`);
                this.loadedScripts.add(src);
                this.loadingPromises.delete(src);
                resolve();
            };

            script.onerror = () => {
                console.error(`❌ Failed to load script: ${src}`);
                this.loadingPromises.delete(src);
                reject(new Error(`Failed to load script: ${src}`));
            };

            document.head.appendChild(script);
        });

        this.loadingPromises.set(src, promise);
        return promise;
    }

    /**
     * Load multiple scripts in parallel
     * @param {Array} scripts - Array of script sources or objects with src and options
     * @returns {Promise} Promise that resolves when all scripts are loaded
     */
    loadScripts(scripts) {
        const promises = scripts.map(script => {
            if (typeof script === 'string') {
                return this.loadScript(script);
            }
            return this.loadScript(script.src, script.options || {});
        });
        return Promise.all(promises);
    }

    /**
     * Load Plotly.js library for data visualization
     * @returns {Promise} Promise that resolves when Plotly is available
     */
    loadPlotly() {
        return this.loadScript('/static/js/dist/plotly.min.js', {
            charset: 'utf-8'
        }).then(() => {
            if (typeof Plotly === 'undefined') {
                throw new Error('Plotly failed to initialize');
            }
            return Plotly;
        });
    }

    /**
     * Load Apache Arrow library for binary data processing
     * @returns {Promise} Promise that resolves when Arrow is available
     */
    loadArrow() {
        return this.loadScript('/static/js/dist/arrow.min.js', {
            charset: 'utf-8'
        }).then(() => {
            if (typeof Arrow === 'undefined') {
                throw new Error('Arrow failed to initialize');
            }
            return Arrow;
        });
    }

    /**
     * Load analytics dependencies (both Plotly and Arrow)
     * @returns {Promise} Promise that resolves when both libraries are available
     */
    loadAnalyticsDependencies() {
        console.log('📊 Loading analytics dependencies...');
        return this.loadScripts([
            { src: '/static/js/dist/plotly.min.js', options: { charset: 'utf-8' } },
            { src: '/static/js/dist/arrow.min.js', options: { charset: 'utf-8' } }
        ]).then(() => {
            console.log('🔍 Checking for global variables after script load...');
            console.log('Plotly available:', typeof window.Plotly !== 'undefined');
            console.log('Arrow available:', typeof window.Arrow !== 'undefined');

            // Wait a brief moment for variables to be set
            return new Promise(resolve => {
                setTimeout(() => {
                    const plotlyAvailable = typeof window.Plotly !== 'undefined';
                    const arrowAvailable = typeof window.Arrow !== 'undefined';

                    console.log('🔍 Rechecking after timeout...');
                    console.log('Plotly available:', plotlyAvailable);
                    console.log('Arrow available:', arrowAvailable);

                    if (!plotlyAvailable || !arrowAvailable) {
                        console.warn('⚠️ Some analytics dependencies may not be available:', {
                            Plotly: plotlyAvailable,
                            Arrow: arrowAvailable
                        });
                        // Don't throw error if at least one library loaded
                        if (!plotlyAvailable && !arrowAvailable) {
                            throw new Error('Analytics dependencies failed to initialize');
                        }
                    }

                    resolve({
                        Plotly: window.Plotly,
                        Arrow: window.Arrow
                    });
                }, 100);
            });
        });
    }

    /**
     * Preload a script without executing it immediately
     * @param {string} src - Script source URL
     */
    preloadScript(src) {
        if (this.loadedScripts.has(src) || this.loadingPromises.has(src)) {
            return;
        }

        const link = document.createElement('link');
        link.rel = 'preload';
        link.as = 'script';
        link.href = src;
        document.head.appendChild(link);
    }

    /**
     * Check if a library is available
     * @param {string} libraryName - Global variable name to check
     * @returns {boolean} True if library is available
     */
    isLibraryAvailable(libraryName) {
        return typeof window[libraryName] !== 'undefined';
    }
}

// Create global instance
window.lazyLoader = new LazyLoader();

// Page-specific loading utilities
window.loadPageDependencies = function(pageName) {
    switch (pageName) {
        case 'analytics':
            return window.lazyLoader.loadAnalyticsDependencies();
        case 'sharecard':
            // Share card only needs minimal dependencies, no heavy libs required
            return Promise.resolve();
        case '1rm':
            // 1RM calculator doesn't need heavy libraries
            return Promise.resolve();
        case 'home':
            // Home page only needs basic functionality
            return Promise.resolve();
        default:
            return Promise.resolve();
    }
};

// Auto-detect page type and load appropriate dependencies
window.autoLoadPageDependencies = function() {
    let pageName = 'unknown';

    // Detect page type based on URL or DOM elements
    const path = window.location.pathname;
    if (path === '/' || path === '') {
        pageName = 'home';
    } else if (path === '/analytics') {
        pageName = 'analytics';
    } else if (path === '/sharecard') {
        pageName = 'sharecard';
    } else if (path === '/1rm') {
        pageName = '1rm';
    }

    // Alternative detection based on specific DOM elements (only if URL detection failed)
    if (pageName === 'unknown') {
        // Check for analytics page specific elements
        if (document.getElementById('weightDistribution') && document.getElementById('dotsScatter')) {
            pageName = 'analytics';
        } else if (document.getElementById('shareName') || document.getElementById('sharecard-container')) {
            pageName = 'sharecard';
        } else if (document.getElementById('oneRepMaxForm') || document.querySelector('.one-rm-calculator')) {
            pageName = '1rm';
        } else {
            // Default to home for unknown pages to prevent unnecessary loading
            pageName = 'home';
        }
    }

    console.log(`🏷️ Detected page type: ${pageName}`);
    return window.loadPageDependencies(pageName);
};

// Intersection Observer for lazy loading on visibility
window.lazyLoadOnVisible = function(element, callback) {
    if ('IntersectionObserver' in window) {
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    callback();
                    observer.unobserve(entry.target);
                }
            });
        }, { threshold: 0.1 });

        observer.observe(element);
    } else {
        // Fallback for older browsers
        callback();
    }
};

// Auto-initialize when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Lazy loader initialized');

    // Auto-load page dependencies based on detected page type
    window.autoLoadPageDependencies().catch(error => {
        console.error('Failed to auto-load page dependencies:', error);
        // Don't prevent page functionality if dependency loading fails
    });
});

// Export for module systems if available
if (typeof module !== 'undefined' && module.exports) {
    module.exports = LazyLoader;
}