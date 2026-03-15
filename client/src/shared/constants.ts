export const SCRIPTS_NAME = 'scripts';
export const RUN_NAME = 'run';

export const DEFAULT_CODE = `
import sys
import json

# NEW
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
`;