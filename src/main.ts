import {createApp} from "vue";
import App from "./App.vue";
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'
import '@/styles/common.css'

import {create,} from "naive-ui";
import store from "@/store";

const naive =create()

const app = createApp(App)

app.use(store)
app.use(naive)

app.mount("#app");

// console.log('Frontend loaded in', performance.now() - window.__START_TIME__, 'ms');