/**
 * Lazy Loading Module for Iron Insights
 * Dynamically loads JavaScript libraries based on page requirements
 */

interface ScriptOptions {
  defer?: boolean;
  charset?: string;
}

interface ScriptConfig {
  src: string;
  options?: ScriptOptions;
}

interface AnalyticsDependencies {
  Plotly: any;
  Arrow: any;
}

// Augment the global Window interface
// Note: Window.Plotly and Window.Arrow types are defined in their respective entry files
declare global {
  interface Window {
    lazyLoader: LazyLoader;
    loadPageDependencies: (pageName: string) => Promise<void | AnalyticsDependencies>;
    autoLoadPageDependencies: () => Promise<void | AnalyticsDependencies>;
    lazyLoadOnVisible: (element: Element, callback: () => void) => void;
  }
}

class LazyLoader {
  private loadedScripts: Set<string>;
  private loadingPromises: Map<string, Promise<void>>;

  constructor() {
    this.loadedScripts = new Set();
    this.loadingPromises = new Map();
  }

  /**
   * Load a script dynamically and return a promise
   * @param src - Script source URL
   * @param options - Loading options
   * @returns Promise that resolves when script is loaded
   */
  loadScript(src: string, options: ScriptOptions = {}): Promise<void> {
    // Return existing promise if already loading
    if (this.loadingPromises.has(src)) {
      console.log(`📋 Script already loading: ${src}`);
      return this.loadingPromises.get(src)!;
    }

    // Return resolved promise if already loaded
    if (this.loadedScripts.has(src)) {
      console.log(`✅ Script already loaded: ${src}`);
      return Promise.resolve();
    }

    console.log(`🔄 Loading script: ${src}`);

    const promise = new Promise<void>((resolve, reject) => {
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
   * @param scripts - Array of script sources or objects with src and options
   * @returns Promise that resolves when all scripts are loaded
   */
  loadScripts(scripts: (string | ScriptConfig)[]): Promise<void[]> {
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
   * @returns Promise that resolves when Plotly is available
   */
  loadPlotly(): Promise<any> {
    return this.loadScript('/static/js/dist/plotly.min.js', {
      charset: 'utf-8'
    }).then(() => {
      if (typeof window.Plotly === 'undefined') {
        throw new Error('Plotly failed to initialize');
      }
      return window.Plotly;
    });
  }

  /**
   * Load Apache Arrow library for binary data processing
   * @returns Promise that resolves when Arrow is available
   */
  loadArrow(): Promise<any> {
    return this.loadScript('/static/js/dist/arrow.min.js', {
      charset: 'utf-8'
    }).then(() => {
      if (typeof window.Arrow === 'undefined') {
        throw new Error('Arrow failed to initialize');
      }
      return window.Arrow;
    });
  }

  /**
   * Load analytics dependencies (both Plotly and Arrow)
   * @returns Promise that resolves when both libraries are available
   */
  loadAnalyticsDependencies(): Promise<AnalyticsDependencies> {
    console.log('📊 Loading analytics dependencies...');
    return this.loadScripts([
      { src: '/static/js/dist/plotly.min.js', options: { charset: 'utf-8' } },
      { src: '/static/js/dist/arrow.min.js', options: { charset: 'utf-8' } }
    ]).then(() => {
      console.log('🔍 Checking for global variables after script load...');
      console.log('Plotly available:', typeof window.Plotly !== 'undefined');
      console.log('Arrow available:', typeof window.Arrow !== 'undefined');

      // Wait a brief moment for variables to be set
      return new Promise<AnalyticsDependencies>((resolve, reject) => {
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
              reject(new Error('Analytics dependencies failed to initialize'));
              return;
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
   * @param src - Script source URL
   */
  preloadScript(src: string): void {
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
   * @param libraryName - Global variable name to check
   * @returns True if library is available
   */
  isLibraryAvailable(libraryName: string): boolean {
    return typeof (window as any)[libraryName] !== 'undefined';
  }
}

// Create global instance
window.lazyLoader = new LazyLoader();

// Page-specific loading utilities
window.loadPageDependencies = function(pageName: string): Promise<void | AnalyticsDependencies> {
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
window.autoLoadPageDependencies = function(): Promise<void | AnalyticsDependencies> {
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
window.lazyLoadOnVisible = function(element: Element, callback: () => void): void {
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
  window.autoLoadPageDependencies().catch((error: unknown) => {
    console.error('Failed to auto-load page dependencies:', error);
    // Don't prevent page functionality if dependency loading fails
  });
});

// Export the class to make this a module (required for global augmentation)
export default LazyLoader;
