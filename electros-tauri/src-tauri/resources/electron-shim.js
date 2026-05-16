/**
 * Electron preload-compatible API for Tauri.
 * Injected before the Electros GUI loads so existing code can use window.electron.
 */
(function () {
  if (window.electron) {
    return;
  }

  const VALID_INVOKE = new Set([
    "read-config",
    "write-config",
    "read-hosts",
    "write-hosts",
    "cors-safe-fetch",
    "list-backgrounds",
    "get-background-data",
    "import-background",
    "save-background-from-url",
    "delete-background",
    "convert-existing-backgrounds",
    "minimize-window",
    "open-ssh",
    "maximize-window",
    "close-window",
    "toggle-full-screen",
    "open-rdp",
    "launch-rdp-process",
    "cleanup-rdp-process",
    "check-port",
    "os-prefers-dark-theme",
    "os-prefers-reduced-transparency",
    "open-browser",
    "open-dot-config",
    "app-version",
    "safestorage-encrypt",
    "safestorage-decrypt",
    "get-daemons-log",
  ]);

  const VALID_SEND = new Set(["open-rdp"]);
  const VALID_ON = new Set(["window-close", "rdp-process-closed", "deep-link", "terminal-output"]);

  const listeners = new Map();

  function getTauriInvoke() {
    if (window.__TAURI_INTERNALS__?.invoke) {
      return window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
    }
    if (window.__TAURI__?.core?.invoke) {
      return window.__TAURI__.core.invoke.bind(window.__TAURI__.core);
    }
    throw new Error("Tauri invoke API not available");
  }

  function getTauriListen() {
    if (window.__TAURI_INTERNALS__?.listen) {
      return window.__TAURI_INTERNALS__.listen.bind(window.__TAURI_INTERNALS__);
    }
    if (window.__TAURI__?.event?.listen) {
      return window.__TAURI__.event.listen.bind(window.__TAURI__.event);
    }
    return null;
  }

  async function tauriInvoke(cmd, payload) {
    const invoke = getTauriInvoke();
    return invoke(cmd, payload);
  }

  async function electronInvoke(channel, ...args) {
    if (!VALID_INVOKE.has(channel)) {
      throw new Error(`Invalid IPC channel: ${channel}`);
    }
    return tauriInvoke("electron_invoke", { channel, args });
  }

  function ensureListenerSet(channel) {
    if (listeners.has(channel)) {
      return;
    }
    const listen = getTauriListen();
    if (!listen) {
      return;
    }
    const unsubs = [];
    listen(channel, (event) => {
      const payload = event.payload;
      const cbs = listeners.get(channel) || [];
      cbs.forEach((cb) => {
        try {
          if (channel === "deep-link") {
            cb(payload);
          } else if (channel === "window-close" || channel === "rdp-process-closed") {
            cb({}, payload);
          } else {
            cb({}, payload);
          }
        } catch (e) {
          console.error("electron shim listener error", e);
        }
      });
    }).then((unsub) => unsubs.push(unsub));
    listeners.set(channel, []);
  }

  window.electron = {
    invoke: electronInvoke,
    createPopup: (options) => tauriInvoke("create_popup", { options }),
    ipcRenderer: {
      send: (channel, ...args) => {
        if (!VALID_SEND.has(channel)) {
          return;
        }
        tauriInvoke("electron_send", { channel, args }).catch(console.error);
      },
      on: (channel, callback) => {
        if (!VALID_ON.has(channel)) {
          return;
        }
        ensureListenerSet(channel);
        const list = listeners.get(channel) || [];
        list.push(callback);
        listeners.set(channel, list);
      },
    },
    onDeepLink: (callback) => {
      ensureListenerSet("deep-link");
      const list = listeners.get("deep-link") || [];
      list.push(callback);
      listeners.set("deep-link", list);
    },
  };
})();
