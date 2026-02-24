# Инструкция по запуску и использованию веб-сервера для динамических Python-скриптов

## Введение

Этот сервер на **Rust** (с использованием фреймворка **Axum**) позволяет:
- Хранить Python-скрипты в специальной папке (`./scripts`).
- Управлять скриптами через REST API (создание, чтение, обновление, удаление).
- Запускать скрипты по HTTP-запросу, передавая им произвольные JSON-данные (через **stdin**) и аргументы командной строки.
- Получать результат выполнения (stdout, stderr, код возврата) в формате JSON.
- Автоматически ограничивать количество одновременно выполняющихся скриптов (семафор).
- Кэшировать результаты одинаковых запросов (с учётом времени изменения файла).
- Завершать скрипты по таймауту (30 секунд).

Сервер автоматически сканирует папку `./scripts` каждые 5 секунд и подхватывает новые/удалённые `.py` файлы.

---

## Предварительные требования

- **Rust** (установка: [rustup.rs](https://rustup.rs/)) – для компиляции и запуска сервера.
- **Python 3** (должен быть доступен в системе как `python3`) – для выполнения скриптов.
- **curl** или аналогичный инструмент для тестирования API (можно использовать Postman, Insomnia и т.д.).

---

## Настройка проекта

1. Создайте новый проект Rust:
   ```bash
   cargo new rust_python_axum
   cd rust_python_axum
   ```

2. Замените содержимое файла `Cargo.toml` на следующий:
   ```toml
   [package]
   name = "rust_python_axum"
   version = "0.2.0"
   edition = "2021"

   [dependencies]
   axum = "0.7"
   tokio = { version = "1", features = ["full"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   thiserror = "1"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   futures = "0.3"
   bytes = "1"
   wait-timeout = "0.2"
   ```

3. Скопируйте полный код сервера (из предыдущего ответа) в файл `src/main.rs`.

4. Создайте директорию для скриптов:
   ```bash
   mkdir scripts
   ```

---

## Запуск сервера

В терминале, находясь в корне проекта, выполните:
```bash
cargo run
```

Вы должны увидеть примерно такой вывод:
```
2025-01-01T12:00:00.123456Z  INFO rust_python_axum: Server listening on http://0.0.0.0:3000
2025-01-01T12:00:00.123789Z  INFO rust_python_axum: Scanned scripts: found 0 scripts
```

Сервер запущен и слушает порт `3000` на всех интерфейсах.

---

## API Reference

### 1. Получить список всех доступных скриптов
**GET** `/scripts`

Пример запроса:
```bash
curl http://localhost:3000/scripts
```

Ответ:
```json
["example.py", "another.py"]
```

### 2. Создать новый скрипт
**POST** `/scripts`

Тело запроса (JSON):
```json
{
  "name": "hello.py",
  "code": "import sys, json; print(json.dumps({\"message\": \"Hello, \" + json.load(sys.stdin)[\"name\"]}))"
}
```

- `name` – имя файла (должно оканчиваться на `.py` и не содержать путь).
- `code` – содержимое скрипта.

Пример:
```bash
curl -X POST http://localhost:3000/scripts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "hello.py",
    "code": "import sys, json; data = json.load(sys.stdin); print(json.dumps({\"greeting\": f\"Hello, {data[\"name\"]}\"}))"
  }'
```

При успехе возвращается статус `201 Created`.

### 3. Обновить существующий скрипт
**PUT** `/scripts/{name}`

Тело запроса (JSON):
```json
{
  "code": "новый код скрипта"
}
```

Пример:
```bash
curl -X PUT http://localhost:3000/scripts/hello.py \
  -H "Content-Type: application/json" \
  -d '{"code": "print(\"new version\")"}'
```

Возвращает `200 OK` или ошибку, если скрипт не найден.

### 4. Удалить скрипт
**DELETE** `/scripts/{name}`

Пример:
```bash
curl -X DELETE http://localhost:3000/scripts/hello.py
```

Возвращает `204 No Content` при успехе.

### 5. Запустить один конкретный скрипт
**POST** `/run/{name}`

Тело запроса (JSON):
```json
{
  "data": { "любые": "данные" },
  "args": ["--option", "value"]   // опционально
}
```

- `data` – произвольный JSON, который будет передан скрипту через stdin.
- `args` – массив строк, которые будут переданы как аргументы командной строки скрипту.

Пример:
```bash
curl -X POST http://localhost:3000/run/hello.py \
  -H "Content-Type: application/json" \
  -d '{"data": {"name": "Alice"}, "args": ["--verbose"]}'
```

Ответ:
```json
{
  "stdout": "{\"greeting\":\"Hello, Alice\"}",
  "stderr": "",
  "exit_code": 0,
  "timed_out": false
}
```

### 6. Запустить несколько скриптов (все или выбранные)
**POST** `/run`

Можно указать параметр `?names=script1.py,script2.py` для запуска конкретных скриптов. Если параметр не указан, будут запущены **все** скрипты из папки.

Тело запроса такое же, как для одиночного запуска.

Пример (запустить все скрипты):
```bash
curl -X POST http://localhost:3000/run \
  -H "Content-Type: application/json" \
  -d '{"data": {"x": 42}}'
```

Ответ:
```json
{
  "results": {
    "hello.py": {
      "stdout": "{\"greeting\":\"Hello, Alice\"}",
      "stderr": "",
      "exit_code": 0,
      "timed_out": false
    },
    "another.py": {
      "stdout": "{\"result\":84}",
      "stderr": "",
      "exit_code": 0,
      "timed_out": false
    }
  }
}
```

Пример (запустить только два скрипта):
```bash
curl -X POST "http://localhost:3000/run?names=hello.py,another.py" \
  -H "Content-Type: application/json" \
  -d '{"data": {"value": 100}}'
```

---

## Примеры использования

### Создание простого скрипта

Создадим скрипт `math.py`, который принимает число и возвращает его квадрат.

Запрос:
```bash
curl -X POST http://localhost:3000/scripts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "math.py",
    "code": "import sys, json; data = json.load(sys.stdin); result = data[\"x\"] ** 2; print(json.dumps({\"square\": result}))"
  }'
```

Проверим, что скрипт появился в списке:
```bash
curl http://localhost:3000/scripts
```
Должны увидеть `["math.py"]`.

### Запуск с данными

```bash
curl -X POST http://localhost:3000/run/math.py \
  -H "Content-Type: application/json" \
  -d '{"data": {"x": 7}}'
```

Ответ:
```json
{
  "stdout": "{\"square\":49}",
  "stderr": "",
  "exit_code": 0,
  "timed_out": false
}
```

### Передача аргументов командной строки

Скрипт может использовать `sys.argv`. Например, создадим скрипт `args_demo.py`:

```bash
curl -X POST http://localhost:3000/scripts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "args_demo.py",
    "code": "import sys, json; print(json.dumps({\"args\": sys.argv[1:], \"data\": json.load(sys.stdin)}))"
  }'
```

Запустим с аргументами:
```bash
curl -X POST http://localhost:3000/run/args_demo.py \
  -H "Content-Type: application/json" \
  -d '{"data": {"foo": "bar"}, "args": ["--debug", "42"]}'
```

Ответ покажет переданные аргументы и данные:
```json
{
  "stdout": "{\"args\":[\"--debug\",\"42\"],\"data\":{\"foo\":\"bar\"}}",
  "stderr": "",
  "exit_code": 0,
  "timed_out": false
}
```

### Обработка ошибок и таймаутов

Если скрипт выбрасывает исключение, stderr будет содержать traceback, а exit_code станет ненулевым. Также можно протестировать таймаут, создав скрипт с бесконечным циклом:

```bash
curl -X POST http://localhost:3000/scripts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "infinite.py",
    "code": "while True: pass"
  }'
```

Запрос на запуск:
```bash
curl -X POST http://localhost:3000/run/infinite.py -d '{"data":{}}'
```

Через 30 секунд сервер вернёт ошибку 504 (Gateway Timeout) с сообщением "Script execution timed out".

---

## Написание Python-скриптов: требования

Чтобы скрипт корректно взаимодействовал с сервером, необходимо соблюдать простые правила:

1. **Чтение входных данных**: скрипт должен читать **весь stdin** (это JSON, присланный в поле `data`). Рекомендуется использовать `sys.stdin.read()`.
2. **Вывод результата**: результат должен быть выведен в **stdout** в виде **одной JSON-строки**. Можно использовать `print(json.dumps(...))`. После вывода можно завершиться.
3. **Обработка ошибок**: при возникновении ошибки можно вывести JSON с полем `error` или просто записать сообщение в stderr (оно попадёт в поле `stderr` ответа).
4. **Без бесконечных циклов** – скрипт должен завершаться. Если он будет работать дольше 30 секунд, сервер убьёт его по таймауту (но вернёт ошибку, а не результат).
5. **Необязательно использовать `sys.stdout.flush()`**, так как после завершения процесса буфер сбрасывается автоматически, но если вы хотите выводить промежуточные результаты (не рекомендуется для данного API), то flush необходим.

### Пример шаблона скрипта

```python
import sys
import json

def main():
    # Читаем всё из stdin
    raw_data = sys.stdin.read()
    if not raw_data:
        result = {"error": "No input provided"}
    else:
        try:
            data = json.loads(raw_data)
            # Здесь ваша логика
            result = {"received": data, "status": "ok"}
        except Exception as e:
            result = {"error": str(e)}

    # Выводим результат как JSON
    print(json.dumps(result))

if __name__ == "__main__":
    main()
```

---

## Примечания и рекомендации

- **Безопасность**: не используйте этот сервер в открытом интернете без аутентификации, так как любой может создавать и запускать произвольный Python-код на вашем сервере.
- **Производительность**: каждый запуск скрипта создаёт новый процесс Python. Для частых лёгких вызовов это может быть накладно. Рассмотрите возможность пуллинга процессов или использования более лёгких механизмов (например, Wasm), если требуется высокая частота.
- **Кэширование**: кэш учитывает входные данные (`data` и `args`) и время изменения файла. Если скрипт использует внешние ресурсы (API, файлы), результат может устареть – в таком случае кэш лучше отключить или сократить TTL.
- **Лимиты**: по умолчанию одновременно могут выполняться не более 4 скриптов (параметр `max_concurrent` в `AppState::new`). При превышении лимита запросы будут ждать освобождения слота.
- **Таймаут**: жёстко задан 30 секунд. Если нужно изменить, отредактируйте `Duration::from_secs(30)` в коде.

---

## Заключение

Вы получили полнофункциональный веб-сервер на Rust, который позволяет динамически управлять Python-скриптами и запускать их через HTTP. Это может быть полезно для:
- Создания плагинной системы.
- Выполнения пользовательского кода в изолированной среде (с осторожностью).
- Интеграции Python-логики в микросервисную архитектуру.

Сервер легко расширить: добавить аутентификацию, ограничение по памяти/CPU для процессов, поддержку других языков, асинхронные очереди и т.д.

Если у вас возникнут вопросы или потребуется доработка, обращайтесь!
# runner-programming-language-scripts
