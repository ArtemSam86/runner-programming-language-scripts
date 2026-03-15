# ---- Build stage ----
FROM node:20-alpine AS builder
WORKDIR /app

# Копируем package-файлы и устанавливаем зависимости
COPY package*.json ./
RUN npm ci

# Копируем исходный код и собираем приложение
COPY . .
RUN npm run build

# ---- Production stage ----
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html

# Конфигурация nginx для SPA (перенаправление всех путей на index.html)
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]