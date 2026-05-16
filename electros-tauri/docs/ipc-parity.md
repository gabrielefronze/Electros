# IPC parity (Electron preload ↔ Tauri)

Channel names match [`electros-electron/preload.js`](../../electros-electron/preload.js).

## `invoke` channels

| Channel | Tauri implementation |
|---------|---------------------|
| `read-config` | `~/.elemento/settings` JSON |
| `write-config` | Merge + write settings |
| `read-hosts` | `~/.elemento/hosts` lines |
| `write-hosts` | Write hosts file |
| `cors-safe-fetch` | `reqwest` with POTD host allowlist |
| `list-backgrounds` | Scan `~/.elemento/backgrounds` |
| `get-background-data` | File URL or base64 thumbnail |
| `import-background` | Native file dialog + WebP conversion |
| `save-background-from-url` | Download + WebP |
| `delete-background` | Delete file |
| `convert-existing-backgrounds` | Batch WebP conversion |
| `minimize-window` / `maximize-window` / `close-window` / `toggle-full-screen` | Main webview window |
| `check-port` | TCP connect probe |
| `os-prefers-dark-theme` | `dark-light` crate |
| `os-prefers-reduced-transparency` | Returns `false` |
| `open-browser` | `open` crate |
| `open-dot-config` | Open `~/.elemento` in file manager |
| `app-version` | `CARGO_PKG_VERSION` |
| `safestorage-encrypt` / `safestorage-decrypt` | AES-GCM + OS keyring |
| `get-daemons-log` | Ring buffer from daemon stdout/stderr |
| `open-ssh` | Node `ssh.cjs` + SSH webview |
| `open-rdp` | RDP HTML webview |
| `launch-rdp-process` / `cleanup-rdp-process` | `mstsc-rs` sidecar |
| `create-popup` | Separate command (not in preload whitelist) |

## Events

| Event | Source |
|-------|--------|
| `deep-link` | `tauri-plugin-deep-link` (`electros://`) |
| `window-close` | SSH webview close |
| `rdp-process-closed` | `mstsc-rs` process exit |
| `terminal-output` | Reserved for daemon terminal |

## Custom protocol

| Protocol | Purpose |
|----------|---------|
| `elemento-bg://bg/...` | Wallpaper files under `~/.elemento/backgrounds` |
