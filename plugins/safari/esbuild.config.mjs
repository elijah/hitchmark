import esbuild from "esbuild";

const production = process.argv.includes("--production");

const sharedConfig = {
  bundle: true,
  format: /** @type {const} */ ("iife"),
  minify: production,
  sourcemap: !production,
  platform: /** @type {const} */ ("browser"),
  // chrome.* globals are provided by the browser extension runtime
  external: [],
  define: {},
};

await esbuild.build({
  ...sharedConfig,
  entryPoints: [{ in: "src/background.ts", out: "background" }],
  format: "iife",
  outdir: "Resources",
});

await esbuild.build({
  ...sharedConfig,
  entryPoints: [
    { in: "src/popup.ts", out: "popup" },
    { in: "src/content.ts", out: "content" },
  ],
  outdir: "Resources",
});

console.log("Safari extension built successfully.");
