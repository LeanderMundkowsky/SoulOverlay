import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./assets/main.css";

// Connect to standalone Vue DevTools (Electron app) in dev mode.
// Start it with: npx vue-devtools
if (import.meta.env.DEV) {
  import("@vue/devtools").then((m) => {
    m.devtools.connect("http://localhost", 8098);
  });
}

// Suppress the native browser context menu — it has no useful actions in an overlay.
document.addEventListener("contextmenu", (e) => e.preventDefault());

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.mount("#app");
