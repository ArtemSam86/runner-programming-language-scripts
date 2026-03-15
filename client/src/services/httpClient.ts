import axios from 'axios';
import router from '@/router';
import {LOGIN_NAME} from "@/router/constants.ts";

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

const api = axios.create({
  baseURL: API_URL,
  headers: {
  "Content-Type": "application/json",
  // "Access-Control-Allow-Origin": "*",
  // Accept: "application/json",
  },
});

api.interceptors.request.use(
    (config) => {
      const token = localStorage.getItem('token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    },
    (error) => Promise.reject(error)
);

// Обработка ошибок 401 – перенаправление на логин
api.interceptors.response.use(
    (response) => response,
    (error) => {
      if (error.response?.status === 401) {
        // Очистить хранилище и перенаправить на страницу логина
        localStorage.removeItem('token');
        localStorage.removeItem('username');
        router.push({ name: LOGIN_NAME });
        // window.location.href = '/login';
      }
      return Promise.reject(error);
    }
);

export default api;