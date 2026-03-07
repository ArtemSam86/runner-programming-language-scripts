import requests
import json

# 1. GET-запрос: Получение списка постовWWWWWW444444444
url = 'https://jsonplaceholder.typicode.com/posts'
response = requests.get(url)

# Проверка успешности запроса
if response.status_code == 200:
    posts = response.json()  # Преобразование JSON в список Python
    print(json.dumps(posts))
else:
    print(json.dumps({ "error": "Ошибка запроса" }))