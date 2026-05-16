#!/usr/bin/env node
/**
 * Frees a TCP port before starting the Tauri dev server (macOS/Linux).
 */
import { execSync } from "node:child_process";

const port = process.argv[2] ?? "5173";

function freePort() {
  try {
    const out = execSync(`lsof -ti:${port}`, { encoding: "utf8" }).trim();
    if (!out) return;
    for (const pid of out.split(/\s+/).filter(Boolean)) {
      try {
        process.kill(Number(pid), "SIGTERM");
        console.log(`Stopped process ${pid} on port ${port}`);
      } catch {
        /* already gone */
      }
    }
    // Brief wait so the port is released
    execSync("sleep 0.3");
  } catch {
    /* nothing listening */
  }
}

freePort();
