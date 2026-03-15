import { createWebHistory, createRouter, type RouteRecordRaw} from 'vue-router';
import PanelPage from '@/pages/PanelPage.vue';
import LoginPage from '@/pages/LoginPage.vue';
import RegisterPage from '@/pages/RegisterPage.vue';
import {useAuth} from "@/composables/useAuth.ts";
import {
    LOGIN_NAME,
    LOGIN_PATH,
    PANEL_NAME,
    PANEL_PATH,
    REGISTER_NAME,
    REGISTER_PATH
} from "@/router/constants.ts";

const routes: RouteRecordRaw[] = [
    {
        path: '/',
        redirect: PANEL_PATH,
    },
    {
        meta: { requiresAuth: true },
        name: PANEL_NAME,
        path: PANEL_PATH,
        component: PanelPage
    },
    {
        meta: { public: true },
        name: LOGIN_NAME,
        path: LOGIN_PATH,
        component: LoginPage
    },
    {
        meta: { public: true },
        name: REGISTER_NAME,
        path: REGISTER_PATH,
        component: RegisterPage
    },
];

const router = createRouter({
    history: createWebHistory(),
    routes,
})

router.beforeEach((to, _) => {
    const { isAuthenticated } = useAuth();
    const authRequired = !!to.meta.requiresAuth && !isAuthenticated.value;
    const guestOnly = !!to.meta.public && isAuthenticated.value
        && (to.path === LOGIN_PATH || to.path === REGISTER_PATH);

    if (authRequired) {
        return { name: LOGIN_NAME };
    }
    if (guestOnly) {
        return { name: PANEL_NAME };
    }
    return true;
});

export default router;