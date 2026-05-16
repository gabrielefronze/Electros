#!/usr/bin/env node
import { execSync } from "child_process";
import { cpSync, rmSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const TAURI_ROOT = resolve(__dirname);
const GUI_ROOT = resolve(__dirname, "../elemento-gui-new");
const DIST_SRC = resolve(GUI_ROOT, "dist-renderer");
const DIST_DEST = resolve(TAURI_ROOT, "dist-renderer");

function log(msg) {
  console.log(`\n\x1b[36m▶\x1b[0m ${msg}`);
}
function ok(msg) {
  console.log(`\x1b[32m✔\x1b[0m ${msg}`);
}
function fail(msg) {
  console.error(`\x1b[31m✖\x1b[0m ${msg}`);
}

function run(cmd, cwd = TAURI_ROOT) {
  execSync(cmd, { cwd, stdio: "inherit" });
}

function cleanup() {
  log("Cleaning up copied dist-renderer…");
  if (existsSync(DIST_DEST)) {
    rmSync(DIST_DEST, { recursive: true, force: true });
    ok("dist-renderer removed from tauri folder.");
  }
}

(async () => {
  process.on("SIGINT", () => {
    cleanup();
    process.exit(1);
  });
  process.on("SIGTERM", () => {
    cleanup();
    process.exit(1);
  });

    const debug = process.argv.includes("--debug");
    const extraArgs = process.argv.filter((a) => a !== "--debug").slice(2).join(" ");
    const tauriArgs = [debug ? "" : "", extraArgs].filter(Boolean).join(" ");

    try {
    const viteBin = resolve(GUI_ROOT, "node_modules/.bin/vite");
    log("Building renderer with Vite…");
    run(`"${viteBin}" build`, GUI_ROOT);
    ok("Vite build complete.");

    log(`Copying dist-renderer → ${DIST_DEST}`);
    if (!existsSync(DIST_SRC)) {
      throw new Error(`Vite output not found at: ${DIST_SRC}`);
    }
    cpSync(DIST_SRC, DIST_DEST, { recursive: true });
    ok("dist-renderer copied.");

    const titlebarSrc = resolve(TAURI_ROOT, "titlebar");
    const titlebarDest = resolve(DIST_DEST, "titlebar");
    if (existsSync(titlebarSrc)) {
      log(`Copying titlebar → ${titlebarDest}`);
      cpSync(titlebarSrc, titlebarDest, { recursive: true });
      ok("titlebar copied.");
    }

        const tauriBin = resolve(TAURI_ROOT, "node_modules/.bin/tauri");
        log(`Running tauri build${debug ? " (debug)" : ""}…`);
        run(`"${tauriBin}" build${tauriArgs ? ` ${tauriArgs}` : ""}`, TAURI_ROOT);
    ok("Tauri build complete.");
  } catch (err) {
    fail(`Build failed: ${err.message}`);
    cleanup();
    process.exit(1);
  }

  cleanup();
  console.log("\n\x1b[32m● All done!\x1b[0m\n");
})();
