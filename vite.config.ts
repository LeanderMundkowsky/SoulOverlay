import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueDevTools from "vite-plugin-vue-devtools";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vueDevTools({
      componentInspector: {
        toggleComboKey: false,     // disable Ctrl+Shift toggle — keep keys free for overlay
        toggleButtonVisibility: "never", // hide the inspector button
      },
    }),
    vue(),
  ],
  define: {
    __APP_VERSION__: JSON.stringify(process.env.npm_package_version || "0.0.0"),
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
