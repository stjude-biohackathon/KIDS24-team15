import { createRouter, createWebHistory } from 'vue-router'

const routes = [
    {
        path: '/:pathMatch(.*)*',
        redirect: '/',
    },
    {
        path: '/',
        name: 'home',
        component: () => import('@/views/TESDashboard.vue'),
    },
]

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes
})

/** Logic needed for processing before handling each route **/

export default router