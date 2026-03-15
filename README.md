# runner-programming-language-scripts

## Описание
Проект представляет собой сервер для выполнения Python-скриптов с возможностью их хранения, редактирования и запуска через API. Реализована аутентификация через JWT, хранение метаданных в MongoDB, кэширование результатов выполнения и автоматическое сканирование директории со скриптами. Клиентская часть написана на Vue 3 и взаимодействует с API.

## Технологии
- **Backend**: Rust, Axum, MongoDB, JWT, bcrypt.
- **Frontend**: Vue 3, TypeScript, Vite.
- **Инфраструктура**: Docker, Docker Compose.

---

## Требования
- Установленные [Docker](https://docs.docker.com/get-docker/) и [Docker Compose](https://docs.docker.com/compose/install/) (версии 2.x).
- Порт 3000 (бэкенд) и 8081 (клиент) должны быть свободны на хосте.
- Для локальной разработки без Docker: Rust (1.88+), Node.js (20+), MongoDB.

---

## Быстрый старт (Docker Compose)

1. **Клонируйте репозиторий**:
   ```bash
   git clone git@github.com:ArtemSam86/runner-programming-language-scripts.git
   cd runner-programming-language-scripts
   ```

2. **Настройте переменные окружения**:
   Создайте файл `.env` в корне проекта (рядом с `docker-compose.yml`) со следующим содержимым:
   ```env
   JWT_SECRET=your-secure-jwt-secret-min-32-chars
   SUPER_ADMIN_NAME=admin
   SUPER_ADMIN_PASSWORD=strong-password
   MONGO_URI=mongodb://mongodb:27017
   MONGO_DB_NAME=script_manager
   ALLOWED_ORIGINS=http://localhost:8081
   CORS_ALLOW_CREDENTIALS=false
   RUST_LOG=info
   ```
   *Замените значения на свои (особенно JWT_SECRET и пароль суперадмина).*

3. **Запустите контейнеры**:
   ```bash
   docker-compose up -d
   ```
   При первом запуске будут скачаны образы, собраны бэкенд и клиент, создана база данных и автоматически добавлен суперадминистратор (если пользователей нет).

4. **Проверьте доступность сервисов**:
   - Бэкенд API: http://localhost:3000
   - Клиент: http://localhost:8081
   - Swagger UI: http://localhost:3000/swagger-ui

5. **Остановка**:
   ```bash
   docker-compose down
   ```
   Для полной остановки с удалением томов (стереть данные БД): `docker-compose down -v`.

---

## Локальный запуск без Docker (для разработки)

### Backend
1. Установите Rust (версия 1.88+) и MongoDB (локально или через Docker).
2. Создайте файл `.env` в корне проекта (см. пример выше).
3. Запустите:
   ```bash
   cargo run
   ```
   Бэкенд будет доступен на `http://localhost:3000`.

### Frontend
1. Перейдите в папку `client`:
   ```bash
   cd client
   ```
2. Установите зависимости:
   ```bash
   npm install
   ```
3. Создайте файл `.env.local` со ссылкой на API:
   ```
   VITE_API_URL=http://localhost:3000
   ```
4. Запустите dev-сервер:
   ```bash
   npm run dev
   ```
   Клиент будет доступен на `http://localhost:5173` (или другом свободном порту).

---

## API Документация

### Аутентификация

#### `POST /register`
Регистрация нового пользователя.
- **Тело запроса**:
  ```json
  {
    "username": "user",
    "password": "pass"
  }
  ```
- **Ответы**:
   - `201 Created` – пользователь создан.
   - `409 Conflict` – пользователь уже существует.
   - `400 Bad Request` – некорректные данные.

#### `POST /login`
Вход в систему, получение JWT-токена.
- **Тело запроса**:
  ```json
  {
    "username": "user",
    "password": "pass"
  }
  ```
- **Ответ** (`200 OK`):
  ```json
  {
    "token": "eyJhbGciOiJIUzI1NiIs...",
    "username": "user"
  }
  ```
- **Ошибки**:
   - `401 Unauthorized` – неверные учётные данные.

### Управление скриптами (требуют JWT в заголовке `Authorization: Bearer <token>`)

#### `GET /scripts?query=...&sort_by=...&sort_order=...`
Получить список всех скриптов с фильтрацией и сортировкой.
- **Параметры запроса** (опционально):
   - `query` – строка для поиска по имени, коду, описанию и т.д.
   - `sort_by` – поле сортировки: `name`, `size`, `created`, `modified` (по умолчанию `name`).
   - `sort_order` – `asc` или `desc` (по умолчанию `asc`).
- **Ответ**:
  ```json
  [
    {
      "name": "script.py",
      "code": "...",
      "description": null,
      "result": null,
      "size": 1234,
      "created": "2026-03-15T12:00:00Z",
      "modified": "2026-03-15T12:30:00Z"
    }
  ]
  ```

#### `GET /scripts/{name}`
Получить конкретный скрипт по имени.
- **Ответ**: аналогично объекту из списка.

#### `POST /scripts`
Создать новый скрипт.
- **Тело запроса**:
  ```json
  {
    "name": "script.py",
    "code": "print('Hello')",
    "description": "optional description",
    "result": "optional expected result"
  }
  ```
- **Ответ**: `201 Created`.

#### `PUT /scripts/{name}`
Обновить существующий скрипт (частичное обновление).
- **Тело запроса** (все поля опциональны):
  ```json
  {
    "code": "new code",
    "description": "new description",
    "result": "new result"
  }
  ```
- **Ответ**: `200 OK` с обновлённым объектом скрипта.

#### `DELETE /scripts/{name}`
Удалить скрипт.
- **Ответ**: `204 No Content`.

### Выполнение скриптов

#### `POST /run?names=...`
Запустить один или несколько скриптов (имена через запятую). Если `names` не указан, выполняются все скрипты.
- **Параметры запроса**: `names` – список имён через запятую.
- **Тело запроса**:
  ```json
  {
    "data": { "any": "json" },
    "args": ["--arg1", "value"]
  }
  ```
  `args` опционален.
- **Ответ**:
  ```json
  {
    "results": {
      "script1.py": {
        "stdout": "...",
        "stderr": "...",
        "exit_code": 0,
        "timed_out": false
      }
    }
  }
  ```

#### `POST /run/{name}`
Запустить один скрипт по имени.
- **Тело запроса**: аналогично `/run`.
- **Ответ**: объект `ScriptResult`.

---

## Переменные окружения

| Переменная             | Описание                                                                        | Значение по умолчанию |
|------------------------|---------------------------------------------------------------------------------|-----------------------|
| `MONGO_URI`            | Адрес MongoDB (в Docker используйте `mongodb://mongodb:27017`)                  | `mongodb://localhost:27017` |
| `MONGO_DB_NAME`        | Имя базы данных                                                                 | `script_manager`      |
| `JWT_SECRET`           | Секретный ключ для подписи JWT (минимум 32 символа)                             | **обязательно**       |
| `SUPER_ADMIN_NAME`     | Имя суперадминистратора (создаётся при первом запуске)                          | `superadmin`          |
| `SUPER_ADMIN_PASSWORD` | Пароль суперадминистратора                                                      | **обязательно**       |
| `ALLOWED_ORIGINS`      | Разрешённые источники для CORS (через запятую). Для разработки можно `*`.      | (все)                 |
| `CORS_ALLOW_CREDENTIALS`| Разрешить отправку credentials (cookies, заголовки авторизации)                | `false`               |
| `RUST_LOG`             | Уровень логирования (`info`, `debug`, `warn`, `error`)                          | `info`                |

---

## Структура проекта (бэкенд)

```
src/
├── main.rs                 # точка входа, миграции, запуск
├── app_state.rs            # состояние приложения (кэш, пулы)
├── auth_middleware.rs      # JWT-мидлварь
├── db.rs                   # работа с MongoDB, модели
├── error.rs                # кастомные ошибки и IntoResponse
├── handlers.rs             # обработчики HTTP-запросов
├── jwt.rs                  # создание и проверка JWT
├── migrations/             # миграции базы данных
│   ├── mod.rs
│   ├── v1_*.rs
│   └── ...
├── models.rs               # структуры запросов/ответов (с аннотациями Swagger)
├── script_runner.rs        # логика выполнения скриптов, кэширование
└── utils.rs                # вспомогательные функции
```

---

## Примечания по разработке

- **Swagger UI** доступен по адресу `/swagger-ui`. Для авторизации введите полученный JWT-токен в поле Authorize (кнопка с замком).
- При добавлении новых зависимостей в `Cargo.toml` не забывайте обновлять версии и проверять совместимость с образами Docker.
- Для тестирования API можно использовать `curl` или Postman.
- Логи контейнеров смотрите командой `docker-compose logs -f`.

## Часто задаваемые вопросы

**Q:** Почему после перезагрузки Swagger UI токен исчезает?
**A:** Это особенность браузерного хранения – токен хранится только в памяти страницы. Просто введите его заново.

**Q:** Как добавить новый Python-пакет в образ?
**A:** В Dockerfile (runtime-стадия) добавьте пакет через `apk add py3-имяпакета`. Если пакета нет в репозитории Alpine, используйте виртуальное окружение с `pip`.

**Q:** Где хранятся скрипты?
**A:** В папке `scripts` на хосте, смонтированной в контейнер. Вы можете добавлять/удалять файлы прямо там.

---

## Лицензия
MIT