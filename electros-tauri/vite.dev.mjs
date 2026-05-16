/**
 * Vite dev server for electros-tauri only (Electron keeps using elemento-gui-new/vite.config.mjs).
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

const here = fileURLToPath(new URL(".", import.meta.url));
const guiRoot = path.resolve(here, "../elemento-gui-new");
const titlebarDir = path.resolve(here, "titlebar");

function serveTitlebarPlugin() {
  return {
    name: "electros-serve-titlebar",
    configureServer(server) {
      server.middlewares.use("/titlebar", (req, res, next) => {
        const rel = (req.url || "/").split("?")[0].replace(/^\//, "");
        const filePath = path.join(titlebarDir, rel);
        if (!filePath.startsWith(titlebarDir) || !fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
          next();
          return;
        }
        const ext = path.extname(filePath);
        const types = {
          ".css": "text/css",
          ".js": "text/javascript",
          ".svg": "image/svg+xml",
          ".png": "image/png",
        };
        res.setHeader("Content-Type", types[ext] || "application/octet-stream");
        fs.createReadStream(filePath).pipe(res);
      });
    },
  };
}

export default defineConfig({
  root: guiRoot,
  base: "./",
  resolve: {
    preserveSymlinks: true,
    alias: {
      "@interoperable": path.resolve(guiRoot, "electros/js/interoperable"),
      "@electros/js": path.resolve(guiRoot, "electros/js/electros/js"),
      "@gui": path.resolve(guiRoot, "electros/js/gui/components"),
      "@dataTypes": path.resolve(guiRoot, "electros/js/dataTypes"),
      "@common": path.resolve(guiRoot, "electros/js/common"),
      "@Daemons": path.resolve(guiRoot, "electros/js/daemonsBridge"),
      "@AuthenticationDaemon": path.resolve(guiRoot, "electros/js/authentication"),
      "@ComputeDaemon": path.resolve(guiRoot, "electros/js/compute"),
      "@NetworkingDaemon": path.resolve(guiRoot, "electros/js/networks"),
      "@StorageDaemon": path.resolve(guiRoot, "electros/js/storage"),
      "@ServicesDaemon": path.resolve(guiRoot, "electros/js/services"),
      "@TargetDaemon": path.resolve(guiRoot, "electros/js/targets"),
      "@": path.resolve(guiRoot, "electros/js"),
      "@vnc": path.resolve(guiRoot, "electros/remotes/vnc"),
    },
  },
  optimizeDeps: {
    entries: [path.resolve(guiRoot, "electros/electros.html")],
  },
  plugins: [
    serveTitlebarPlugin(),
    viteStaticCopy({
      targets: [
        { src: "electros/**/*.json", dest: "electros" },
        { src: "electros/**/*.svg", dest: "electros" },
        { src: "electros/js/pages/**/*.css", dest: "electros" },
        { src: "electros/remotes/titlebar/**/*", dest: "electros" },
      ],
    }),
  ],
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      usePolling: true,
      followSymlinks: true,
      ignored: [
        "**/src-tauri/**",
        "**/target/**",
        "**/electros-tauri/src-tauri/**",
      ],
    },
    fs: {
      allow: [".."],
    },
  },
});
