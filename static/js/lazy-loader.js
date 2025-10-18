"use strict";
var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);
class LazyLoader {
  constructor() {
    __publicField(this, "loadedScripts");
    __publicField(this, "loadingPromises");
    this.loadedScripts = /* @__PURE__ */ new Set();
    this.loadingPromises = /* @__PURE__ */ new Map();
  }
  /**
   * Load a script dynamically and return a promise
   * @param src - Script source URL
   * @param options - Loading options
   * @returns Promise that resolves when script is loaded
   */
  loadScript(src, options = {}) {
    if (this.loadingPromises.has(src)) {
      console.log(`\u{1F4CB} Script already loading: ${src}`);
      return this.loadingPromises.get(src);
    }
    if (this.loadedScripts.has(src)) {
      console.log(`\u2705 Script already loaded: ${src}`);
      return Promise.resolve();
    }
    const existingScript = document.querySelector(`script[src="${src}"]`);
    if (existingScript) {
      console.log(`\u2705 Script already in DOM: ${src}`);
      this.loadedScripts.add(src);
      return Promise.resolve();
    }
    console.log(`\u{1F504} Loading script: ${src}`);
    const promise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = src;
      script.async = true;
      if (options.defer) script.defer = true;
      if (options.charset) script.charset = options.charset;
      script.onload = () => {
        console.log(`\u2705 Script loaded successfully: ${src}`);
        this.loadedScripts.add(src);
        this.loadingPromises.delete(src);
        resolve();
      };
      script.onerror = () => {
        console.error(`\u274C Failed to load script: ${src}`);
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
  loadScripts(scripts) {
    const promises = scripts.map((script) => {
      if (typeof script === "string") {
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
  loadPlotly() {
    return this.loadScript("/static/js/dist/plotly.min.js", {
      charset: "utf-8"
    }).then(() => {
      if (typeof window.Plotly === "undefined") {
        throw new Error("Plotly failed to initialize");
      }
      return window.Plotly;
    });
  }
  /**
   * Load Apache Arrow library for binary data processing
   * @returns Promise that resolves when Arrow is available
   */
  loadArrow() {
    return this.loadScript("/static/js/dist/arrow.min.js", {
      charset: "utf-8"
    }).then(() => {
      if (typeof window.Arrow === "undefined") {
        throw new Error("Arrow failed to initialize");
      }
      return window.Arrow;
    });
  }
  /**
   * Load analytics dependencies (both Plotly and Arrow)
   * @returns Promise that resolves when both libraries are available
   */
  loadAnalyticsDependencies() {
    console.log("\u{1F4CA} Loading analytics dependencies...");
    return this.loadScripts([
      { src: "/static/js/dist/plotly.min.js", options: { charset: "utf-8" } },
      { src: "/static/js/dist/arrow.min.js", options: { charset: "utf-8" } }
    ]).then(() => {
      console.log("\u{1F50D} Checking for global variables after script load...");
      console.log("Plotly available:", typeof window.Plotly !== "undefined");
      console.log("Arrow available:", typeof window.Arrow !== "undefined");
      return new Promise((resolve, reject) => {
        setTimeout(() => {
          const plotlyAvailable = typeof window.Plotly !== "undefined";
          const arrowAvailable = typeof window.Arrow !== "undefined";
          console.log("\u{1F50D} Rechecking after timeout...");
          console.log("Plotly available:", plotlyAvailable);
          console.log("Arrow available:", arrowAvailable);
          if (!plotlyAvailable || !arrowAvailable) {
            console.warn("\u26A0\uFE0F Some analytics dependencies may not be available:", {
              Plotly: plotlyAvailable,
              Arrow: arrowAvailable
            });
            if (!plotlyAvailable && !arrowAvailable) {
              reject(new Error("Analytics dependencies failed to initialize"));
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
  preloadScript(src) {
    if (this.loadedScripts.has(src) || this.loadingPromises.has(src)) {
      return;
    }
    const link = document.createElement("link");
    link.rel = "preload";
    link.as = "script";
    link.href = src;
    document.head.appendChild(link);
  }
  /**
   * Check if a library is available
   * @param libraryName - Global variable name to check
   * @returns True if library is available
   */
  isLibraryAvailable(libraryName) {
    return typeof window[libraryName] !== "undefined";
  }
}
window.lazyLoader = new LazyLoader();
window.loadPageDependencies = function(pageName) {
  switch (pageName) {
    case "analytics":
      return window.lazyLoader.loadAnalyticsDependencies();
    case "sharecard":
      return Promise.resolve();
    case "1rm":
      return Promise.resolve();
    case "home":
      return Promise.resolve();
    default:
      return Promise.resolve();
  }
};
window.autoLoadPageDependencies = function() {
  let pageName = "unknown";
  const path = window.location.pathname;
  if (path === "/" || path === "") {
    pageName = "home";
  } else if (path === "/analytics") {
    pageName = "analytics";
  } else if (path === "/sharecard") {
    pageName = "sharecard";
  } else if (path === "/1rm") {
    pageName = "1rm";
  }
  if (pageName === "unknown") {
    if (document.getElementById("weightDistribution") && document.getElementById("dotsScatter")) {
      pageName = "analytics";
    } else if (document.getElementById("shareName") || document.getElementById("sharecard-container")) {
      pageName = "sharecard";
    } else if (document.getElementById("oneRepMaxForm") || document.querySelector(".one-rm-calculator")) {
      pageName = "1rm";
    } else {
      pageName = "home";
    }
  }
  console.log(`\u{1F3F7}\uFE0F Detected page type: ${pageName}`);
  return window.loadPageDependencies(pageName);
};
window.lazyLoadOnVisible = function(element, callback) {
  if ("IntersectionObserver" in window) {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          callback();
          observer.unobserve(entry.target);
        }
      });
    }, { threshold: 0.1 });
    observer.observe(element);
  } else {
    callback();
  }
};
document.addEventListener("DOMContentLoaded", function() {
  console.log("\u{1F680} Lazy loader initialized");
});
