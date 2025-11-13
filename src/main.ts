import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router/index.ts";
import Notifications from '@kyvg/vue3-notification'

import '@cityssm/bulma-sticky-table/bulma-with-sticky-table.css'
import '@creativebulma/bulma-tooltip/dist/bulma-tooltip.min.css'
import '@/assets/main.css'
// import 'bulma/css/bulma.min.css'

createApp(App)
    .use(router)
    .use(Notifications)
    .mount("#app");
