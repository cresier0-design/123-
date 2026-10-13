import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (path) => readFileSync(join(root, path), 'utf8');
const config = JSON.parse(read('src-tauri/tauri.conf.json'));
const pkg = JSON.parse(read('package.json'));
const lock = JSON.parse(read('package-lock.json'));
assert.equal(config.version, pkg.version, 'Tauri and package versions differ');
assert.equal(lock.version, pkg.version, 'npm lock version differs');
assert.ok(read('src-tauri/Cargo.toml').includes(`version = "${pkg.version}"`));
assert.ok(read('src-tauri/Cargo.lock').includes('name = "crocodile-gena-studio"'));

const required = [
  'src-tauri/build.rs', 'src-tauri/src/lib.rs', 'src-tauri/src/main.rs',
  'src-tauri/src/linux_startup.rs', 'tools/smoke-linux.py',
  'src-tauri/capabilities/default.json', 'ui/index.html',
  '.github/workflows/linux.yml', 'LINUX-START.txt',
  ...config.bundle.icon.map((path) => `src-tauri/${path}`),
  ...['anticheat', 'hypervisor', 'millennium', 'vbs'].map((name) => `src-tauri/src/linux/${name}.rs`),
  ...['SteamFix64.dll', 'winmm_unity.dll', 'EpicFix64.dll', 'EpicFix.ini', 'EOSSDK-Win64-Shipping.dll'].map((name) => `backend/core_engine/${name}`),
  'CreamAPI/steam_api.dll', 'CreamAPI/steam_api64.dll',
  'SmokeAPI/SmokeAPI64.dll', 'SmokeAPI/SmokeAPI86.dll',
  ...['OpenSteamTool.dll', 'dwmapi.dll', 'xinput1_4.dll', 'opensteamtool.toml'].map((name) => `src-tauri/src/assets/${name}`),
];
for (const path of required) {
  assert.ok(existsSync(join(root, path)), `Missing required file: ${path}`);
  assert.ok(statSync(join(root, path)).size > 0, `Empty required file: ${path}`);
}
for (const path of Object.keys(config.bundle.resources)) {
  assert.ok(existsSync(resolve(root, 'src-tauri', path)), `Missing bundled resource: ${path}`);
}
for (const match of read('ui/index.html').matchAll(/(?:src|href)="\.\/([^"?#]+)"/g)) {
  assert.ok(existsSync(join(root, 'ui', match[1])), `Missing frontend file: ${match[1]}`);
}

const ryuu = read('src-tauri/src/ryuu.rs');
assert.ok(ryuu.includes('option_env!("RYUU_AUTH_KEY")'), 'Ryuu key must come from build environment');
assert.ok(!/const AUTH_KEY: &str = "[^"\s]+"/.test(ryuu), 'Hardcoded Ryuu key detected');
const workflow = read('.github/workflows/linux.yml');
assert.ok(workflow.includes('secrets.RYUU_AUTH_KEY'));
assert.ok(workflow.includes('workflow_dispatch:'));

let files = 0;
let bytes = 0;
function inspect(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (['node_modules', 'target', 'gen', 'dist-linux', '.git'].includes(entry.name)) continue;
    assert.ok(!/PRIVATE|\.env(?:\.|$)/i.test(entry.name), 'Private local file in export');
    const path = join(dir, entry.name);
    if (entry.isDirectory()) inspect(path);
    else {
      const size = statSync(path).size;
      assert.ok(size < 25 * 1024 * 1024, `File exceeds browser upload limit: ${entry.name}`);
      files += 1;
      bytes += size;
      if (/\.(?:rs|js|lua|tsx|json|toml|yml|yaml|md|txt)$/.test(entry.name)) {
        const text = readFileSync(path, 'utf8');
        assert.ok(!/(?:github_pat_|ghp_)[A-Za-z0-9_]{20,}/.test(text), 'GitHub token in export');
      }
    }
  }
}
inspect(root);
console.log(`Linux export verified: ${files} files, ${(bytes / 1024 / 1024).toFixed(1)} MiB, version ${pkg.version}.`);
