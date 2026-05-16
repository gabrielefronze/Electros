# Electros (Tauri runtime)

Parallel desktop shell for Electros. **Does not replace** [`electros-electron/`](../electros-electron/); the GUI is unchanged and still defaults to the Electron platform in `elemento-gui-new`.

Tauri injects a **`window.electron` compatibility shim** ([`src-tauri/resources/electron-shim.js`](src-tauri/resources/electron-shim.js)) so the existing renderer and `Electron.ts` interoperable layer work without edits.

## Requirements

- **Rust ≥ 1.85** (Homebrew: `brew upgrade rust`; or [rustup](https://rustup.rs/): `rustup update stable`)
- Node.js 18+
- Platform deps: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- **Node.js** on `PATH` (SSH remote via `ssh.cjs`)
- Optional: `GITHUB_TOKEN` to download daemon binaries

## Setup (opt-in)

```bash
cd electros-tauri
chmod +x setup.sh populate-daemons.sh
./setup.sh
```

## Development

**Single command** (starts Vite, then opens the Tauri window when `http://localhost:5173` is ready):

```bash
cd electros-tauri
npm run dev
```

`npm run dev` stops any process already using port **5173** (e.g. a previous Vite or `npm run dev:vite`). To free the port manually:

```bash
lsof -ti:5173 | xargs kill -9
```

Disable daemons: `npm run dev -- --no-daemons`

Optional — Vite only (uses [`vite.dev.mjs`](vite.dev.mjs), ignores Rust `target/` output):

```bash
cd electros-tauri && npm run dev:vite
```

Tauri uses [`vite.dev.mjs`](vite.dev.mjs) so Vite does not watch the Rust `target/` folder (which previously caused spurious reload errors).

## Production build

```bash
cd electros-tauri
./populate-daemons.sh   # optional, needs GITHUB_TOKEN
npm run build
```

Artifacts are produced under `src-tauri/target/release/bundle/`.

## Layout

| Path | Role |
|------|------|
| `electros/` | Symlink → `elemento-gui-new/electros/` |
| `electros-daemons/` | Native client daemons (populated locally) |
| `terminal/` | Daemon log UI (copied from electron package at setup) |
| `src-tauri/` | Rust host + shim |
| `dist-renderer/` | Vite output (copied during `build.mjs`) |

## Staging remotes (SSH / RDP)

Place the same artifacts used by Electron under:

- `electros/remotes/ssh/ssh.cjs` (via GUI symlink + Node server)
- `electros/remotes/rdp/mstsc-rs` (platform binary)

## Electron dev (unchanged)

```bash
cd elemento-gui-new && npx vite
cd electros-electron && npm start
```
