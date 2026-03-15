import {computed, type ComputedRef, type Ref, ref } from 'vue';
import { authService } from '@/services/auth';
import type { LoginRequest, LoginResponse, RegisterRequest } from '@/shared/types/auth.ts';
import router from "@/router";
import {LOGIN_NAME} from "@/router/constants.ts";

interface UseAuth {
    user: Ref<string | null>;
    token: Ref<string | null>
    login: (credentials: LoginRequest) => Promise<LoginResponse>;
    register: (data: RegisterRequest) => Promise<void>;
    logout: () => void;
    isAuthenticated: ComputedRef<boolean>;
}

export const useAuth = (): UseAuth => {
    const user = ref<string | null>(authService.getUsername());
    const token = ref<string | null>(authService.getToken());

    const isAuthenticated = computed(() => !!token.value);

    const login = async (credentials: LoginRequest) => {
        const response = await authService.login(credentials);
        authService.saveToken(response.token, response.username);
        user.value = response.username;
        token.value = response.token;
        return response;
    };

    const register = async (data: RegisterRequest) => {
        await authService.register(data);
    };

    const logout = () => {
        authService.logout();
        user.value = null;
        token.value = null;
        router.push({ name: LOGIN_NAME });
        // window.location.href = '/login';
    };

    return {
        user,
        token,
        isAuthenticated,
        login,
        register,
        logout
    };
}