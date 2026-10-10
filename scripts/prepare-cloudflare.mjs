import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const source = join(root, "website/build");
const output = join(root, "website/.cloudflare-dist");
const manifestPath = join(root, "website/.cloudflare-upload.json");
const releases = JSON.parse(
  readFileSync(join(source, "releases.json"), "utf8"),
);
if (releases.schemaVersion !== 1 || !Object.keys(releases.games).length)
  throw new Error("No playable release catalog");
rmSync(output, { recursive: true, force: true });
cpSync(source, output, {
  recursive: true,
  filter: (path) => relative(source, path).split(/[\\/]/)[0] !== "builds",
});
const uploads = [];
for (const [game, release] of Object.entries(releases.games)) {
  if (
    !["zombies", "throne"].includes(game) ||
    !/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(release.buildId)
  )
    throw new Error("Invalid release");
  const prefix = `builds/${release.buildId}/${game}`;
  const directory = join(source, prefix);
  const manifest = JSON.parse(
    readFileSync(join(directory, "build.json"), "utf8"),
  );
  if (
    manifest.gameId !== game ||
    manifest.buildId !== release.buildId ||
    manifest.module !== `/${prefix}/wasm.js` ||
    manifest.wasm !== `/${prefix}/wasm_bg.wasm` ||
    release.manifest !== `/${prefix}/build.json`
  )
    throw new Error("Build contract mismatch");
  if (!existsSync(join(directory, "wasm.js")))
    throw new Error("Missing module");
  mkdirSync(join(output, prefix), { recursive: true });
  function copy(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const file = join(dir, entry.name),
        key = relative(source, file).split("\\").join("/");
      if (entry.isDirectory()) {
        mkdirSync(join(output, key), { recursive: true });
        copy(file);
        continue;
      }
      if (file === join(directory, "wasm_bg.wasm")) {
        const body = readFileSync(file);
        if (!body.subarray(0, 4).equals(Buffer.from([0, 97, 115, 109])))
          throw new Error("Invalid WASM");
        uploads.push({
          key,
          file,
          size: body.length,
          sha256: createHash("sha256").update(body).digest("hex"),
        });
      } else {
        if (statSync(file).size > 25 * 1024 * 1024)
          throw new Error(`Pages file limit: ${key}`);
        cpSync(file, join(output, key));
      }
    }
  }
  copy(directory);
}
writeFileSync(
  join(output, "_routes.json"),
  JSON.stringify(
    { version: 1, include: ["/builds/*/wasm_bg.wasm"], exclude: [] },
    null,
    2,
  ),
);
writeFileSync(
  join(output, "_headers"),
  "/releases.json\n  Cache-Control: no-store\n/builds/*\n  Cache-Control: public, max-age=31536000, immutable\n/loader.html\n  Cache-Control: no-store\n/loader.js\n  Cache-Control: no-cache\n",
);
writeFileSync(
  manifestPath,
  JSON.stringify({ bucket: "alacod-game-builds", uploads }, null, 2),
);
console.log(
  `Prepared ${output}; ${uploads.length} immutable WASM objects for R2.`,
);
