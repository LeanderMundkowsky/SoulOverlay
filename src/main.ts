import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./assets/main.css";

// Suppress the native browser context menu — it has no useful actions in an overlay.
document.addEventListener("contextmenu", (e) => e.preventDefault());

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.mount("#app");
