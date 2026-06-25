import { defineConfig } from "vite";
import { svelte, vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { optimizeCss, optimizeImports } from "carbon-preprocess-svelte";

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte({
      preprocess: [vitePreprocess(), optimizeImports()],
    }),optimizeCss(),],
  server: {
    proxy: {
      // match all requests starting with /api and proxy them to backend server
      "/api": {
        target: "http://127.0.0.1:9000", // change this to your backend server address
        changeOrigin: true,
      },
    },
  },
});
