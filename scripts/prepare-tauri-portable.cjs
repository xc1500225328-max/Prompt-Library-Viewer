const fs = require("node:fs");
const path = require("node:path");

const rootDir = path.resolve(__dirname, "..");
const cacheDir = path.resolve(
  process.env.PROMPT_LIBRARY_CACHE_DIR || path.join(rootDir, ".cache", "github-markdown")
);

async function main() {
  const releaseExe = path.join(rootDir, "src-tauri", "target", "release", "app.exe");
  if (!fs.existsSync(releaseExe)) {
    throw new Error(`Missing Tauri release exe: ${releaseExe}`);
  }
  const responseCache = path.join(cacheDir, "api-response.json");
  if (!fs.existsSync(responseCache)) {
    throw new Error(`Missing response cache: ${responseCache}`);
  }

  const portableDir = path.resolve(
    process.env.PROMPT_LIBRARY_PORTABLE_DIR || path.join(rootDir, "dist", "tauri-portable")
  );
  fs.rmSync(portableDir, { recursive: true, force: true });
  fs.mkdirSync(path.join(portableDir, ".cache"), { recursive: true });

  fs.copyFileSync(releaseExe, path.join(portableDir, "Prompt Library Viewer.exe"));
  fs.cpSync(cacheDir, path.join(portableDir, ".cache", "github-markdown"), { recursive: true });

  const data = JSON.parse(fs.readFileSync(responseCache, "utf8"));

  console.log(
    JSON.stringify(
      {
        portableDir,
        cacheDir,
        items: data.items.length,
        categories: data.categories.length,
        sources: data.sources.length,
        errors: data.errors.length,
      },
      null,
      2
    )
  );
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
