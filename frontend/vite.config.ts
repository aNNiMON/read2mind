import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Icons from "unplugin-icons/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    // Icons are imported as `~icons/lucide/<name>` and compiled to Vue components
    Icons({ compiler: "vue3" }),
  ],
});
