import "../../lang/index.js";
// Tauri 兼容桥接层：window.electronAPI → @tauri-apps/api invoke()
import "./electronApiCompat";
import { createApp } from "vue";
import { directives } from "@/directives";
import App from "./App.vue";
import "./style.less";
import "@/themes/theme-main.less";

const app = createApp(App);

Object.entries(directives).forEach(([name, directive]) => {
  app.directive(name, directive);
});

app.mount("#app");
