// Sets the app version everywhere it lives: node scripts/bump-version.mjs 0.2.0
import { readFileSync, writeFileSync } from "node:fs";

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) {
  console.error("usage: node scripts/bump-version.mjs <major.minor.patch>");
  process.exit(1);
}

function update(path, from, to) {
  const text = readFileSync(path, "utf8");
  if (!from.test(text)) throw new Error(`no version found in ${path}`);
  writeFileSync(path, text.replace(from, to));
}

update("package.json", /"version": "[^"]+"/, `"version": "${version}"`);
update("src-tauri/tauri.conf.json", /"version": "[^"]+"/, `"version": "${version}"`);
// The first `version =` in Cargo.toml is the package's own.
update("src-tauri/Cargo.toml", /^version = "[^"]+"/m, `version = "${version}"`);
console.log(`version set to ${version}; run \`cargo check\` in src-tauri to update Cargo.lock`);
