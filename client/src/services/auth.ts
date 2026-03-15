import type { LoginRequest, LoginResponse, RegisterRequest } from '@/shared/types/auth.ts';
import HttpClient from '@/services/httpClient.ts';
import {LOGIN_NAME, REGISTER_NAME} from '@/router/constants.ts';

export const authService = {
    async login(credentials: LoginRequest) {
        const response =
            await HttpClient.post<LoginResponse>(`/${LOGIN_NAME}`, credentials);
        return response.data;
    },

    async register(data: RegisterRequest) {
        await HttpClient.post(`/${REGISTER_NAME}`, data);
    },

    logout() {
        localStorage.removeItem('token');
        localStorage.removeItem('username');
    },

    saveToken(token: string, username: string) {
        localStorage.setItem('token', token);
        localStorage.setItem('username', username);
    },

    getToken() {
        return localStorage.getItem('token');
    },

    getUsername() {
        return localStorage.getItem('username');
    }
};