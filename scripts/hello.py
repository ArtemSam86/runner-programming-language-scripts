import json
import os
import sys

data = json.load(sys.stdin)
json_data = json.dumps(
    {
        "curdir": os.curdir,
        "name": os.name,
        "cpu_count": os.cpu_count(),
    }
)
print(json_data)
print(json.dumps(data))
