import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./styles/index.css";

/** Bloquea el menú del WebView (Actualizar / Guardar como). Inputs siguen editables. */
document.addEventListener(
  "contextmenu",
  (e) => {
    const t = e.target as HTMLElement | null;
    if (!t) {
      e.preventDefault();
      return;
    }
    if (t.closest("input, textarea, select, [contenteditable='true']")) return;
    e.preventDefault();
  },
  true,
);

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
