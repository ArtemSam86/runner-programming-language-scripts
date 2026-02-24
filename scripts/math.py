import json
import sys

data = json.load(sys.stdin)
result = data["x"] ** 2
print(json.dumps({"square": result}))
