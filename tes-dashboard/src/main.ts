import ApiService from "./services/api.service.ts"
import App from './App.vue'
import {createApp} from 'vue'
import { createPinia } from 'pinia'
import {registerPlugins} from '@/plugins'
import router from './router.ts'

import '../node_modules/roboto-fontface/css/roboto/roboto-fontface.css'

ApiService.init()

if (import.meta.env.VITE_APP_AUTH_HEADER) {
    ApiService.setHeader()
}

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)

registerPlugins(app)

app.mount('#app')