/**
 * Installs the Electros custom titlebar (parity with electros-electron Loaders.js).
 * Loaded via Tauri invoke initialization script so it survives navigation to devUrl.
 */
(function () {
  function guiBase() {
    if (window.__ELECTROS_GUI_BASE__) {
      return window.__ELECTROS_GUI_BASE__.replace(/\/$/, "");
    }
    return new URL(".", window.location.href).href.replace(/\/$/, "");
  }

  function titlebarBase() {
    if (window.__ELECTROS_TITLEBAR_BASE__) {
      return window.__ELECTROS_TITLEBAR_BASE__.replace(/\/$/, "");
    }
    return new URL("../titlebar/", window.location.href).href.replace(/\/$/, "");
  }

  function ensureStylesheet(href) {
    if (document.querySelector('link[rel="stylesheet"][href="' + href + '"]')) {
      return;
    }
    var link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = href;
    document.head.appendChild(link);
  }

  function installTitlebar() {
    if (document.querySelector(".electros-titlebar")) {
      return;
    }
    if (!document.body) {
      return;
    }
    if (!window.electron) {
      return;
    }

    var gui = guiBase();
    var tb = titlebarBase();
    ensureStylesheet(gui + "/css/themes.css");
    ensureStylesheet(gui + "/css/form-controls.css");
    ensureStylesheet(tb + "/titlebar.css");

    var titlebar = document.createElement("div");
    titlebar.className = "electros-titlebar";
    // "deep" so clicks on the title label (child) still start a window drag
    titlebar.setAttribute("data-tauri-drag-region", "deep");

    var titleElement = document.createElement("div");
    titleElement.className = "electros-titlebar-title";
    titleElement.textContent = document.title;

    document.body.insertBefore(titlebar, document.body.firstChild);

    var script = document.createElement("script");
    script.src = tb + "/titlebar.js";
    script.onload = function () {
      if (typeof initializeTitlebar === "function") {
        initializeTitlebar({ minimizeOnly: false });
        var buttons = titlebar.querySelector(".electros-titlebar-buttons");
        if (buttons) {
          buttons.setAttribute("data-tauri-drag-region", "false");
        }
      }
      titlebar.appendChild(titleElement);
    };
    script.onerror = function () {
      console.error("[electros-tauri] failed to load titlebar.js from", script.src);
    };
    document.head.appendChild(script);
  }

  function scheduleInstall() {
    if (document.querySelector(".electros-titlebar")) {
      return;
    }
    installTitlebar();
    if (!document.querySelector(".electros-titlebar")) {
      window.setTimeout(scheduleInstall, 50);
    }
  }

  function boot() {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", scheduleInstall);
    } else {
      scheduleInstall();
    }
    window.addEventListener("load", scheduleInstall);
  }

  boot();
})();
