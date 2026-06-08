import * as esbuild from "esbuild";

const production = process.argv.includes("--production");
const watch = process.argv.includes("--watch");

const entryPoints = [
  { in: "src/taskpane.ts", out: "taskpane" },
  { in: "src/commands.ts", out: "commands" },
];

const ctx = await esbuild.context({
  entryPoints,
  bundle: true,
  format: "iife",
  minify: production,
  sourcemap: !production,
  platform: "browser",
  outdir: "dist",
  // Office JS is loaded via CDN script tag — don't bundle
  external: ["office-js"],
  logLevel: "info",
});

if (watch) {
  await ctx.watch();
  console.log("Watching…");
} else {
  await ctx.rebuild();
  await ctx.dispose();
}
