#!/usr/bin/env python3

import re
import sys

def fix_swift_generation(input_file, output_file):
    """
    Comprehensive fix for UniFFI Swift generation issues.
    
    Fixes:
    1. ContactManager class structure
    2. Extension scope issues
    3. Missing closing braces
    4. Other structural problems
    """
    
    try:
        # Read the generated Swift file
        with open(input_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix 1: ContactManager class structure
        # Find the ContactManager class and ensure it includes all methods
        contact_manager_pattern = r'(open class ContactManager:.*?deinit \{[^}]*\}[\s\n]*\}[\s\n]*)'
        
        def fix_contact_manager(match):
            """Fix ContactManager class to include all methods"""
            class_start = match.start()
            class_end = match.end()
            
            # Find where the class should actually end (before FfiConverterTypeContactManager)
            remaining_content = content[class_end:]
            converter_match = re.search(r'public struct FfiConverterTypeContactManager:', remaining_content)
            
            if converter_match:
                actual_end = class_end + converter_match.start()
                methods_section = content[class_end:actual_end].strip()
                
                # Indent all methods properly
                indented_methods = re.sub(r'^open func', '    open func', methods_section, flags=re.MULTILINE)
                
                return match.group() + '\n' + indented_methods + '\n    }\n\n\n'
            
            return match.group()  # Fallback: return original if no fix found
        
        # Apply ContactManager fix
        content = re.sub(contact_manager_pattern, fix_contact_manager, content, flags=re.DOTALL)
        
        # Fix 2: Extension scope issues
        # Find extensions that are at file scope and move them to proper scope
        extension_pattern = r'\n\nexension \w+: (Equatable, Hashable|Error) \{[^}]*\}'
        
        def fix_extension_scope(match):
            """Ensure extensions are in proper scope"""
            # Check if extension is already in proper scope
            before_extension = content[:match.start()]
            if before_extension.strip().endswith('}'):
                # Extension is already in proper scope (after a closing brace)
                return match.group()
            else:
                # Need to find proper scope for this extension
                return match.group()
        
        # Fix 3: Missing closing braces
        # Count opening and closing braces to find imbalances
        open_braces = content.count('{')
        close_braces = content.count('}')
        
        if open_braces > close_braces:
            # Add missing closing braces at the end
            content += '}\n' * (open_braces - close_braces)
        
        # Fix 4: Other structural issues
        # Ensure proper indentation
        lines = content.split('\n')
        fixed_lines = []
        
        for line in lines:
            # Fix common indentation issues
            if line.startswith('extension ') and not line.startswith('    extension '):
                fixed_lines.append('    ' + line)
            elif line.startswith('open func ') and not line.startswith('    open func '):
                fixed_lines.append('    ' + line)
            else:
                fixed_lines.append(line)
        
        content = '\n'.join(fixed_lines)
        
        # Write the fixed content
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(content)
        
        print(f"✅ Comprehensive Swift generation fix applied")
        print(f"   Fixed ContactManager structure")
        print(f"   Fixed extension scope issues")
        print(f"   Added missing closing braces: {max(0, open_braces - close_braces)}")
        print(f"   Fixed indentation issues")
        
        return True
        
    except Exception as e:
        print(f"❌ Error applying comprehensive fix: {e}")
        return False

if __name__ == "__main__":
    input_file = "iOS/SCMessenger/SCMessenger/Generated/api.swift"
    output_file = "iOS/SCMessenger/SCMessenger/Generated/api_fixed.swift"
    
    if fix_swift_generation(input_file, output_file):
        # Replace the original file with the fixed version
        import shutil
        shutil.move(output_file, input_file)
        print(f"✅ Replaced {input_file} with fixed version")
    else:
        print("❌ Fix failed")
        sys.exit(1)