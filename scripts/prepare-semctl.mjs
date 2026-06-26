import { cpSync, existsSync, mkdirSync } from "node:fs";
import { execSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const host = execSync("rustc -vV", { encoding: "utf8" })
  .match(/^host: (.+)$/m)?.[1];

if (!host) {
  console.error("prepare-semctl: could not detect Rust host triple");
  process.exit(1);
}

const target = process.env.CARGO_BUILD_TARGET || host;
const profile = process.env.PROFILE === "release" ? "release" : "debug";
const releaseFlag = profile === "release" ? " --release" : "";
const targetFlag = target !== host ? ` --target ${target}` : "";

execSync(`cargo build -p semctl --bin semctl${releaseFlag}${targetFlag}`, {
  cwd: root,
  stdio: "inherit",
});

const semctlName = process.platform === "win32" ? "semctl.exe" : "semctl";
const candidates = [
  join(root, "target", target, profile, semctlName),
  join(root, "target", profile, semctlName),
];

const built = candidates.find((path) => existsSync(path));
if (!built) {
  console.error("prepare-semctl: semctl binary not found after build");
  process.exit(1);
}

const binDir = join(root, "src-tauri", "bin");
mkdirSync(binDir, { recursive: true });

const staged = join(
  binDir,
  process.platform === "win32" ? `semctl-${target}.exe` : `semctl-${target}`,
);
cpSync(built, staged);
console.log(`prepare-semctl: staged ${staged}`);
