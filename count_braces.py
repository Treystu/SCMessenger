with open('cli\\src\\transport_bridge.rs', 'r') as f:
    content = f.read()
    
# Count opening and closing braces
open_braces = content.count('{')
close_braces = content.count('}')
print(f'Opening braces: {open_braces}')
print(f'Closing braces: {close_braces}')
print(f'Difference: {close_braces - open_braces}')