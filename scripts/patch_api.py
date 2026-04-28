import re
import sys

path = '/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/iOS/SCMessenger/SCMessenger/Generated/api.swift'
with open(path, 'r') as f:
    code = f.read()

# Replace nonisolated(unsafe) with nonisolated
code = code.replace('nonisolated(unsafe) static func', 'nonisolated static func')
code = code.replace('public nonisolated(unsafe) static func', 'public nonisolated static func')

# Add nonisolated to fileprivate functions
code = re.sub(r'fileprivate func (read|write|createReader|createWriter|hasRemaining)', r'nonisolated fileprivate func \1', code)

# Add nonisolated to extensions and structs FfiConverter... if there are errors?
# Actually the easiest way to make all global functions nonisolated is:
code = code.replace('fileprivate func', 'nonisolated fileprivate func')
code = code.replace('private func', 'nonisolated private func')

# Methods in RustBuffer and ForeignBytes extensions
code = re.sub(r'(fileprivate extension RustBuffer {\n)((?:.|\n)*?)(^\})', lambda m: m.group(1) + m.group(2).replace('    init', '    nonisolated init').replace('    static func', '    nonisolated static func').replace('    func', '    nonisolated func') + m.group(3), code, flags=re.MULTILINE)

code = re.sub(r'(fileprivate extension ForeignBytes {\n)((?:.|\n)*?)(^\})', lambda m: m.group(1) + m.group(2).replace('    init', '    nonisolated init') + m.group(3), code, flags=re.MULTILINE)

code = re.sub(r'(fileprivate extension Data {\n)((?:.|\n)*?)(^\})', lambda m: m.group(1) + m.group(2).replace('    init', '    nonisolated init') + m.group(3), code, flags=re.MULTILINE)

code = re.sub(r'(fileprivate extension NSLock {\n)((?:.|\n)*?)(^\})', lambda m: m.group(1) + m.group(2).replace('    func', '    nonisolated func') + m.group(3), code, flags=re.MULTILINE)

# Remove duplicates if any
code = code.replace('nonisolated nonisolated', 'nonisolated')
code = code.replace('public nonisolated nonisolated', 'public nonisolated')

with open(path, 'w') as f:
    f.write(code)
print("api.swift patched successfully.")
