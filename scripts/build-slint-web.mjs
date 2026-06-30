import { cpSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const root = resolve(import.meta.dirname, "..");
const web = join(root, "web");
const dist = join(web, "dist");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));

rmSync(dist, { recursive: true, force: true });
mkdirSync(dist, { recursive: true });

const result = spawnSync(
  "wasm-pack",
  [
    "build",
    join(root, "crates/memory_pak_app"),
    "--target",
    "web",
    "--release",
    "--out-dir",
    join(dist, "pkg"),
    "--out-name",
    "memory_pak"
  ],
  { stdio: "inherit" }
);
if (result.status !== 0) process.exit(result.status ?? 1);

for (const file of [
  ".gitignore",
  "package.json",
  "memory_pak.d.ts",
  "memory_pak_bg.wasm.d.ts"
]) {
  rmSync(join(dist, "pkg", file), { force: true });
}

for (const file of ["index.html", "manifest.webmanifest"]) {
  cpSync(join(web, "static", file), join(dist, file));
}
cpSync(join(root, "icons", "web"), join(dist, "icons"), { recursive: true });

function filesUnder(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? filesUnder(path) : [path];
  });
}

const precache = filesUnder(dist)
  .map((path) => `./${relative(dist, path).replaceAll("\\", "/")}`)
  .filter((path) => !path.endsWith("/sw.js"));
const template = readFileSync(join(web, "static", "sw.template.js"), "utf8");
const buildDigest = createHash("sha256")
  .update(readFileSync(join(dist, "pkg", "memory_pak_bg.wasm")))
  .digest("hex")
  .slice(0, 16);
const worker = template
  .replace("__CACHE_NAME__", `memory-pak-${pkg.version}-${buildDigest}`)
  .replace("__PRECACHE__", JSON.stringify(precache, null, 2));
writeFileSync(join(dist, "sw.js"), worker);
