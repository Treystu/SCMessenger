import sys

path = '/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Generated/api.swift'
with open(path, 'r') as f:
    code = f.read()

# Replace "nonisolated(unsafe) static func" with "static func"
code = code.replace('nonisolated(unsafe) static func', 'static func')
code = code.replace('public nonisolated(unsafe) static func', 'public static func')

# Write back
with open(path, 'w') as f:
    f.write(code)
print("api.swift patched successfully.")
