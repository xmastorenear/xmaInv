import { createApp } from "vue";
import App from "./App.vue";
import i18n from './i18n';
import "/node_modules/flag-icons/css/flag-icons.min.css";

createApp(App).use(i18n).mount("#app");
