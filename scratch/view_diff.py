with open("tmp/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK_response.md", "r", encoding="utf-8") as f:
    content = f.read()

import re
blocks = list(re.finditer(r"```(diff|kotlin)\n(.*?)\n```", content, re.DOTALL))
if len(blocks) > 1:
    print(blocks[1].group(2))
else:
    print("Only one block found")
